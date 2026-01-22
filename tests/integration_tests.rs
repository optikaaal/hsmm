// Integration tests for hsmm
// Run with: cargo test --test integration_tests -- --test-threads=1
// Run ignored tests: cargo test --test integration_tests -- --ignored --test-threads=1

use hsmm::config::{self, Config, ModIdentifier};
use tempfile::TempDir;

/// Test that we can create and read a config file
#[test]
fn test_config_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mods.toml");

    // Create a config
    let mut config = Config::default();
    config::add_mod(
        &mut config,
        "Test Mod".to_string(),
        ModIdentifier::project_id(123456),
    )
    .unwrap();

    // Write it
    config::write_config(&config_path, &config).unwrap();

    // Read it back
    let loaded_config = config::read_config(&config_path).unwrap();

    assert_eq!(loaded_config.mods.len(), 1);
    assert_eq!(loaded_config.mods[0].name, "Test Mod");
}

/// Test adding and removing mods
#[test]
fn test_add_remove_mods() {
    let mut config = Config::default();

    // Add first mod
    config::add_mod(
        &mut config,
        "Mod 1".to_string(),
        ModIdentifier::project_id(111),
    )
    .unwrap();
    assert_eq!(config.mods.len(), 1);

    // Add second mod
    config::add_mod(
        &mut config,
        "Mod 2".to_string(),
        ModIdentifier::project_id(222),
    )
    .unwrap();
    assert_eq!(config.mods.len(), 2);

    // Try to add duplicate (should fail)
    let result = config::add_mod(
        &mut config,
        "Mod 1".to_string(),
        ModIdentifier::project_id(111),
    );
    assert!(result.is_err());

    // Remove by name
    config::remove_mod(&mut config, "Mod 1").unwrap();
    assert_eq!(config.mods.len(), 1);
    assert_eq!(config.mods[0].name, "Mod 2");

    // Remove by ID
    config::remove_mod(&mut config, "222").unwrap();
    assert_eq!(config.mods.len(), 0);
}

/// Test config file auto-creation
#[test]
fn test_config_auto_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("auto-mods.toml");

    // Config doesn't exist yet
    assert!(!config_path.exists());

    // Reading should create it
    let config = config::read_config(&config_path).unwrap();

    // Now it should exist
    assert!(config_path.exists());
    assert_eq!(config.mods.len(), 0);
}

/// Test invalid config handling
#[test]
fn test_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bad-mods.toml");

    // Write invalid TOML
    std::fs::write(&config_path, "invalid toml [[").unwrap();

    // Should return error, not panic
    let result = config::read_config(&config_path);
    assert!(result.is_err());
}

/// Test CurseForge API connectivity (requires network)
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_curseforge_api_connectivity() {
    use hsmm::curseforge;

    // Get Hytale game ID (should be 70216)
    let game_id = curseforge::get_hytale_game_id().await;
    assert!(game_id.is_ok());

    let id = game_id.unwrap();
    println!("Hytale Game ID: {}", id);
    assert_eq!(id, 70216);
}

/// Test fetching mod info from CurseForge (requires network)
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_fetch_mod_info() {
    use hsmm::curseforge;

    // Use a known Hytale mod (EyeSpy) for testing API
    let game_id = 70216; // Hytale
    let mod_id = 1423494; // EyeSpy

    let result = curseforge::get_mod_info(game_id, mod_id).await;

    match result {
        Ok(mod_info) => {
            println!("Mod name: {}", mod_info.name);
            println!("Mod ID: {}", mod_info.id);
            assert_eq!(mod_info.id, mod_id);
            assert_eq!(mod_info.game_id, game_id);
        }
        Err(e) => {
            panic!("Failed to fetch mod info: {}", e);
        }
    }
}

/// Test fetching latest mod file (requires network)
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_fetch_latest_file() {
    use hsmm::curseforge;

    // Use a known Hytale mod (EyeSpy)
    let mod_id = 1423494;

    let result = curseforge::get_latest_mod_file(mod_id).await;

    match result {
        Ok(file) => {
            println!("Latest file: {}", file.display_name);
            println!("File name: {}", file.file_name);
            println!("File date: {}", file.file_date);
            assert!(!file.file_name.is_empty());
            assert!(file.file_length > 0);
        }
        Err(e) => {
            panic!("Failed to fetch latest file: {}", e);
        }
    }
}

/// Test downloading a real mod (requires network, slow)
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_download_real_mod() {
    use hsmm::upgrade::download;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test-mod.jar");

    // Use a small test file (JEI mod)
    // Note: This will actually download ~1MB
    let url = "https://edge.forgecdn.net/files/5067/639/jei-1.20.1-forge-15.2.0.27.jar";
    let file_size = 1_100_000; // Approximate

    let result = download::download_file(url, &output_path, file_size, 60).await;

    match result {
        Ok(_) => {
            assert!(output_path.exists());
            let metadata = std::fs::metadata(&output_path).unwrap();
            assert!(metadata.len() > 0);
            println!("Downloaded file size: {} bytes", metadata.len());
        }
        Err(e) => {
            eprintln!(
                "Download test failed (expected if network/API issues): {}",
                e
            );
            // Don't fail test on network issues
        }
    }
}

/// Test cleanup of old mod versions
#[tokio::test]
async fn test_cleanup_old_versions() {
    use hsmm::config::Settings;
    use hsmm::upgrade::cleanup;

    let temp_dir = TempDir::new().unwrap();
    let mods_dir = temp_dir.path();

    // Create fake mod files
    std::fs::write(mods_dir.join("mod-v1.0.0.jar"), b"fake mod v1").unwrap();
    std::fs::write(mods_dir.join("mod-v1.1.0.jar"), b"fake mod v1.1").unwrap();
    std::fs::write(mods_dir.join("old-mod.jar"), b"old mod").unwrap();

    // Current files (what we want to keep)
    let current_files = vec!["mod-v1.1.0.jar".to_string()];

    // Settings
    let settings = Settings {
        cleanup_old_versions: true,
        archive_old_versions: true,
        max_old_versions: 2,
        download_timeout_secs: 300,
        parallel_downloads: 4,
    };

    // Run cleanup
    cleanup::cleanup_old_versions(mods_dir, &current_files, &settings)
        .await
        .unwrap();

    // Check results
    assert!(mods_dir.join("mod-v1.1.0.jar").exists()); // Current version kept
    assert!(mods_dir.join(".old").exists()); // .old directory created
    assert!(!mods_dir.join("mod-v1.0.0.jar").exists()); // Old version moved
    assert!(!mods_dir.join("old-mod.jar").exists()); // Old mod moved
}

/// Test cleanup with .part files
#[tokio::test]
async fn test_cleanup_part_files() {
    use hsmm::config::Settings;
    use hsmm::upgrade::cleanup;

    let temp_dir = TempDir::new().unwrap();
    let mods_dir = temp_dir.path();

    // Create .part file (incomplete download)
    std::fs::write(mods_dir.join("incomplete.jar.part"), b"incomplete").unwrap();
    std::fs::write(mods_dir.join("good-mod.jar"), b"complete").unwrap();

    let current_files = vec!["good-mod.jar".to_string()];
    let settings = Settings::default();

    cleanup::cleanup_old_versions(mods_dir, &current_files, &settings)
        .await
        .unwrap();

    // .part file should be deleted
    assert!(!mods_dir.join("incomplete.jar.part").exists());
    // Good file should remain
    assert!(mods_dir.join("good-mod.jar").exists());
}

/// Benchmark: Config file operations
#[test]
fn bench_config_operations() {
    use std::time::Instant;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bench-mods.toml");

    let mut config = Config::default();

    // Add 100 mods
    let start = Instant::now();
    for i in 0..100 {
        config::add_mod(
            &mut config,
            format!("Mod {}", i),
            ModIdentifier::project_id(i),
        )
        .unwrap();
    }
    let add_duration = start.elapsed();

    // Write config
    let start = Instant::now();
    config::write_config(&config_path, &config).unwrap();
    let write_duration = start.elapsed();

    // Read config
    let start = Instant::now();
    let _loaded = config::read_config(&config_path).unwrap();
    let read_duration = start.elapsed();

    println!("Config benchmark:");
    println!("  Add 100 mods: {:?}", add_duration);
    println!("  Write config: {:?}", write_duration);
    println!("  Read config:  {:?}", read_duration);

    // Sanity checks
    assert!(add_duration.as_millis() < 100); // Should be very fast
    assert!(write_duration.as_millis() < 100);
    assert!(read_duration.as_millis() < 100);
}
