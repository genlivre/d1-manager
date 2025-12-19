//! Version information
//!
//! This module provides version and build information for the application.

/// Current application version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// Get build information string
pub fn build_info() -> String {
    let os = if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    format!("v{} ({} {}, {})", VERSION, os, arch, profile)
}

/// Get short version string
pub fn short_version() -> String {
    format!("v{}", VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert!(info.starts_with("v"));
        assert!(info.contains("release") || info.contains("debug"));
    }
}
