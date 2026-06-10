use anahtar_core::{
    add_entry_save_as, delete_entry_save_as, edit_entry_save_as, inspect_header, list_entries,
    open_database, search_entries, show_entry, upgrade_to_kdbx41, AddEntryRequest,
    EditEntryRequest, EntryDetail, EntrySummary, SaveAsOptions, UpgradeReport, VaultInfo,
    WriteReport,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{io::Write, path::PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "anahtar",
    version,
    about = "KeePass KDBX CLI for personal vault workflows"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect KDBX header metadata without unlocking the database.
    Inspect {
        vault: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// List entries without printing passwords.
    List {
        vault: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Search entries without printing passwords.
    Search {
        vault: PathBuf,
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Show one entry. Password is hidden unless --reveal-password is used.
    Show {
        vault: PathBuf,
        selector: String,
        #[arg(long)]
        reveal_password: bool,
        #[arg(long)]
        json: bool,
    },
    /// Save a vault as KDBX 4.1 without modifying the input file.
    Upgrade {
        input: PathBuf,
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
        input: PathBuf,
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
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Edit explicitly provided entry fields and write a new KDBX 4.1 output file.
    Edit {
        input: PathBuf,
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
        input: PathBuf,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { vault, json } => {
            let info = inspect_header(vault)?;
            print_vault_info(&info, json)?;
        }
        Command::List { vault, json } => {
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let entries = list_entries(&db);
            print_entries(&entries, json)?;
        }
        Command::Search { vault, query, json } => {
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let entries = search_entries(&db, &query);
            print_entries(&entries, json)?;
        }
        Command::Show {
            vault,
            selector,
            reveal_password,
            json,
        } => {
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let detail = show_entry(&db, &selector, reveal_password)?;
            print_detail(&detail, json)?;
        }
        Command::Upgrade {
            input,
            output,
            force,
            dry_run,
            json,
        } => {
            if !dry_run {
                preflight_output(&output, force)?;
            }
            let password = prompt_password()?;
            let report = upgrade_to_kdbx41(input, output, &password, force, dry_run)?;
            print_upgrade_report(&report, json)?;
        }
        Command::Add {
            input,
            output,
            group,
            title,
            username,
            url,
            notes,
            password_prompt,
            no_password,
            force,
            json,
        } => {
            preflight_output(&output, force)?;
            validate_add_password_mode(password_prompt, no_password)?;
            let password = prompt_password()?;
            let entry_password = resolve_entry_password(password_prompt, no_password)?;
            let report = add_entry_save_as(
                input,
                &password,
                SaveAsOptions {
                    output_path: output,
                    force,
                },
                AddEntryRequest {
                    group_path: group,
                    title,
                    username,
                    password: entry_password,
                    url,
                    notes,
                },
            )?;
            print_write_report(&report, json)?;
        }
        Command::Edit {
            input,
            selector,
            output,
            title,
            username,
            url,
            notes,
            password_prompt,
            force,
            json,
        } => {
            preflight_output(&output, force)?;
            let password = prompt_password()?;
            let entry_password = if password_prompt {
                Some(prompt_entry_password_with_confirmation()?)
            } else {
                None
            };
            let report = edit_entry_save_as(
                input,
                &selector,
                &password,
                SaveAsOptions {
                    output_path: output,
                    force,
                },
                EditEntryRequest {
                    title,
                    username,
                    password: entry_password,
                    url,
                    notes,
                },
            )?;
            print_write_report(&report, json)?;
        }
        Command::Delete {
            input,
            entry_id,
            output,
            yes,
            force,
            json,
        } => {
            validate_uuid_selector(&entry_id)?;
            preflight_output(&output, force)?;
            let password = prompt_password()?;
            let db = open_database(&input, &password)?;
            let detail = show_entry(&db, &entry_id, false)?;
            if !yes {
                confirm_delete(&detail)?;
            }
            let report = delete_entry_save_as(
                input,
                &entry_id,
                &password,
                SaveAsOptions {
                    output_path: output,
                    force,
                },
            )?;
            print_write_report(&report, json)?;
        }
    }
    Ok(())
}

fn prompt_password() -> Result<String> {
    Ok(rpassword::prompt_password("KDBX master password: ")?)
}

fn preflight_output(output: &PathBuf, force: bool) -> Result<()> {
    if output.exists() && !force {
        anyhow::bail!("output already exists: {}", output.display());
    }
    Ok(())
}

fn validate_add_password_mode(password_prompt: bool, no_password: bool) -> Result<()> {
    match (password_prompt, no_password) {
        (true, false) | (false, true) => Ok(()),
        _ => anyhow::bail!("use exactly one of --password-prompt or --no-password"),
    }
}

fn resolve_entry_password(password_prompt: bool, no_password: bool) -> Result<Option<String>> {
    validate_add_password_mode(password_prompt, no_password)?;
    if no_password {
        Ok(None)
    } else {
        Ok(Some(prompt_entry_password_with_confirmation()?))
    }
}

fn validate_uuid_selector(selector: &str) -> Result<()> {
    uuid::Uuid::parse_str(selector)
        .map(|_| ())
        .map_err(|_| anyhow::anyhow!("delete requires an entry UUID selector"))
}

fn prompt_entry_password_with_confirmation() -> Result<String> {
    let password = rpassword::prompt_password("Entry password: ")?;
    let confirm = rpassword::prompt_password("Confirm entry password: ")?;
    if password != confirm {
        anyhow::bail!("entry password confirmation did not match");
    }
    Ok(password)
}

fn confirm_delete(detail: &EntryDetail) -> Result<()> {
    println!("Delete entry?");
    println!("ID: {}", detail.id);
    println!("Title: {}", detail.title.as_deref().unwrap_or(""));
    println!("Username: {}", detail.username.as_deref().unwrap_or(""));
    println!("URL: {}", detail.url.as_deref().unwrap_or(""));
    print!("Type DELETE to confirm: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "DELETE" {
        anyhow::bail!("delete confirmation failed");
    }
    Ok(())
}

fn print_vault_info(info: &VaultInfo, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(info)?);
    } else {
        println!("Path: {}", info.path.display());
        println!("Size: {} bytes", info.file_size_bytes);
        println!("Format: {}", info.version);
    }
    Ok(())
}

fn print_entries(entries: &[EntrySummary], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(entries)?);
    } else {
        for entry in entries {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                entry.id,
                entry.group_path,
                entry.title.as_deref().unwrap_or(""),
                entry.username.as_deref().unwrap_or(""),
                entry.url.as_deref().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn print_write_report(report: &WriteReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!("Operation: {:?}", report.operation);
        println!("Input: {}", report.input_path.display());
        println!("Output: {}", report.output_path.display());
        println!("Input format: {}", report.input_version);
        println!("Output format: {}", report.output_version);
        println!(
            "Counts: groups {} -> {}, entries {} -> {}",
            report.input_group_count,
            report.output_group_count,
            report.input_entry_count,
            report.output_entry_count
        );
        if let Some(id) = &report.changed_entry_id {
            println!("Changed entry id: {id}");
        }
        println!("Write complete. Original input was not modified.");
        println!("Next: open the output file in Strongbox and manually verify it before using it as a primary vault.");
    }
    Ok(())
}

fn print_upgrade_report(report: &UpgradeReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        if let Some(warning) = &report.warning {
            println!("Warning: {warning}");
        }
        println!("Input: {}", report.input_path.display());
        println!("Output: {}", report.output_path.display());
        println!("Input format: {}", report.input_version);
        println!("Output format: {}", report.output_version);
        println!(
            "Input counts: groups={}, entries={}",
            report.input_group_count, report.input_entry_count
        );
        if report.dry_run {
            println!("Dry run: no file was written.");
        } else {
            println!(
                "Output counts: groups={}, entries={}",
                report.output_group_count.unwrap_or_default(),
                report.output_entry_count.unwrap_or_default()
            );
            println!("Upgrade complete. Original input was not modified.");
            println!("Next: open the output file in Strongbox and manually verify important entries before using it as a primary vault.");
        }
    }
    Ok(())
}

fn print_detail(detail: &EntryDetail, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(detail)?);
    } else {
        println!("ID: {}", detail.id);
        println!("Group: {}", detail.group_path);
        println!("Title: {}", detail.title.as_deref().unwrap_or(""));
        println!("Username: {}", detail.username.as_deref().unwrap_or(""));
        println!("URL: {}", detail.url.as_deref().unwrap_or(""));
        println!("Notes: {}", detail.notes.as_deref().unwrap_or(""));
        match &detail.password {
            Some(password) => println!("Password: {password}"),
            None => println!("Password: <hidden; pass --reveal-password to display>"),
        }
        if !detail.custom_fields.is_empty() {
            println!("Custom fields:");
            for field in &detail.custom_fields {
                let marker = if field.protected {
                    "protected"
                } else {
                    "plain"
                };
                println!("  {} ({marker}): {}", field.key, field.value);
            }
        }
    }
    Ok(())
}
