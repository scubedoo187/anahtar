use crate::config::load_config;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn resolve_vault(cli_vault: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(vault) = cli_vault {
        return Ok(vault);
    }
    if let Some(vault) = load_config()?.vault {
        return Ok(vault);
    }
    anyhow::bail!("No vault provided and no default vault configured. Run: anahtar config set vault /path/to/vault.kdbx")
}

pub fn preflight_output(output: &Path, force: bool) -> Result<()> {
    if output.exists() && !force {
        anyhow::bail!("output already exists: {}", output.display());
    }
    Ok(())
}

pub fn ensure_edit_has_change(
    title: &Option<String>,
    username: &Option<String>,
    url: &Option<String>,
    notes: &Option<String>,
    password_prompt: bool,
) -> Result<()> {
    if title.is_none() && username.is_none() && url.is_none() && notes.is_none() && !password_prompt
    {
        anyhow::bail!("edit requires at least one field option or --password-prompt");
    }
    Ok(())
}
