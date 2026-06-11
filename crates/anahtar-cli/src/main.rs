mod cli;
mod clipboard;
mod config;
mod generator;
mod printing;
mod prompts;
mod vault;

use anahtar_core::{
    add_entry_save_as_with_credentials, add_group_save_as_with_credentials, audit_database,
    delete_entry_save_as_with_credentials, delete_group_save_as_with_credentials,
    edit_entry_save_as_with_credentials, inspect_header, list_entries, list_groups,
    move_entry_save_as_with_credentials, open_database_with_credentials,
    rename_group_save_as_with_credentials, safe_in_place_write_with_credentials, search_entries,
    show_entry_by_selector, totp_code_by_selector, upgrade_to_kdbx41_with_credentials,
    AddEntryRequest, EditEntryRequest, EntrySelector, InPlaceOptions, SaveAsOptions,
    VaultCredentials, WriteReport,
};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Cli, Command, EntrySelectorArgs, GroupCommand};
use clipboard::copy_with_clear;
use config::{handle_config, load_config, validate_generator_length};
use generator::generate_password;
use printing::{
    print_audit, print_detail, print_dry_run, print_entries, print_groups, print_totp,
    print_upgrade_report, print_vault_info, print_write_report,
};
use prompts::{
    confirm_delete, confirm_group_delete, prompt_entry_password_with_confirmation, prompt_password,
};
use std::path::PathBuf;
use vault::{ensure_edit_has_change, preflight_output, resolve_vault};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { vault, json } => {
            let vault = resolve_vault(vault)?;
            let info = inspect_header(vault)?;
            print_vault_info(&info, json)?;
        }
        Command::List {
            vault,
            key_file,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            let entries = list_entries(&db);
            print_entries(&entries, json)?;
        }
        Command::Search {
            vault,
            key_file,
            query,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            let entries = search_entries(&db, &query);
            print_entries(&entries, json)?;
        }
        Command::Show {
            vault,
            key_file,
            selector,
            reveal_password,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            let selector = selector_from_args(selector)?;
            let detail = show_entry_by_selector(&db, &selector, reveal_password)?;
            print_detail(&detail, json)?;
        }
        Command::Upgrade {
            vault,
            key_file,
            output,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if !dry_run {
                preflight_output(&output, force)?;
            }
            let credentials = prompt_credentials(key_file)?;
            let report =
                upgrade_to_kdbx41_with_credentials(vault, output, &credentials, force, dry_run)?;
            print_upgrade_report(&report, json)?;
        }
        Command::Add {
            vault,
            key_file,
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
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            validate_add_password_mode(password_prompt, no_password, generate_password)?;
            if dry_run {
                print_dry_run("add", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let credentials = prompt_credentials(key_file)?;
            let entry_password =
                resolve_entry_password(password_prompt, no_password, generate_password)?;
            let request = AddEntryRequest {
                group_path: group,
                title,
                username,
                password: entry_password,
                url,
                notes,
            };
            let report = if let Some(output) = output {
                add_entry_save_as_with_credentials(
                    vault,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    request,
                )?
            } else {
                add_entry_in_place(vault, &credentials, request)?
            };
            print_write_report(&report, json)?;
        }
        Command::Edit {
            vault,
            key_file,
            selector,
            output,
            title,
            username,
            url,
            notes,
            password_prompt,
            force,
            dry_run,
            json,
        } => {
            ensure_edit_has_change(&title, &username, &url, &notes, password_prompt)?;
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            let credentials = prompt_credentials(key_file)?;
            let selector = selector_from_args(selector)?;
            let selector_id = resolve_selector_id(&vault, &credentials, &selector)?;
            if dry_run {
                print_dry_run("edit", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let entry_password = if password_prompt {
                Some(prompt_entry_password_with_confirmation()?)
            } else {
                None
            };
            let request = EditEntryRequest {
                title,
                username,
                password: entry_password,
                url,
                notes,
            };
            let report = if let Some(output) = output {
                edit_entry_save_as_with_credentials(
                    vault,
                    &selector_id,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    request,
                )?
            } else {
                edit_entry_in_place(vault, &selector_id, &credentials, request)?
            };
            print_write_report(&report, json)?;
        }
        Command::Delete {
            vault,
            key_file,
            selector,
            output,
            yes,
            force,
            dry_run,
            json,
        } => {
            let selector = selector_from_args(selector)?;
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(&vault, &credentials)?;
            let detail = show_entry_by_selector(&db, &selector, false)?;
            if dry_run {
                print_dry_run("delete", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            if !yes {
                confirm_delete(&detail)?;
            }
            let report = if let Some(output) = output {
                delete_entry_save_as_with_credentials(
                    vault,
                    &detail.id,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                )?
            } else {
                delete_entry_in_place(vault, &detail.id, &credentials)?
            };
            print_write_report(&report, json)?;
        }
        Command::Config { command } => handle_config(command)?,
        Command::Group { command } => handle_group(command)?,
        Command::Move {
            vault,
            key_file,
            selector,
            group,
            output,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            let credentials = prompt_credentials(key_file)?;
            let selector = selector_from_args(selector)?;
            if dry_run {
                let _ = resolve_selector_id(&vault, &credentials, &selector)?;
                print_dry_run("move", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let report = if let Some(output) = output {
                move_entry_save_as_with_credentials(
                    vault,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    &selector,
                    &group,
                )?
            } else {
                move_entry_in_place(vault, &credentials, &selector, &group)?
            };
            print_write_report(&report, json)?;
        }
        Command::Completions { shell } => {
            let mut command = Cli::command();
            let name = command.get_name().to_string();
            clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
        }
        Command::Audit {
            vault,
            key_file,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            let report = audit_database(&db);
            print_audit(&report, json)?;
        }
        Command::CopyPassword {
            vault,
            key_file,
            selector,
            clear_after,
        } => {
            let selector = selector_from_args(selector)?;
            let value = secret_field(vault, key_file, &selector, SecretField::Password)?;
            copy_with_clear(&value, clear_after)?;
        }
        Command::CopyUsername {
            vault,
            key_file,
            selector,
            clear_after,
        } => {
            let selector = selector_from_args(selector)?;
            let value = secret_field(vault, key_file, &selector, SecretField::Username)?;
            copy_with_clear(&value, clear_after)?;
        }
        Command::CopyUrl {
            vault,
            key_file,
            selector,
            clear_after,
        } => {
            let selector = selector_from_args(selector)?;
            let value = secret_field(vault, key_file, &selector, SecretField::Url)?;
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
            key_file,
            copy,
            selector,
            clear_after,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            let selector = selector_from_args(selector)?;
            let code = totp_code_by_selector(&db, &selector)?;
            if copy {
                copy_with_clear(&code.code, clear_after)?;
            } else {
                print_totp(&code);
            }
        }
    }
    Ok(())
}

fn selector_from_args(args: EntrySelectorArgs) -> Result<EntrySelector> {
    let selectors = [
        args.selector.map(EntrySelector::Auto),
        args.id.map(EntrySelector::Id),
        args.title.map(EntrySelector::Title),
        args.url.map(EntrySelector::Url),
        args.username.map(EntrySelector::Username),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    match selectors.as_slice() {
        [selector] => Ok(selector.clone()),
        [] => anyhow::bail!("entry selector required: provide a positional selector or one of --id/--title/--url/--username"),
        _ => anyhow::bail!("provide exactly one entry selector"),
    }
}

fn resolve_selector_id(
    vault: &PathBuf,
    credentials: &VaultCredentials,
    selector: &EntrySelector,
) -> Result<String> {
    let db = open_database_with_credentials(vault, credentials)?;
    Ok(show_entry_by_selector(&db, selector, false)?.id)
}

fn prompt_credentials(cli_key_file: Option<PathBuf>) -> Result<VaultCredentials> {
    let key_file = resolve_key_file(cli_key_file)?;
    let password = prompt_password()?;
    Ok(VaultCredentials { password, key_file })
}

fn resolve_key_file(cli_key_file: Option<PathBuf>) -> Result<Option<PathBuf>> {
    if let Some(path) = cli_key_file {
        return canonicalize_existing_file(path, "key-file").map(Some);
    }
    Ok(load_config()?.key_file)
}

fn canonicalize_existing_file(path: PathBuf, label: &str) -> Result<PathBuf> {
    if !path.exists() {
        anyhow::bail!("{label} path does not exist: {}", path.display());
    }
    if !path.is_file() {
        anyhow::bail!("{label} path is not a file: {}", path.display());
    }
    Ok(path.canonicalize()?)
}

fn handle_group(command: GroupCommand) -> Result<()> {
    match command {
        GroupCommand::List {
            vault,
            key_file,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            let credentials = prompt_credentials(key_file)?;
            let db = open_database_with_credentials(vault, &credentials)?;
            print_groups(&list_groups(&db), json)?;
        }
        GroupCommand::Add {
            vault,
            key_file,
            path,
            output,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            if dry_run {
                print_dry_run("group add", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let credentials = prompt_credentials(key_file)?;
            let report = if let Some(output) = output {
                add_group_save_as_with_credentials(
                    vault,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    &path,
                )?
            } else {
                add_group_in_place(vault, &credentials, &path)?
            };
            print_write_report(&report, json)?;
        }
        GroupCommand::Rename {
            vault,
            key_file,
            path,
            new_name,
            output,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            if dry_run {
                print_dry_run("group rename", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let credentials = prompt_credentials(key_file)?;
            let report = if let Some(output) = output {
                rename_group_save_as_with_credentials(
                    vault,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    &path,
                    &new_name,
                )?
            } else {
                rename_group_in_place(vault, &credentials, &path, &new_name)?
            };
            print_write_report(&report, json)?;
        }
        GroupCommand::Delete {
            vault,
            key_file,
            path,
            output,
            yes,
            force,
            dry_run,
            json,
        } => {
            let vault = resolve_vault(vault)?;
            if let Some(output) = &output {
                preflight_output(output, force)?;
            }
            if dry_run {
                print_dry_run("group delete", &vault, output.as_ref(), json)?;
                return Ok(());
            }
            let credentials = prompt_credentials(key_file)?;
            if !yes {
                confirm_group_delete(&path)?;
            }
            let report = if let Some(output) = output {
                delete_group_save_as_with_credentials(
                    vault,
                    &credentials,
                    SaveAsOptions {
                        output_path: output,
                        force,
                    },
                    &path,
                )?
            } else {
                delete_group_in_place(vault, &credentials, &path)?
            };
            print_write_report(&report, json)?;
        }
    }
    Ok(())
}

fn add_group_in_place(
    vault: PathBuf,
    credentials: &VaultCredentials,
    path: &str,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        add_group_save_as_with_credentials(
            input,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            path,
        )
    })
}

fn rename_group_in_place(
    vault: PathBuf,
    credentials: &VaultCredentials,
    path: &str,
    new_name: &str,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        rename_group_save_as_with_credentials(
            input,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            path,
            new_name,
        )
    })
}

fn delete_group_in_place(
    vault: PathBuf,
    credentials: &VaultCredentials,
    path: &str,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        delete_group_save_as_with_credentials(
            input,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            path,
        )
    })
}

fn move_entry_in_place(
    vault: PathBuf,
    credentials: &VaultCredentials,
    selector: &EntrySelector,
    group: &str,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        move_entry_save_as_with_credentials(
            input,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            selector,
            group,
        )
    })
}

fn add_entry_in_place(
    vault: PathBuf,
    credentials: &VaultCredentials,
    request: AddEntryRequest,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        add_entry_save_as_with_credentials(
            input,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            request,
        )
    })
}

fn edit_entry_in_place(
    vault: PathBuf,
    selector: &str,
    credentials: &VaultCredentials,
    request: EditEntryRequest,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        edit_entry_save_as_with_credentials(
            input,
            selector,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
            request,
        )
    })
}

fn delete_entry_in_place(
    vault: PathBuf,
    entry_id: &str,
    credentials: &VaultCredentials,
) -> Result<WriteReport> {
    safe_in_place(vault, credentials, |input, output| {
        delete_entry_save_as_with_credentials(
            input,
            entry_id,
            credentials,
            SaveAsOptions {
                output_path: output,
                force: false,
            },
        )
    })
}

fn safe_in_place<F>(
    vault: PathBuf,
    credentials: &VaultCredentials,
    save_as: F,
) -> Result<WriteReport>
where
    F: FnOnce(PathBuf, PathBuf) -> anahtar_core::Result<WriteReport>,
{
    let backup_dir = load_config()?.backup_dir;
    Ok(safe_in_place_write_with_credentials(
        credentials,
        InPlaceOptions {
            target_path: vault,
            backup_dir,
        },
        |input, output| save_as(input.to_path_buf(), output.to_path_buf()),
    )?)
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

fn secret_field(
    vault: Option<PathBuf>,
    key_file: Option<PathBuf>,
    selector: &EntrySelector,
    field: SecretField,
) -> Result<String> {
    let vault = resolve_vault(vault)?;
    let credentials = prompt_credentials(key_file)?;
    let db = open_database_with_credentials(vault, &credentials)?;
    let detail = show_entry_by_selector(&db, selector, matches!(field, SecretField::Password))?;
    match field {
        SecretField::Password => detail.password,
        SecretField::Username => detail.username,
        SecretField::Url => detail.url,
    }
    .filter(|v| !v.is_empty())
    .ok_or_else(|| anyhow::anyhow!("requested field is empty or unavailable"))
}
