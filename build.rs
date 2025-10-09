use std::time::SystemTime;
use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    set_build_version();
}

/// Gets the Git commit hash and dirty status.
/// Returns `None` if Git is unavailable.
fn get_git_version() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short=10", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        })
}

/// Sets `BUILD_VERSION`, `GIT_VERSION` and `BUILD_TIMESTAMP`
fn set_build_version() {
    let build_version = format!("v{}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=BUILD_VERSION={}", build_version);

    let git_version = get_git_version().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);

    let build_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
}
