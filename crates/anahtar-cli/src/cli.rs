use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "anahtar",
    version,
    about = "KeePass KDBX CLI for personal vault workflows"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect KDBX header metadata without unlocking the database.
    Inspect {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// List entries without printing passwords.
    List {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Search entries without printing passwords.
    Search {
        #[arg(long)]
        vault: Option<PathBuf>,
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Show one entry. Password is hidden unless --reveal-password is used.
    Show {
        #[arg(long)]
        vault: Option<PathBuf>,
        selector: String,
        #[arg(long)]
        reveal_password: bool,
        #[arg(long)]
        json: bool,
    },
    /// Save a vault as KDBX 4.1 without modifying the input file.
    Upgrade {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add an entry and write a new KDBX 4.1 output file.
    Add {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        group: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        password_prompt: bool,
        #[arg(long)]
        no_password: bool,
        #[arg(long)]
        generate_password: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Edit explicitly provided entry fields and write a new KDBX 4.1 output file.
    Edit {
        #[arg(long)]
        vault: Option<PathBuf>,
        selector: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        password_prompt: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Delete an entry by UUID and write a new KDBX 4.1 output file.
    Delete {
        #[arg(long)]
        vault: Option<PathBuf>,
        entry_id: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Manage Anahtar configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Copy an entry password to the clipboard without printing it.
    CopyPassword {
        #[arg(long)]
        vault: Option<PathBuf>,
        selector: String,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Copy an entry username to the clipboard.
    CopyUsername {
        #[arg(long)]
        vault: Option<PathBuf>,
        selector: String,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Copy an entry URL to the clipboard.
    CopyUrl {
        #[arg(long)]
        vault: Option<PathBuf>,
        selector: String,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Generate a secure random password.
    Generate {
        #[arg(long)]
        length: Option<usize>,
        #[arg(long)]
        copy: bool,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Display or copy a TOTP code without exposing the OTP URI.
    Totp {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        copy: bool,
        selector: String,
        #[arg(long)]
        clear_after: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show,
    Get {
        key: String,
    },
    Set {
        #[command(subcommand)]
        command: ConfigSetCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigSetCommand {
    Vault { path: PathBuf },
    GeneratorLength { n: usize },
    ClipboardClearAfter { seconds: u64 },
}
