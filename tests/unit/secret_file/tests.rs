use std::io::Write;

use anyhow::Result;
use tempfile::NamedTempFile;

use super::{protect_for_test, read_secret};

#[test]
fn protected_regular_secret_is_read_with_its_bound() -> Result<()> {
    let mut file = NamedTempFile::new()?;
    write!(file, "credential")?;
    protect_for_test(file.path())?;
    assert_eq!(
        &*read_secret(file.path(), 10, "test secret")?,
        b"credential"
    );
    assert!(read_secret(file.path(), 9, "test secret").is_err());
    Ok(())
}

#[test]
fn directories_are_not_accepted_as_secret_files() -> Result<()> {
    let directory = tempfile::tempdir()?;
    read_secret(directory.path(), 64, "test secret")
        .expect_err("a credential path must be a regular file");
    Ok(())
}

#[cfg(unix)]
#[test]
fn unix_group_read_is_rejected() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut file = NamedTempFile::new()?;
    write!(file, "credential")?;
    std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o640))?;
    let error = read_secret(file.path(), 64, "test secret")
        .expect_err("group-readable credentials must fail closed");
    assert!(error.to_string().contains("group or other"), "{error:#}");
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_ordinary_identity_read_is_rejected() -> Result<()> {
    use windows_sys::Win32::Security::{WinAuthenticatedUserSid, WinBuiltinUsersSid, WinWorldSid};

    for sid in [WinWorldSid, WinBuiltinUsersSid, WinAuthenticatedUserSid] {
        let mut file = NamedTempFile::new()?;
        write!(file, "credential")?;
        super::windows::replace_test_acl(file.path(), Some(sid))?;
        let error = read_secret(file.path(), 64, "test secret")
            .expect_err("ordinary-user-readable credentials must fail closed");
        let text = format!("{error:#}");
        assert!(text.contains("Windows identity"), "{text}");
    }
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_system_and_administrators_entries_are_accepted() -> Result<()> {
    use windows_sys::Win32::Security::{WinBuiltinAdministratorsSid, WinLocalSystemSid};

    for sid in [WinLocalSystemSid, WinBuiltinAdministratorsSid] {
        let mut file = NamedTempFile::new()?;
        write!(file, "credential")?;
        super::windows::replace_test_acl(file.path(), Some(sid))?;
        assert_eq!(
            &*read_secret(file.path(), 64, "test secret")?,
            b"credential"
        );
    }
    Ok(())
}
