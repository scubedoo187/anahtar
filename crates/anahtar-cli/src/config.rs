use crate::cli::{ConfigCommand, ConfigSetCommand};
use anyhow::{Context, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub const DEFAULT_GENERATOR_LENGTH: usize = 32;
pub const DEFAULT_CLIPBOARD_CLEAR_SECONDS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnahtarConfig {
    pub vault: Option<PathBuf>,
    pub generator_length: usize,
    pub clipboard_clear_after_seconds: u64,
}

impl Default for AnahtarConfig {
    fn default() -> Self {
        Self {
            vault: None,
            generator_length: DEFAULT_GENERATOR_LENGTH,
            clipboard_clear_after_seconds: DEFAULT_CLIPBOARD_CLEAR_SECONDS,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("could not determine platform config directory")?;
    Ok(base_dirs.config_dir().join("anahtar").join("config.toml"))
}

pub fn load_config() -> Result<AnahtarConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AnahtarConfig::default());
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: AnahtarConfig =
        toml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    validate_config(&config)?;
    Ok(config)
}

pub fn save_config(config: &AnahtarConfig) -> Result<()> {
    validate_config(config)?;
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn validate_config(config: &AnahtarConfig) -> Result<()> {
    validate_generator_length(config.generator_length)?;
    validate_clear_after(config.clipboard_clear_after_seconds)?;
    Ok(())
}

pub fn validate_generator_length(n: usize) -> Result<()> {
    if !(8..=256).contains(&n) {
        anyhow::bail!("generator length must be between 8 and 256");
    }
    Ok(())
}

pub fn validate_clear_after(seconds: u64) -> Result<()> {
    if seconds > 3600 {
        anyhow::bail!("clipboard clear timeout must be <= 3600 seconds");
    }
    Ok(())
}

pub fn handle_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Show => {
            let config = load_config()?;
            println!("{}", toml::to_string_pretty(&config)?);
        }
        ConfigCommand::Get { key } => {
            let config = load_config()?;
            match key.as_str() {
                "vault" => println!(
                    "{}",
                    config
                        .vault
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                "generator-length" | "generator_length" => println!("{}", config.generator_length),
                "clipboard-clear-after" | "clipboard_clear_after_seconds" => {
                    println!("{}", config.clipboard_clear_after_seconds)
                }
                _ => anyhow::bail!("unknown config key: {key}"),
            }
        }
        ConfigCommand::Set { command } => {
            let mut config = load_config()?;
            match command {
                ConfigSetCommand::Vault { path } => {
                    config.vault = Some(canonicalize_vault_path(path)?)
                }
                ConfigSetCommand::GeneratorLength { n } => {
                    validate_generator_length(n)?;
                    config.generator_length = n;
                }
                ConfigSetCommand::ClipboardClearAfter { seconds } => {
                    validate_clear_after(seconds)?;
                    config.clipboard_clear_after_seconds = seconds;
                }
            }
            save_config(&config)?;
            println!("Config saved: {}", config_path()?.display());
        }
    }
    Ok(())
}

fn canonicalize_vault_path(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        anyhow::bail!("vault path does not exist: {}", path.display());
    }
    if !path.is_file() {
        anyhow::bail!("vault path is not a file: {}", path.display());
    }
    path.canonicalize()
        .with_context(|| format!("canonicalize vault path {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::canonicalize_vault_path;
    use std::fs;

    #[test]
    fn canonicalize_vault_path_requires_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.kdbx");
        let err = canonicalize_vault_path(missing).unwrap_err().to_string();
        assert!(err.contains("vault path does not exist"));

        let err = canonicalize_vault_path(dir.path().to_path_buf())
            .unwrap_err()
            .to_string();
        assert!(err.contains("vault path is not a file"));
    }

    #[test]
    fn canonicalize_vault_path_returns_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.kdbx");
        fs::write(&vault, b"not a real vault").unwrap();

        let canonical = canonicalize_vault_path(vault).unwrap();
        assert!(canonical.is_absolute());
        assert!(canonical.ends_with("vault.kdbx"));
    }
}
