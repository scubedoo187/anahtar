//! GUI/CLI reusable application facade for Anahtar.
//!
//! This crate intentionally remains stateless: callers provide vault paths and
//! credentials per operation so master passwords are not stored in long-lived
//! service state.

use anahtar_core::{
    add_entry_save_as_with_credentials, add_group_save_as_with_credentials, audit_database,
    delete_entry_save_as_with_credentials, delete_group_save_as_with_credentials,
    edit_entry_save_as_with_credentials, inspect_header, list_entries, list_groups,
    move_entry_save_as_with_credentials, open_database_with_credentials,
    rename_group_save_as_with_credentials, safe_in_place_write_with_credentials, search_entries,
    show_entry_by_selector, totp_code_by_selector, AddEntryRequest, AuditReport, EditEntryRequest,
    EntryDetail, EntrySelector, EntrySummary, GroupSummary, InPlaceOptions, Result, SaveAsOptions,
    TotpCode, VaultCredentials, VaultInfo, WriteReport,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum WriteMode {
    SaveAs { output_path: PathBuf, force: bool },
    InPlace { backup_dir: Option<PathBuf> },
    DryRun,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnahtarService;

impl AnahtarService {
    pub fn inspect(path: impl AsRef<Path>) -> Result<VaultInfo> {
        inspect_header(path)
    }

    pub fn list(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
    ) -> Result<Vec<EntrySummary>> {
        let db = open_database_with_credentials(path, credentials)?;
        Ok(list_entries(&db))
    }

    pub fn search(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        query: &str,
    ) -> Result<Vec<EntrySummary>> {
        let db = open_database_with_credentials(path, credentials)?;
        Ok(search_entries(&db, query))
    }

    pub fn show(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        selector: &EntrySelector,
        reveal_password: bool,
    ) -> Result<EntryDetail> {
        let db = open_database_with_credentials(path, credentials)?;
        show_entry_by_selector(&db, selector, reveal_password)
    }

    pub fn groups(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
    ) -> Result<Vec<GroupSummary>> {
        let db = open_database_with_credentials(path, credentials)?;
        Ok(list_groups(&db))
    }

    pub fn audit(path: impl AsRef<Path>, credentials: &VaultCredentials) -> Result<AuditReport> {
        let db = open_database_with_credentials(path, credentials)?;
        Ok(audit_database(&db))
    }

    pub fn totp(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        selector: &EntrySelector,
    ) -> Result<TotpCode> {
        let db = open_database_with_credentials(path, credentials)?;
        totp_code_by_selector(&db, selector)
    }

    pub fn add_entry(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        request: AddEntryRequest,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => add_entry_save_as_with_credentials(
                input_path,
                credentials,
                SaveAsOptions { output_path, force },
                request,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    add_entry_save_as_with_credentials(
                        input,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        request,
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn edit_entry(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        selector_id: &str,
        request: EditEntryRequest,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => edit_entry_save_as_with_credentials(
                input_path,
                selector_id,
                credentials,
                SaveAsOptions { output_path, force },
                request,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    edit_entry_save_as_with_credentials(
                        input,
                        selector_id,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        request,
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn delete_entry(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        entry_id: &str,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => delete_entry_save_as_with_credentials(
                input_path,
                entry_id,
                credentials,
                SaveAsOptions { output_path, force },
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    delete_entry_save_as_with_credentials(
                        input,
                        entry_id,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn add_group(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        group_path: &str,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => add_group_save_as_with_credentials(
                input_path,
                credentials,
                SaveAsOptions { output_path, force },
                group_path,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    add_group_save_as_with_credentials(
                        input,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        group_path,
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn rename_group(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        group_path: &str,
        new_name: &str,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => rename_group_save_as_with_credentials(
                input_path,
                credentials,
                SaveAsOptions { output_path, force },
                group_path,
                new_name,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    rename_group_save_as_with_credentials(
                        input,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        group_path,
                        new_name,
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn delete_group(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        group_path: &str,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => delete_group_save_as_with_credentials(
                input_path,
                credentials,
                SaveAsOptions { output_path, force },
                group_path,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    delete_group_save_as_with_credentials(
                        input,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        group_path,
                    )
                },
            )
            .map(Some),
        }
    }

    pub fn move_entry(
        path: impl AsRef<Path>,
        credentials: &VaultCredentials,
        selector: &EntrySelector,
        group_path: &str,
        mode: WriteMode,
    ) -> Result<Option<WriteReport>> {
        let input_path = path.as_ref();
        match mode {
            WriteMode::DryRun => Ok(None),
            WriteMode::SaveAs { output_path, force } => move_entry_save_as_with_credentials(
                input_path,
                credentials,
                SaveAsOptions { output_path, force },
                selector,
                group_path,
            )
            .map(Some),
            WriteMode::InPlace { backup_dir } => safe_in_place_write_with_credentials(
                credentials,
                InPlaceOptions {
                    target_path: input_path.to_path_buf(),
                    backup_dir,
                },
                |input, output| {
                    move_entry_save_as_with_credentials(
                        input,
                        credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        selector,
                        group_path,
                    )
                },
            )
            .map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_vault() -> PathBuf {
        PathBuf::from("../../test-vaults/generated/phase3-base.kdbx")
    }

    #[test]
    fn service_lists_generated_vault_entries() {
        let vault = generated_vault();
        if !vault.exists() {
            return;
        }
        let credentials = VaultCredentials::password_only("testpass");
        let entries = AnahtarService::list(vault, &credentials).unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn service_save_as_add_entry_flow() {
        let vault = generated_vault();
        if !vault.exists() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("app-add.kdbx");
        let credentials = VaultCredentials::password_only("testpass");
        let report = AnahtarService::add_entry(
            &vault,
            &credentials,
            AddEntryRequest {
                group_path: "General/Web".to_string(),
                title: "App Service Example".to_string(),
                username: Some("app@example.com".to_string()),
                password: Some("app-service-password".to_string()),
                url: Some("https://example.com".to_string()),
                notes: None,
            },
            WriteMode::SaveAs {
                output_path: output.clone(),
                force: false,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(report.output_path, output);
        assert!(report.changed_entry_id.is_some());
    }
}
