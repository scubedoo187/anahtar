mod cli;
mod clipboard;
mod config;
mod generator;
mod printing;
mod prompts;
mod vault;

use anahtar_core::{
    add_entry_save_as, delete_entry_save_as, edit_entry_save_as, inspect_header, list_entries,
    open_database, search_entries, show_entry, totp_code, upgrade_to_kdbx41, AddEntryRequest,
    EditEntryRequest, SaveAsOptions,
};
use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use clipboard::copy_with_clear;
use config::{handle_config, load_config, validate_generator_length};
use generator::generate_password;
use printing::{
    print_detail, print_entries, print_totp, print_upgrade_report, print_vault_info,
    print_write_report,
};
use prompts::{confirm_delete, prompt_entry_password_with_confirmation, prompt_password};
use std::path::PathBuf;
use vault::{ensure_edit_has_change, preflight_output, resolve_vault, validate_uuid_selector};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { vault, json } => {
            let vault = resolve_vault(vault)?;
            let info = inspect_header(vault)?;
            print_vault_info(&info, json)?;
        }
        Command::List { vault, json } => {
            let vault = resolve_vault(vault)?;
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let entries = list_entries(&db);
            print_entries(&entries, json)?;
        }
        Command::Search { vault, query, json } => {
            let vault = resolve_vault(vault)?;
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
            let vault = resolve_vault(vault)?;
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let detail = show_entry(&db, &selector, reveal_password)?;
            print_detail(&detail, json)?;
        }
        Command::Upgrade {
            vault,
            output,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if !dry_run {
                preflight_output(&output, force)?;
            }
            let password = prompt_password()?;
            let report = upgrade_to_kdbx41(vault, output, &password, force, dry_run)?;
            print_upgrade_report(&report, json)?;
        }
        Command::Add {
            vault,
            output,
            group,
            title,
            username,
            url,
            notes,
            password_prompt,
            no_password,
            generate_password,
            force,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            preflight_output(&output, force)?;
            validate_add_password_mode(password_prompt, no_password, generate_password)?;
            let password = prompt_password()?;
            let entry_password =
                resolve_entry_password(password_prompt, no_password, generate_password)?;
            let report = add_entry_save_as(
                vault,
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
            vault,
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
            ensure_edit_has_change(&title, &username, &url, &notes, password_prompt)?;
            let vault = resolve_vault(vault)?;
            preflight_output(&output, force)?;
            let password = prompt_password()?;
            let entry_password = if password_prompt {
                Some(prompt_entry_password_with_confirmation()?)
            } else {
                None
            };
            let report = edit_entry_save_as(
                vault,
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
            vault,
            entry_id,
            output,
            yes,
            force,
            json,
        } => {
            validate_uuid_selector(&entry_id)?;
            let vault = resolve_vault(vault)?;
            preflight_output(&output, force)?;
            let password = prompt_password()?;
            let db = open_database(&vault, &password)?;
            let detail = show_entry(&db, &entry_id, false)?;
            if !yes {
                confirm_delete(&detail)?;
            }
            let report = delete_entry_save_as(
                vault,
                &entry_id,
                &password,
                SaveAsOptions {
                    output_path: output,
                    force,
                },
            )?;
            print_write_report(&report, json)?;
        }
        Command::Config { command } => handle_config(command)?,
        Command::CopyPassword {
            vault,
            selector,
            clear_after,
        } => {
            let value = secret_field(vault, &selector, SecretField::Password)?;
            copy_with_clear(&value, clear_after)?;
        }
        Command::CopyUsername {
            vault,
            selector,
            clear_after,
        } => {
            let value = secret_field(vault, &selector, SecretField::Username)?;
            copy_with_clear(&value, clear_after)?;
        }
        Command::CopyUrl {
            vault,
            selector,
            clear_after,
        } => {
            let value = secret_field(vault, &selector, SecretField::Url)?;
            copy_with_clear(&value, clear_after)?;
        }
        Command::Generate {
            length,
            copy,
            clear_after,
        } => {
            let config = load_config()?;
            let length = length.unwrap_or(config.generator_length);
            validate_generator_length(length)?;
            let password = generate_password(length)?;
            if copy {
                copy_with_clear(&password, clear_after)?;
            } else {
                println!("{password}");
            }
        }
        Command::Totp {
            vault,
            copy,
            selector,
            clear_after,
        } => {
            let vault = resolve_vault(vault)?;
            let password = prompt_password()?;
            let db = open_database(vault, &password)?;
            let code = totp_code(&db, &selector)?;
            if copy {
                copy_with_clear(&code.code, clear_after)?;
            } else {
                print_totp(&code);
            }
        }
    }
    Ok(())
}

fn validate_add_password_mode(
    password_prompt: bool,
    no_password: bool,
    generate_password: bool,
) -> Result<()> {
    let count = [password_prompt, no_password, generate_password]
        .into_iter()
        .filter(|v| *v)
        .count();
    if count != 1 {
        anyhow::bail!(
            "use exactly one of --password-prompt, --no-password, or --generate-password"
        );
    }
    Ok(())
}

fn resolve_entry_password(
    password_prompt: bool,
    no_password: bool,
    generate_password_flag: bool,
) -> Result<Option<String>> {
    validate_add_password_mode(password_prompt, no_password, generate_password_flag)?;
    if no_password {
        Ok(None)
    } else if generate_password_flag {
        Ok(Some(generate_password(load_config()?.generator_length)?))
    } else {
        Ok(Some(prompt_entry_password_with_confirmation()?))
    }
}

#[derive(Debug, Clone, Copy)]
enum SecretField {
    Password,
    Username,
    Url,
}

fn secret_field(vault: Option<PathBuf>, selector: &str, field: SecretField) -> Result<String> {
    let vault = resolve_vault(vault)?;
    let password = prompt_password()?;
    let db = open_database(vault, &password)?;
    let detail = show_entry(&db, selector, matches!(field, SecretField::Password))?;
    match field {
        SecretField::Password => detail.password,
        SecretField::Username => detail.username,
        SecretField::Url => detail.url,
    }
    .filter(|v| !v.is_empty())
    .ok_or_else(|| anyhow::anyhow!("requested field is empty or unavailable"))
}
