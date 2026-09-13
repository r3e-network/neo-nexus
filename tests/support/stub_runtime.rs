//! A stand-in node runtime for lifecycle tests.
//!
//! Included by both the in-crate unit tests and the integration suites through
//! `#[path]`, because they are separate compilation units and this fixture has
//! to behave identically in each.
//!
//! Two properties make a usable stand-in, and the obvious candidates have
//! neither:
//!
//! 1. **It must ignore its arguments.** `LaunchPlanner` puts each client's own
//!    flags on the command line — `node` and `--config-file` for NeoGo,
//!    `--config` for NeoRs, `--background` for neo-cli. `sleep` rejects those
//!    and exits, which the launch barrier then correctly reports as a failed
//!    start.
//!
//! 2. **The OS must report it as the executable NeoNexus registered.** A node's
//!    process is identified by comparing `Process::exe()` against its binary
//!    path, and neither shell spelling survives that. A `#!` script is run by
//!    its interpreter, so the kernel reports `/bin/sh`. `sh -c "one command"`
//!    execs that command in place, so the kernel reports `/bin/sleep`. On macOS
//!    `/bin/sh` itself reports as `/bin/bash`. Any of these reads as a recycled
//!    pid, and the node can then never be stopped or restarted.
//!
//! A small compiled binary has both properties on every platform the suite runs
//! on, so that is what this builds — once per test binary, on first use.

use std::path::PathBuf;

/// Path to a runtime stand-in that ignores its arguments and stays running.
pub fn stub_runtime_binary() -> PathBuf {
    use std::sync::LazyLock;
    static STUB: LazyLock<PathBuf> = LazyLock::new(|| {
        let dir = tempfile::tempdir().expect("stub runtime directory");
        // The suite needs this for its whole run; dropping the handle here
        // would delete the binary before the first launch.
        let root = dir.keep();
        let source = root.join("stub_runtime.rs");
        std::fs::write(
            &source,
            "fn main() { std::thread::sleep(std::time::Duration::from_secs(600)); }\n",
        )
        .expect("write stub runtime source");
        let binary = root.join(if cfg!(windows) {
            "stub-runtime.exe"
        } else {
            "stub-runtime"
        });
        let compiled = std::process::Command::new("rustc")
            .arg("-O")
            .arg("--edition=2021")
            .arg("-o")
            .arg(&binary)
            .arg(&source)
            .output()
            .expect("run rustc to build the stub runtime");
        assert!(
            compiled.status.success(),
            "failed to build the stub runtime: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        binary
    });
    STUB.clone()
}
