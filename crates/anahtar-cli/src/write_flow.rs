use anahtar_core::{
    add_entry_save_as_with_credentials, add_group_save_as_with_credentials,
    delete_entry_save_as_with_credentials, delete_group_save_as_with_credentials,
    edit_entry_save_as_with_credentials, move_entry_save_as_with_credentials,
    rename_group_save_as_with_credentials, safe_in_place_write_with_credentials, AddEntryRequest,
    EditEntryRequest, EntrySelector, InPlaceOptions, SaveAsOptions, VaultCredentials, WriteReport,
};
use anyhow::Result;
use std::path::PathBuf;

use crate::config::load_config;

pub fn add_group_in_place(
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

pub fn rename_group_in_place(
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

pub fn delete_group_in_place(
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

pub fn move_entry_in_place(
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

pub fn add_entry_in_place(
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

pub fn edit_entry_in_place(
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

pub fn delete_entry_in_place(
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
