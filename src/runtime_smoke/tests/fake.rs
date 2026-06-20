use std::{fs, path::Path, path::PathBuf};

use anyhow::Result;

pub(super) enum FakeBehavior {
    PrintAndExit(&'static str, i32),
    PrintStdoutStderrAndExit(&'static str, &'static str, i32),
    Sleep,
}

pub(super) fn fake_binary_path(root: &Path, name: &str) -> PathBuf {
    root.join(if cfg!(windows) {
        format!("{name}.cmd")
    } else {
        name.to_string()
    })
}

pub(super) fn fake_runtime_command(script_path: &Path) -> (PathBuf, Vec<String>) {
    #[cfg(windows)]
    {
        (
            PathBuf::from("cmd"),
            vec!["/C".to_string(), script_path.display().to_string()],
        )
    }
    #[cfg(not(windows))]
    {
        (
            PathBuf::from("/bin/sh"),
            vec![script_path.display().to_string()],
        )
    }
}

pub(super) fn write_fake_binary(path: &Path, behavior: FakeBehavior) -> Result<()> {
    #[cfg(windows)]
    let text = match behavior {
        FakeBehavior::PrintAndExit(output, code) => {
            format!("@echo off\r\necho {output}\r\nexit /b {code}\r\n")
        }
        FakeBehavior::PrintStdoutStderrAndExit(stdout, stderr, code) => {
            format!("@echo off\r\necho {stdout}\r\necho {stderr} 1>&2\r\nexit /b {code}\r\n")
        }
        FakeBehavior::Sleep => "@echo off\r\nping -n 6 127.0.0.1 >nul\r\nexit /b 0\r\n".to_string(),
    };

    #[cfg(not(windows))]
    let text = match behavior {
        FakeBehavior::PrintAndExit(output, code) => {
            format!("#!/bin/sh\nprintf '%s\\n' '{output}'\nexit {code}\n")
        }
        FakeBehavior::PrintStdoutStderrAndExit(stdout, stderr, code) => {
            format!(
                "#!/bin/sh\nprintf '%s\\n' '{stdout}'\nprintf '%s\\n' '{stderr}' >&2\nexit {code}\n"
            )
        }
        FakeBehavior::Sleep => "#!/bin/sh\nsleep 5\nexit 0\n".to_string(),
    };

    fs::write(path, text)?;
    make_executable(path)?;
    Ok(())
}

pub(super) fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}
