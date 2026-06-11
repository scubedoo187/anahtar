use crate::config::{load_config, validate_clear_after};
use anyhow::Result;
use std::{thread, time::Duration};

pub fn copy_with_clear(value: &str, clear_after: Option<u64>) -> Result<()> {
    let config = load_config()?;
    let seconds = clear_after.unwrap_or(config.clipboard_clear_after_seconds);
    validate_clear_after(seconds)?;
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(value.to_string())?;
    println!("Copied to clipboard. Clearing in {seconds} seconds...");
    thread::sleep(Duration::from_secs(seconds));
    let mut clipboard = arboard::Clipboard::new()?;
    if clipboard.get_text().ok().as_deref() == Some(value) {
        clipboard.set_text(String::new())?;
        println!("Clipboard cleared.");
    } else {
        println!("Clipboard changed; not clearing.");
    }
    Ok(())
}
