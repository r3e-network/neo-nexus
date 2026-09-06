//! Shared fail-closed reader for files containing long-lived credentials.

use std::{fs::File, io::Read, path::Path};

use anyhow::{bail, Context, Result};
use zeroize::Zeroizing;

/// Read one regular, non-link credential file after enforcing the native
/// platform's owner-only permission model.
pub(crate) fn read_secret(path: &Path, limit: u64, label: &str) -> Result<Zeroizing<Vec<u8>>> {
    reject_link(path, label)?;
    let file = open_without_following_links(path)
        .with_context(|| format!("failed to open {label} file {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect {label} file {}", path.display()))?;
    if !metadata.is_file() {
        bail!("{label} path {} is not a regular file", path.display());
    }
    if metadata.len() > limit {
        bail!(
            "{label} file {} exceeds the {limit}-byte limit",
            path.display()
        );
    }
    validate_permissions(&file, &metadata, path, label)?;

    let mut bytes = Zeroizing::new(Vec::new());
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read {label} file {}", path.display()))?;
    if bytes.len() as u64 > limit {
        bail!(
            "{label} file {} exceeds the {limit}-byte limit",
            path.display()
        );
    }
    Ok(bytes)
}

fn reject_link(path: &Path, label: &str) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {label} path {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!(
            "{label} path {} must not be a symbolic link",
            path.display()
        );
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            bail!(
                "{label} path {} must not be a Windows reparse point",
                path.display()
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn open_without_following_links(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_without_following_links(path: &Path) -> std::io::Result<File> {
    File::open(path)
}

#[cfg(unix)]
fn validate_permissions(
    _file: &File,
    metadata: &std::fs::Metadata,
    path: &Path,
    label: &str,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if metadata.permissions().mode() & 0o077 != 0 {
        bail!(
            "{label} file {} must not be readable, writable, or executable by group or other users",
            path.display()
        );
    }
    Ok(())
}

#[cfg(windows)]
fn validate_permissions(
    file: &File,
    _metadata: &std::fs::Metadata,
    path: &Path,
    label: &str,
) -> Result<()> {
    windows::validate_acl(file, path, label)
}

#[cfg(not(any(unix, windows)))]
fn validate_permissions(
    _file: &File,
    _metadata: &std::fs::Metadata,
    path: &Path,
    label: &str,
) -> Result<()> {
    bail!(
        "cannot verify {label} file permissions on this platform: {}",
        path.display()
    )
}

#[cfg(test)]
pub(crate) fn protect_for_test(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
    #[cfg(windows)]
    {
        windows::replace_test_acl(path, None)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        bail!("credential-file tests require Unix or Windows permission support")
    }
}

#[cfg(windows)]
mod windows {
    use std::{
        ffi::c_void,
        mem::size_of,
        os::windows::io::AsRawHandle,
        path::Path,
        ptr::{addr_of, null_mut},
    };

    use anyhow::{bail, Context, Result};
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::{
        Foundation::{
            LocalFree, GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE, HANDLE,
        },
        Security::{
            EqualSid, GetAce, GetLengthSid, IsValidAcl, IsValidSid, IsWellKnownSid,
            WinAccountDomainGuestsSid, WinAccountDomainUsersSid, WinAnonymousSid,
            WinAuthenticatedUserSid, WinBuiltinAdministratorsSid, WinBuiltinGuestsSid,
            WinBuiltinUsersSid, WinCreatorOwnerRightsSid, WinLocalSystemSid, WinWorldSid,
            ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION, INHERIT_ONLY_ACE,
            OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        },
        Storage::FileSystem::{
            DELETE, FILE_APPEND_DATA, FILE_EXECUTE, FILE_READ_DATA, FILE_WRITE_DATA, WRITE_DAC,
            WRITE_OWNER,
        },
        System::SystemServices::{
            ACCESS_ALLOWED_ACE_TYPE, ACCESS_ALLOWED_CALLBACK_ACE_TYPE,
            ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE, ACCESS_ALLOWED_COMPOUND_ACE_TYPE,
            ACCESS_ALLOWED_OBJECT_ACE_TYPE,
        },
    };

    const SENSITIVE_ACCESS: u32 = FILE_READ_DATA
        | FILE_WRITE_DATA
        | FILE_APPEND_DATA
        | FILE_EXECUTE
        | DELETE
        | WRITE_DAC
        | WRITE_OWNER
        | GENERIC_READ
        | GENERIC_WRITE
        | GENERIC_EXECUTE
        | GENERIC_ALL;

    struct LocalDescriptor(PSECURITY_DESCRIPTOR);

    impl Drop for LocalDescriptor {
        fn drop(&mut self) {
            // SAFETY: GetSecurityInfo/GetNamedSecurityInfoW allocate successful
            // descriptors with LocalAlloc and ownership is held exactly once here.
            unsafe {
                LocalFree(self.0.cast());
            }
        }
    }

    pub(super) fn validate_acl(file: &std::fs::File, path: &Path, label: &str) -> Result<()> {
        let mut owner: PSID = null_mut();
        let mut dacl: *mut ACL = null_mut();
        let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: the handle remains live for the call, every output pointer is
        // writable, and the returned descriptor is released by LocalDescriptor.
        let status = unsafe {
            GetSecurityInfo(
                file.as_raw_handle() as HANDLE,
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &mut owner,
                null_mut(),
                &mut dacl,
                null_mut(),
                &mut descriptor,
            )
        };
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status as i32)).with_context(|| {
                format!(
                    "failed to read the Windows ACL for {label} file {}",
                    path.display()
                )
            });
        }
        let _descriptor = LocalDescriptor(descriptor);
        // SAFETY: owner points inside the live descriptor when non-null.
        if descriptor.is_null() || owner.is_null() || unsafe { IsValidSid(owner) } == 0 {
            bail!(
                "{label} file {} has no verifiable Windows owner",
                path.display()
            );
        }
        if broad_owner(owner) {
            bail!(
                "{label} file {} is owned by a broad Windows identity rather than an individual account, LocalSystem, or Administrators",
                path.display()
            );
        }
        // A null DACL grants full access to everyone. An empty, non-null DACL
        // grants nobody access and is therefore safe.
        if dacl.is_null() {
            bail!(
                "{label} file {} has a null Windows DACL that grants everyone access",
                path.display()
            );
        }
        // SAFETY: dacl points inside the live security descriptor.
        if unsafe { IsValidAcl(dacl) } == 0 {
            bail!(
                "{label} file {} has an invalid Windows DACL",
                path.display()
            );
        }

        // SAFETY: IsValidAcl established a readable ACL header.
        let ace_count = unsafe { (*dacl).AceCount };
        for index in 0..u32::from(ace_count) {
            let mut raw_ace: *mut c_void = null_mut();
            // SAFETY: index is within the validated ACL's AceCount range.
            if unsafe { GetAce(dacl, index, &mut raw_ace) } == 0 || raw_ace.is_null() {
                return Err(std::io::Error::last_os_error()).with_context(|| {
                    format!(
                        "failed to inspect Windows ACL entry {index} for {label} file {}",
                        path.display()
                    )
                });
            }
            // SAFETY: GetAce returned an ACE owned by the live descriptor.
            let header = unsafe { &*raw_ace.cast::<ACE_HEADER>() };
            if u32::from(header.AceFlags) & INHERIT_ONLY_ACE != 0 {
                continue;
            }
            match u32::from(header.AceType) {
                ACCESS_ALLOWED_ACE_TYPE => {
                    validate_basic_allow(raw_ace, header, owner, index, path, label)?;
                }
                ACCESS_ALLOWED_COMPOUND_ACE_TYPE
                | ACCESS_ALLOWED_OBJECT_ACE_TYPE
                | ACCESS_ALLOWED_CALLBACK_ACE_TYPE
                | ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE => {
                    bail!(
                        "{label} file {} uses unsupported conditional or object Windows allow ACL entry {index}; use direct owner/SYSTEM/Administrators entries",
                        path.display()
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn broad_owner(owner: PSID) -> bool {
        // SAFETY: callers validate owner and keep its descriptor live.
        unsafe {
            IsWellKnownSid(owner, WinWorldSid) != 0
                || IsWellKnownSid(owner, WinAuthenticatedUserSid) != 0
                || IsWellKnownSid(owner, WinBuiltinUsersSid) != 0
                || IsWellKnownSid(owner, WinBuiltinGuestsSid) != 0
                || IsWellKnownSid(owner, WinAccountDomainUsersSid) != 0
                || IsWellKnownSid(owner, WinAccountDomainGuestsSid) != 0
                || IsWellKnownSid(owner, WinAnonymousSid) != 0
        }
    }

    fn validate_basic_allow(
        raw_ace: *mut c_void,
        header: &ACE_HEADER,
        owner: PSID,
        index: u32,
        path: &Path,
        label: &str,
    ) -> Result<()> {
        if usize::from(header.AceSize) < size_of::<ACCESS_ALLOWED_ACE>() {
            bail!(
                "{label} file {} has a truncated Windows allow ACL entry {index}",
                path.display()
            );
        }
        // SAFETY: the validated header declares enough bytes for this ACE.
        let ace = unsafe { &*raw_ace.cast::<ACCESS_ALLOWED_ACE>() };
        if ace.Mask & SENSITIVE_ACCESS == 0 {
            return Ok(());
        }
        let sid: PSID = addr_of!(ace.SidStart).cast_mut().cast();
        // SAFETY: the SID starts inside the ACE returned by GetAce.
        if unsafe { IsValidSid(sid) } == 0 {
            bail!(
                "{label} file {} has an invalid SID in Windows ACL entry {index}",
                path.display()
            );
        }
        // SAFETY: IsValidSid established a readable SID.
        let sid_length = unsafe { GetLengthSid(sid) } as usize;
        let sid_offset = size_of::<ACCESS_ALLOWED_ACE>() - size_of::<u32>();
        if sid_offset + sid_length > usize::from(header.AceSize) {
            bail!(
                "{label} file {} has a truncated SID in Windows ACL entry {index}",
                path.display()
            );
        }
        // SAFETY: both SIDs were validated and remain live for these calls.
        let trusted = unsafe {
            EqualSid(sid, owner) != 0
                || IsWellKnownSid(sid, WinLocalSystemSid) != 0
                || IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0
                || IsWellKnownSid(sid, WinCreatorOwnerRightsSid) != 0
        };
        if !trusted {
            bail!(
                "{label} file {} grants read, write, or execute access to a Windows identity other than its owner, LocalSystem, or Administrators (ACL entry {index}, mask 0x{:08x})",
                path.display(),
                ace.Mask
            );
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn replace_test_acl(
        path: &Path,
        extra_reader: Option<windows_sys::Win32::Security::WELL_KNOWN_SID_TYPE>,
    ) -> Result<()> {
        use std::os::windows::ffi::OsStrExt;

        use windows_sys::Win32::Security::Authorization::{
            GetNamedSecurityInfoW, SetNamedSecurityInfoW,
        };
        use windows_sys::Win32::{
            Security::{
                AddAccessAllowedAce, CreateWellKnownSid, InitializeAcl, ACL_REVISION,
                PROTECTED_DACL_SECURITY_INFORMATION, SECURITY_MAX_SID_SIZE,
            },
            Storage::FileSystem::{FILE_ALL_ACCESS, FILE_GENERIC_READ},
        };

        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);
        let mut owner: PSID = null_mut();
        let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: wide is NUL-terminated and all output pointers are writable.
        let status = unsafe {
            GetNamedSecurityInfoW(
                wide.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                &mut owner,
                null_mut(),
                null_mut(),
                null_mut(),
                &mut descriptor,
            )
        };
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status as i32))
                .context("read test file owner");
        }
        let _descriptor = LocalDescriptor(descriptor);
        if owner.is_null() || unsafe { IsValidSid(owner) } == 0 {
            bail!("test file has no valid Windows owner SID");
        }

        let mut extra_sid = [0_u8; SECURITY_MAX_SID_SIZE as usize];
        let extra = if let Some(sid_type) = extra_reader {
            let mut length = extra_sid.len() as u32;
            // SAFETY: the fixed buffer has the documented maximum SID size.
            if unsafe {
                CreateWellKnownSid(
                    sid_type,
                    null_mut(),
                    extra_sid.as_mut_ptr().cast(),
                    &mut length,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error())
                    .context("create test Windows identity SID");
            }
            Some((extra_sid.as_mut_ptr().cast::<c_void>(), length))
        } else {
            None
        };

        let owner_length = unsafe { GetLengthSid(owner) } as usize;
        let ace_prefix = size_of::<ACCESS_ALLOWED_ACE>() - size_of::<u32>();
        let acl_bytes = size_of::<ACL>()
            + ace_prefix
            + owner_length
            + extra.map_or(0, |(_, length)| ace_prefix + length as usize);
        let words = acl_bytes.div_ceil(size_of::<usize>());
        let mut acl_storage = vec![0_usize; words];
        let acl = acl_storage.as_mut_ptr().cast::<ACL>();
        // SAFETY: acl_storage is aligned and has at least acl_bytes writable bytes.
        if unsafe { InitializeAcl(acl, acl_bytes as u32, ACL_REVISION) } == 0 {
            return Err(std::io::Error::last_os_error()).context("initialize test Windows ACL");
        }
        // SAFETY: owner is valid and the initialized ACL has capacity for this ACE.
        if unsafe { AddAccessAllowedAce(acl, ACL_REVISION, FILE_ALL_ACCESS, owner) } == 0 {
            return Err(std::io::Error::last_os_error()).context("add test owner ACL entry");
        }
        if let Some((sid, _)) = extra {
            // SAFETY: sid and the initialized ACL remain live and have reserved capacity.
            if unsafe { AddAccessAllowedAce(acl, ACL_REVISION, FILE_GENERIC_READ, sid) } == 0 {
                return Err(std::io::Error::last_os_error())
                    .context("add test untrusted ACL entry");
            }
        }
        // SAFETY: the path is NUL-terminated and acl remains live for the call.
        let status = unsafe {
            SetNamedSecurityInfoW(
                wide.as_mut_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                acl,
                null_mut(),
            )
        };
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status as i32))
                .context("set test Windows ACL");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/unit/secret_file/tests.rs"]
mod tests;
