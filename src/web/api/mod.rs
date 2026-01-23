pub mod backups;
pub mod config;
pub mod curseforge;
pub mod health;
pub mod logs;
pub mod mods;
pub mod server;

use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub config_path: PathBuf,
    pub mods_dir: PathBuf,
    pub server_files_dir: PathBuf,
}
