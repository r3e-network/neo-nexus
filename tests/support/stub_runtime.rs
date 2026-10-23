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
//! on, so that is what this builds. Integration-test crates build it once per
//! checkout under `CARGO_TARGET_TMPDIR` (cargo's scratch directory inside
//! `target/`) and reuse it, so a run leaves nothing in the system temp
//! directory. In-crate unit tests get no such directory and build it once per
//! test binary into a temp directory that has to outlive the run.

// Each test binary that includes this file uses a different subset of the
// fixtures below. The rest are dead in that binary, which `-D warnings` would
// otherwise report as a build failure in every crate but the one that happens
// to use all three.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

const STUB_SOURCE: &str =
    "fn main() { std::thread::sleep(std::time::Duration::from_secs(600)); }\n";

/// A runtime that rejects whatever command line it is given and exits with a
/// reason an operator can read.
///
/// `sleep` used to play this part because it rejects the client flags a launch
/// puts on the command line. It only exists on Unix, so the failed-launch path
/// would go unproved on Windows. A compiled binary reports the same failure on
/// every platform the suite runs on.
const REJECTING_SOURCE: &str = concat!(
    "fn main() {\n",
    "    eprintln!(\"stub runtime rejected its arguments\");\n",
    "    std::process::exit(2);\n",
    "}\n"
);

/// A runtime that dies where it stands, before the launch barrier closes.
const CRASHING_SOURCE: &str = "fn main() { std::process::exit(3); }\n";

/// A runtime that survives the launch barrier and only then dies — the crash
/// the watchdog exists to notice.
const CRASHING_LATE_SOURCE: &str = concat!(
    "fn main() {\n",
    "    std::thread::sleep(std::time::Duration::from_secs(2));\n",
    "    std::process::exit(3);\n",
    "}\n"
);

/// Path to a runtime stand-in that ignores its arguments and stays running.
pub fn stub_runtime_binary() -> PathBuf {
    use std::sync::LazyLock;
    static STUB: LazyLock<PathBuf> = LazyLock::new(|| materialise(STUB_SOURCE));
    STUB.clone()
}

/// Path to a runtime stand-in that rejects its arguments and exits non-zero.
pub fn rejecting_runtime_binary() -> PathBuf {
    use std::sync::LazyLock;
    static STUB: LazyLock<PathBuf> = LazyLock::new(|| materialise(REJECTING_SOURCE));
    STUB.clone()
}

/// Path to a runtime stand-in that exits non-zero immediately.
pub fn crashing_runtime_binary() -> PathBuf {
    use std::sync::LazyLock;
    static STUB: LazyLock<PathBuf> = LazyLock::new(|| materialise(CRASHING_SOURCE));
    STUB.clone()
}

/// Path to a runtime stand-in that survives startup and exits non-zero after
/// two seconds.
pub fn crashing_late_runtime_binary() -> PathBuf {
    use std::sync::LazyLock;
    static STUB: LazyLock<PathBuf> = LazyLock::new(|| materialise(CRASHING_LATE_SOURCE));
    STUB.clone()
}

fn materialise(source: &'static str) -> PathBuf {
    match option_env!("CARGO_TARGET_TMPDIR") {
        Some(target_tmp) => cached_stub(Path::new(target_tmp), source),
        None => {
            let dir = tempfile::tempdir().expect("stub runtime directory");
            // The suite needs this for its whole run; dropping the handle
            // here would delete the binary before the first launch.
            build_stub(&dir.keep(), source)
        }
    }
}

fn stub_file_name() -> &'static str {
    if cfg!(windows) {
        "stub-runtime.exe"
    } else {
        "stub-runtime"
    }
}

/// The stub built into a directory named after its source, published with a
/// rename so a test binary running concurrently never sees a half-written one.
fn cached_stub(target_tmp: &Path, source: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    let cache = target_tmp.join(format!("stub-runtime-{:016x}", hasher.finish()));
    let binary = cache.join(stub_file_name());
    if binary.is_file() {
        return binary;
    }

    std::fs::create_dir_all(target_tmp).expect("create CARGO_TARGET_TMPDIR");
    let staging = tempfile::Builder::new()
        .prefix("stub-runtime-staging-")
        .tempdir_in(target_tmp)
        .expect("stub runtime staging directory");
    build_stub(staging.path(), source);
    // Losing the race to another test binary is fine: it published the same
    // stub, and `staging` is removed when it drops.
    let _ = std::fs::rename(staging.path(), &cache);
    assert!(
        binary.is_file(),
        "the stub runtime was not published at {}",
        binary.display()
    );
    binary
}

fn build_stub(root: &Path, source: &str) -> PathBuf {
    let source_path = root.join("stub_runtime.rs");
    std::fs::write(&source_path, source).expect("write stub runtime source");
    let binary = root.join(stub_file_name());
    let compiled = std::process::Command::new("rustc")
        .arg("-O")
        .arg("--edition=2021")
        .arg("-o")
        .arg(&binary)
        .arg(&source_path)
        .output()
        .expect("run rustc to build the stub runtime");
    assert!(
        compiled.status.success(),
        "failed to build the stub runtime: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    binary
}
