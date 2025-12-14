//! Version management and update checking
//!
//! This module provides:
//! - Version information display
//! - GitHub Release-based update checking
//! - Download links for new versions

use serde::Deserialize;

/// Current application version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// GitHub repository for update checks (owner/repo format)
pub const GITHUB_REPO: &str = "example/d1-manager";

/// Parsed version for comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    /// Parse version string like "1.2.3" or "v1.2.3"
    pub fn parse(version: &str) -> Option<Self> {
        let version = version.trim().trim_start_matches('v');
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() >= 3 {
            Some(Self {
                major: parts[0].parse().ok()?,
                minor: parts[1].parse().ok()?,
                patch: parts[2].split('-').next()?.parse().ok()?,
            })
        } else if parts.len() == 2 {
            Some(Self {
                major: parts[0].parse().ok()?,
                minor: parts[1].parse().ok()?,
                patch: 0,
            })
        } else {
            None
        }
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &Self) -> bool {
        if self.major != other.major {
            return self.major > other.major;
        }
        if self.minor != other.minor {
            return self.minor > other.minor;
        }
        self.patch > other.patch
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// GitHub Release API response
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: String,
    #[serde(default)]
    pub assets: Vec<GitHubAsset>,
}

/// GitHub Release Asset
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub is_update_available: bool,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

impl UpdateInfo {
    /// Create from current version when no update is available
    pub fn up_to_date() -> Self {
        Self {
            current_version: VERSION.to_string(),
            latest_version: VERSION.to_string(),
            is_update_available: false,
            release_url: String::new(),
            release_notes: None,
            download_url: None,
        }
    }
}

/// Check for updates from GitHub Releases
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent(format!("{}/{}", APP_NAME, VERSION))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if response.status() == 404 {
        // No releases yet
        return Ok(UpdateInfo::up_to_date());
    }

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    // Skip prereleases and drafts
    if release.prerelease || release.draft {
        return Ok(UpdateInfo::up_to_date());
    }

    let current = SemanticVersion::parse(VERSION);
    let latest = SemanticVersion::parse(&release.tag_name);

    let is_update_available = match (&current, &latest) {
        (Some(curr), Some(lat)) => lat.is_newer_than(curr),
        _ => false,
    };

    // Find appropriate download URL based on platform
    let download_url = find_platform_asset(&release.assets);

    Ok(UpdateInfo {
        current_version: VERSION.to_string(),
        latest_version: release.tag_name.trim_start_matches('v').to_string(),
        is_update_available,
        release_url: release.html_url,
        release_notes: release.body,
        download_url,
    })
}

/// Find the download URL for current platform
fn find_platform_asset(assets: &[GitHubAsset]) -> Option<String> {
    let platform_patterns = if cfg!(target_os = "macos") {
        vec!["macos", "darwin", "osx", ".dmg", ".app.zip"]
    } else if cfg!(target_os = "windows") {
        vec!["windows", "win64", "win32", ".exe", ".msi"]
    } else {
        vec!["linux", ".AppImage", ".deb", ".rpm", ".tar.gz"]
    };

    // Also check architecture
    let arch = if cfg!(target_arch = "x86_64") {
        vec!["x86_64", "x64", "amd64"]
    } else if cfg!(target_arch = "aarch64") {
        vec!["aarch64", "arm64"]
    } else {
        vec![]
    };

    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        // Check platform match
        let platform_match = platform_patterns.iter().any(|p| name_lower.contains(p));

        // Check arch match (or no arch specified)
        let arch_match = arch.is_empty() || arch.iter().any(|a| name_lower.contains(a));

        if platform_match && arch_match {
            return Some(asset.browser_download_url.clone());
        }
    }

    // Fallback: just find any matching platform
    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if platform_patterns.iter().any(|p| name_lower.contains(p)) {
            return Some(asset.browser_download_url.clone());
        }
    }

    None
}

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
    fn test_parse_version() {
        let v = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = SemanticVersion::parse("v2.0.0").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);

        let v = SemanticVersion::parse("1.0.0-beta").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SemanticVersion::parse("1.0.0").unwrap();
        let v2 = SemanticVersion::parse("1.0.1").unwrap();
        let v3 = SemanticVersion::parse("1.1.0").unwrap();
        let v4 = SemanticVersion::parse("2.0.0").unwrap();

        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(v4.is_newer_than(&v3));
        assert!(!v1.is_newer_than(&v2));
        assert!(!v1.is_newer_than(&v1));
    }

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert!(info.starts_with("v"));
        assert!(info.contains("release") || info.contains("debug"));
    }
}
