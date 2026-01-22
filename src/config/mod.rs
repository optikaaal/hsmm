pub mod structs;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
pub use structs::*;

/// Read configuration from a TOML file
pub fn read_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();

    // Create default config if file doesn't exist
    if !path.exists() {
        tracing::info!(
            "Config file not found, creating default config at {}",
            path.display()
        );
        let default_config = Config::default();
        write_config(path, &default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Write configuration to a TOML file
pub fn write_config(path: impl AsRef<Path>, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

    fs::write(path, content)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    tracing::debug!("Config written to {}", path.display());
    Ok(())
}

/// Add a mod to the configuration
pub fn add_mod(config: &mut Config, name: String, identifier: ModIdentifier) -> Result<()> {
    // Check if mod already exists
    let exists = config.mods.iter().any(|m| {
        m.name.eq_ignore_ascii_case(&name)
            || match (&m.identifier, &identifier) {
                (
                    ModIdentifier::ProjectId { curseforge: a },
                    ModIdentifier::ProjectId { curseforge: b },
                ) => a == b,
                (
                    ModIdentifier::ProjectSlug { curseforge: a },
                    ModIdentifier::ProjectSlug { curseforge: b },
                ) => a.eq_ignore_ascii_case(b),
                _ => false,
            }
    });

    if exists {
        anyhow::bail!("Mod '{}' already exists in config", name);
    }

    config.mods.push(Mod {
        name,
        identifier,
        enabled: true,
    });

    Ok(())
}

/// Remove a mod from the configuration
pub fn remove_mod(config: &mut Config, name_or_id: &str) -> Result<()> {
    let original_len = config.mods.len();

    config.mods.retain(|m| {
        // Try matching by name
        if m.name.eq_ignore_ascii_case(name_or_id) {
            return false;
        }

        // Try matching by ID
        if let Ok(id) = name_or_id.parse::<i32>() {
            if let ModIdentifier::ProjectId { curseforge } = m.identifier {
                if curseforge == id {
                    return false;
                }
            }
        }

        // Try matching by slug
        if let ModIdentifier::ProjectSlug { curseforge } = &m.identifier {
            if curseforge.eq_ignore_ascii_case(name_or_id) {
                return false;
            }
        }

        true
    });

    if config.mods.len() == original_len {
        anyhow::bail!("Mod '{}' not found in config", name_or_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_mod() {
        let mut config = Config::default();
        add_mod(
            &mut config,
            "Test Mod".to_string(),
            ModIdentifier::project_id(123),
        )
        .unwrap();
        assert_eq!(config.mods.len(), 1);
        assert_eq!(config.mods[0].name, "Test Mod");
    }

    #[test]
    fn test_remove_mod() {
        let mut config = Config::default();
        config.mods.push(Mod {
            name: "Test Mod".to_string(),
            identifier: ModIdentifier::project_id(123),
            enabled: true,
        });

        remove_mod(&mut config, "Test Mod").unwrap();
        assert_eq!(config.mods.len(), 0);
    }
}
