use super::{model::MIB, ResourcePolicy, ResourceReading, ResourceReport, ResourceStatus};
use std::path::{Path, PathBuf};

pub(super) fn collect(workspace: &Path, policy: ResourcePolicy) -> ResourceReport {
    let mut memory = sysinfo::System::new();
    memory.refresh_memory();
    let mut readings = vec![memory_reading(
        memory.total_memory(),
        memory.available_memory(),
        &policy,
    )];
    for path in storage_paths(workspace, &policy) {
        let mut reading = ResourceReading::unknown(
            format!("disk:{}", path.display()),
            path.display().to_string(),
            "Storage statistics unavailable; verify the directory and mount",
        );
        if let Ok(space) = filesystem_space(&path) {
            reading.capacity_bytes = Some(space.total);
            reading.available_bytes = Some(space.available);
            reading.available_inodes = space.available_inodes;
            reading.status = classify_disk(&space, &policy);
            reading.message = if space.read_only {
                "Filesystem is read-only; node database writes cannot succeed".into()
            } else if space.available_inodes == Some(0) {
                "Filesystem has no available inodes".into()
            } else {
                format!(
                    "{} MiB available to this account; critical <= {} MiB, warning <= {} MiB",
                    space.available / MIB,
                    policy.disk_critical_mib,
                    policy.disk_warning_mib
                )
            };
        }
        readings.push(reading);
    }
    ResourceReport {
        checked_at_unix: now(),
        policy,
        readings,
        sample_failed: false,
    }
}

pub(super) fn unavailable(workspace: &Path, policy: ResourcePolicy) -> ResourceReport {
    let mut readings = vec![ResourceReading::unknown(
        "memory".into(),
        "Available host memory".into(),
        "Resource sampling timed out",
    )];
    for path in storage_paths(workspace, &policy) {
        readings.push(ResourceReading::unknown(
            format!("disk:{}", path.display()),
            path.display().to_string(),
            "Storage sampling timed out; the guardian is still running. Recover the mount; if sampling stays unavailable, restart the workbench",
        ));
    }
    ResourceReport {
        checked_at_unix: now(),
        policy,
        readings,
        sample_failed: true,
    }
}

fn storage_paths(workspace: &Path, policy: &ResourcePolicy) -> Vec<PathBuf> {
    let mut paths = vec![workspace.to_path_buf()];
    paths.extend(policy.storage_paths.iter().cloned());
    paths.sort();
    paths.dedup();
    paths
}

pub(super) fn memory_reading(
    total: u64,
    available: u64,
    policy: &ResourcePolicy,
) -> ResourceReading {
    let mut reading = ResourceReading::unknown(
        "memory".into(),
        "Available host memory".into(),
        "Memory statistics unavailable",
    );
    if total > 0 && available <= total {
        let percent = (available as f64 / total as f64) * 100.0;
        reading.capacity_bytes = Some(total);
        reading.available_bytes = Some(available);
        reading.status = if percent <= f64::from(policy.memory_critical_percent) {
            ResourceStatus::Critical
        } else if percent <= f64::from(policy.memory_warning_percent) {
            ResourceStatus::Warning
        } else {
            ResourceStatus::Healthy
        };
        reading.message = format!("{percent:.1}% available memory; investigate node caches and other host processes before OOM");
    }
    reading
}

pub(super) struct Space {
    pub total: u64,
    pub available: u64,
    pub available_inodes: Option<u64>,
    pub read_only: bool,
}

pub(super) fn classify_disk(space: &Space, policy: &ResourcePolicy) -> ResourceStatus {
    if space.total == 0 || space.available > space.total {
        return ResourceStatus::Unknown;
    }
    if space.read_only
        || space.available_inodes == Some(0)
        || space.available <= policy.disk_critical_mib * MIB
    {
        ResourceStatus::Critical
    } else if space.available <= policy.disk_warning_mib * MIB {
        ResourceStatus::Warning
    } else {
        ResourceStatus::Healthy
    }
}

#[cfg(windows)]
pub(super) fn filesystem_space(path: &Path) -> std::io::Result<Space> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            path: *const u16,
            available: *mut u64,
            total: *mut u64,
            free: *mut u64,
        ) -> i32;
    }
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    if !wide.ends_with(&[b'\\' as u16]) {
        wide.push(b'\\' as u16);
    }
    wide.push(0);
    let (mut available, mut total) = (0, 0);
    // SAFETY: terminated UTF-16 lives across the call; output pointers refer to
    // initialized u64 storage matching ULARGE_INTEGER's 64-bit representation.
    if unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            &mut total,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    Ok(Space {
        total,
        available,
        available_inodes: None,
        read_only: false,
    })
}

#[cfg(unix)]
pub(super) fn filesystem_space(path: &Path) -> std::io::Result<Space> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    // statvfs counters vary between u32 and u64 across Unix ABIs.
    fn counter(value: impl Into<u64>) -> u64 {
        value.into()
    }
    let path =
        CString::new(path.as_os_str().as_bytes()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
    let mut info = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: path is NUL-terminated and output is valid writable storage. Only
    // initialize the Rust value after the OS reports a successful write.
    if unsafe { libc::statvfs(path.as_ptr(), info.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let info = unsafe { info.assume_init() };
    let block = counter(if info.f_frsize > 0 {
        info.f_frsize
    } else {
        info.f_bsize
    });
    Ok(Space {
        total: counter(info.f_blocks).saturating_mul(block),
        available: counter(info.f_bavail).saturating_mul(block),
        available_inodes: (info.f_files > 0).then_some(counter(info.f_favail)),
        read_only: info.f_flag & libc::ST_RDONLY != 0,
    })
}

#[cfg(not(any(unix, windows)))]
pub(super) fn filesystem_space(_path: &Path) -> std::io::Result<Space> {
    Err(std::io::ErrorKind::Unsupported.into())
}

pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

pub(super) fn workspace_path(repository: &crate::repository::Repository) -> PathBuf {
    repository
        .db_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}
