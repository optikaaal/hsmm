use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// CurseForge game ID for Hytale (auto-detected on first run)
    #[serde(default)]
    pub game_id: Option<i32>,

    /// List of mods to track and update
    #[serde(default)]
    pub mods: Vec<Mod>,

    /// General settings
    #[serde(default)]
    pub settings: Settings,
}

/// Individual mod configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    /// Display name for the mod
    pub name: String,

    /// CurseForge identifier (project ID or slug)
    pub identifier: ModIdentifier,

    /// Whether this mod is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// The actual filename of the installed mod file (if downloaded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_file: Option<String>,
}

fn default_true() -> bool {
    true
}

/// CurseForge mod identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModIdentifier {
    /// Project ID (numeric)
    ProjectId { curseforge: i32 },

    /// Project slug (string)
    ProjectSlug { curseforge: String },
}

impl ModIdentifier {
    pub fn project_id(id: i32) -> Self {
        Self::ProjectId { curseforge: id }
    }

    pub fn project_slug(slug: impl Into<String>) -> Self {
        Self::ProjectSlug {
            curseforge: slug.into(),
        }
    }
}

/// Global settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Remove old mod versions after upgrade (default: true)
    #[serde(default = "default_true")]
    pub cleanup_old_versions: bool,

    /// Archive old versions to .old/ instead of deleting (default: true)
    /// If false, old versions are permanently deleted
    #[serde(default = "default_true")]
    pub archive_old_versions: bool,

    /// Maximum number of old versions to keep in .old/ (only if archive_old_versions = true)
    #[serde(default = "default_max_old_versions")]
    pub max_old_versions: usize,

    /// Download timeout in seconds
    #[serde(default = "default_timeout")]
    pub download_timeout_secs: u64,

    /// Parallel downloads
    #[serde(default = "default_parallel")]
    pub parallel_downloads: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cleanup_old_versions: true,
            archive_old_versions: true,
            max_old_versions: 3,
            download_timeout_secs: 300,
            parallel_downloads: 4,
        }
    }
}

fn default_max_old_versions() -> usize {
    3
}

fn default_timeout() -> u64 {
    300
}

fn default_parallel() -> usize {
    4
}

/// Metadata about a mod file
#[derive(Debug, Clone)]
pub struct ModMetadata {
    pub project_id: i32,
    pub file_id: i32,
    pub display_name: String,
    pub file_name: String,
    pub download_url: String,
    pub file_length: u64,
    pub file_date: chrono::DateTime<chrono::Utc>,
}
