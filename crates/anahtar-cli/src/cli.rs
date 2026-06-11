use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
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

#[derive(Debug, Clone, Args)]
pub struct EntrySelectorArgs {
    /// Backward-compatible selector shorthand: UUID or exact title.
    pub selector: Option<String>,
    #[arg(long, conflicts_with_all = ["selector", "title", "url", "username"])]
    pub id: Option<String>,
    #[arg(long, conflicts_with_all = ["selector", "id", "url", "username"])]
    pub title: Option<String>,
    #[arg(long, conflicts_with_all = ["selector", "id", "title", "username"])]
    pub url: Option<String>,
    #[arg(long, conflicts_with_all = ["selector", "id", "title", "url"])]
    pub username: Option<String>,
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
        key_file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Search entries without printing passwords.
    Search {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Show one entry. Password is hidden unless --reveal-password is used.
    Show {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
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
        key_file: Option<PathBuf>,
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
        key_file: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
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
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Edit explicitly provided entry fields and write a new KDBX 4.1 output file.
    Edit {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(id = "set-title", long = "set-title")]
        title: Option<String>,
        #[arg(id = "set-username", long = "set-username")]
        username: Option<String>,
        #[arg(id = "set-url", long = "set-url")]
        url: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        password_prompt: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Delete an entry by UUID and write a new KDBX 4.1 output file.
    Delete {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Manage Anahtar configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Manage groups.
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    /// Move an entry to a group.
    Move {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        group: String,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Generate shell completions.
    Completions { shell: Shell },
    /// Audit vault health without printing secrets.
    Audit {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Copy an entry password to the clipboard without printing it.
    CopyPassword {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Copy an entry username to the clipboard.
    CopyUsername {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        clear_after: Option<u64>,
    },
    /// Copy an entry URL to the clipboard.
    CopyUrl {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[command(flatten)]
        selector: EntrySelectorArgs,
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
        key_file: Option<PathBuf>,
        #[arg(long)]
        copy: bool,
        #[command(flatten)]
        selector: EntrySelectorArgs,
        #[arg(long)]
        clear_after: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum GroupCommand {
    List {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Add {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        path: String,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    Rename {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        path: String,
        new_name: String,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    Delete {
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        path: String,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
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
    KeyFile { path: PathBuf },
    BackupDir { path: PathBuf },
    GeneratorLength { n: usize },
    ClipboardClearAfter { seconds: u64 },
}
