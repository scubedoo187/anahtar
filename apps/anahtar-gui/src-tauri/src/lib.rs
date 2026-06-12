use anahtar_app::{AnahtarService, WriteMode};
use anahtar_core::{
    AddEntryRequest, AuditReport, EditEntryRequest, EntryDetail, EntrySelector, EntrySummary,
    GroupSummary, TotpCode, VaultCredentials, VaultInfo, WriteReport,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

type GuiResult<T> = Result<T, GuiError>;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct GuiError {
    kind: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct BackendStatus {
    app: &'static str,
    version: &'static str,
    service: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct GuiConfig {
    last_vault_path: Option<String>,
    recent_vaults: Vec<RecentVault>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RecentVault {
    path: String,
    key_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GuiAddEntryRequest {
    group_path: String,
    title: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GuiEditEntryRequest {
    title: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    backup_dir: Option<String>,
}

#[tauri::command]
fn backend_status() -> BackendStatus {
    BackendStatus {
        app: "Anahtar",
        version: env!("CARGO_PKG_VERSION"),
        service: "anahtar-app ready",
    }
}

#[tauri::command]
fn load_gui_config(app: tauri::AppHandle) -> GuiResult<GuiConfig> {
    read_gui_config(&app).map_err(|error| gui_error("config_failed", error))
}

#[tauri::command]
fn remember_vault(
    app: tauri::AppHandle,
    path: String,
    key_file: Option<String>,
) -> GuiResult<GuiConfig> {
    let mut config = read_gui_config(&app).map_err(|error| gui_error("config_failed", error))?;
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(gui_error("validation_failed", "vault path is required"));
    }

    config.last_vault_path = Some(path.clone());
    config.recent_vaults.retain(|vault| vault.path != path);
    config.recent_vaults.insert(
        0,
        RecentVault {
            path,
            key_file: key_file.filter(|value| !value.trim().is_empty()),
        },
    );
    config.recent_vaults.truncate(5);
    write_gui_config(&app, &config).map_err(|error| gui_error("config_failed", error))?;
    Ok(config)
}

#[tauri::command]
fn clear_recent_vaults(app: tauri::AppHandle) -> GuiResult<GuiConfig> {
    let config = GuiConfig::default();
    write_gui_config(&app, &config).map_err(|error| gui_error("config_failed", error))?;
    Ok(config)
}

#[tauri::command]
fn inspect_vault(path: String) -> GuiResult<VaultInfo> {
    AnahtarService::inspect(path).map_err(|error| gui_error("inspect_failed", error))
}

#[tauri::command]
fn unlock_vault(
    path: String,
    password: String,
    key_file: Option<String>,
) -> GuiResult<Vec<EntrySummary>> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::list(path, &credentials).map_err(|error| gui_error("unlock_failed", error))
}

#[tauri::command]
fn search_entries(
    path: String,
    password: String,
    key_file: Option<String>,
    query: String,
) -> GuiResult<Vec<EntrySummary>> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::search(path, &credentials, &query)
        .map_err(|error| gui_error("read_failed", error))
}

#[tauri::command]
fn show_entry(
    path: String,
    password: String,
    key_file: Option<String>,
    selector_kind: String,
    selector_value: String,
    reveal_password: bool,
) -> GuiResult<EntryDetail> {
    let credentials = credentials_from_gui(password, key_file);
    let selector = selector_from_gui(&selector_kind, selector_value)?;
    AnahtarService::show(path, &credentials, &selector, reveal_password)
        .map_err(|error| gui_error("read_failed", error))
}

#[tauri::command]
fn totp_code(
    path: String,
    password: String,
    key_file: Option<String>,
    selector_kind: String,
    selector_value: String,
) -> GuiResult<TotpCode> {
    let credentials = credentials_from_gui(password, key_file);
    let selector = selector_from_gui(&selector_kind, selector_value)?;
    AnahtarService::totp(path, &credentials, &selector)
        .map_err(|error| gui_error("totp_failed", error))
}

#[tauri::command]
fn list_groups(
    path: String,
    password: String,
    key_file: Option<String>,
) -> GuiResult<Vec<GroupSummary>> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::groups(path, &credentials).map_err(|error| gui_error("group_failed", error))
}

#[tauri::command]
fn audit_vault(path: String, password: String, key_file: Option<String>) -> GuiResult<AuditReport> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::audit(path, &credentials).map_err(|error| gui_error("audit_failed", error))
}

#[tauri::command]
fn add_entry(
    path: String,
    password: String,
    key_file: Option<String>,
    request: GuiAddEntryRequest,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    let backup_dir = optional_path(request.backup_dir);
    AnahtarService::add_entry(
        &path,
        &credentials,
        AddEntryRequest {
            group_path: request.group_path,
            title: request.title,
            username: empty_to_none(request.username),
            password: empty_secret_to_none(request.password),
            url: empty_to_none(request.url),
            notes: empty_to_none(request.notes),
        },
        WriteMode::InPlace { backup_dir },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "add entry dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn edit_entry(
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    request: GuiEditEntryRequest,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    let backup_dir = optional_path(request.backup_dir);
    AnahtarService::edit_entry(
        &path,
        &credentials,
        &entry_id,
        EditEntryRequest {
            title: empty_to_none(request.title),
            username: empty_to_none(request.username),
            password: empty_secret_to_none(request.password),
            url: empty_to_none(request.url),
            notes: empty_to_none(request.notes),
        },
        WriteMode::InPlace { backup_dir },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "edit entry dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn delete_entry(
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    backup_dir: Option<String>,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::delete_entry(
        &path,
        &credentials,
        &entry_id,
        WriteMode::InPlace {
            backup_dir: optional_path(backup_dir),
        },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "delete entry dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn add_group(
    path: String,
    password: String,
    key_file: Option<String>,
    group_path: String,
    backup_dir: Option<String>,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::add_group(
        &path,
        &credentials,
        &group_path,
        WriteMode::InPlace {
            backup_dir: optional_path(backup_dir),
        },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "add group dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn rename_group(
    path: String,
    password: String,
    key_file: Option<String>,
    group_path: String,
    new_name: String,
    backup_dir: Option<String>,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::rename_group(
        &path,
        &credentials,
        &group_path,
        &new_name,
        WriteMode::InPlace {
            backup_dir: optional_path(backup_dir),
        },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "rename group dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn delete_group(
    path: String,
    password: String,
    key_file: Option<String>,
    group_path: String,
    backup_dir: Option<String>,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    let entries = AnahtarService::list(&path, &credentials)
        .map_err(|error| gui_error("write_failed", error))?;
    let entry_count = entries
        .iter()
        .filter(|entry| entry_is_in_group(&entry.group_path, &group_path))
        .count();
    if entry_count > 0 {
        return Err(gui_error(
            "validation_failed",
            format!("group contains {entry_count} entries"),
        ));
    }

    AnahtarService::delete_group(
        &path,
        &credentials,
        &group_path,
        WriteMode::InPlace {
            backup_dir: optional_path(backup_dir),
        },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "delete group dry-run returned no write report",
        )
    })
}

#[tauri::command]
fn move_entry(
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    group_path: String,
    backup_dir: Option<String>,
) -> GuiResult<WriteReport> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::move_entry(
        &path,
        &credentials,
        &EntrySelector::Id(entry_id),
        &group_path,
        WriteMode::InPlace {
            backup_dir: optional_path(backup_dir),
        },
    )
    .map_err(|error| gui_error("write_failed", error))?
    .ok_or_else(|| {
        gui_error(
            "internal_failed",
            "move entry dry-run returned no write report",
        )
    })
}

fn credentials_from_gui(password: String, key_file: Option<String>) -> VaultCredentials {
    VaultCredentials {
        password,
        key_file: key_file
            .filter(|value| !value.is_empty())
            .map(PathBuf::from),
    }
}

fn selector_from_gui(kind: &str, value: String) -> GuiResult<EntrySelector> {
    if value.trim().is_empty() {
        return Err(gui_error(
            "validation_failed",
            "entry selector value is required",
        ));
    }

    match kind {
        "id" => Ok(EntrySelector::Id(value)),
        "title" => Ok(EntrySelector::Title(value)),
        "url" => Ok(EntrySelector::Url(value)),
        "username" => Ok(EntrySelector::Username(value)),
        "auto" => Ok(EntrySelector::Auto(value)),
        _ => Err(gui_error(
            "validation_failed",
            "unknown entry selector kind",
        )),
    }
}

fn optional_path(value: Option<String>) -> Option<PathBuf> {
    empty_to_none(value).map(PathBuf::from)
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn empty_secret_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.is_empty()).then_some(value))
}

fn entry_is_in_group(entry_group_path: &str, group_path: &str) -> bool {
    let entry = normalize_group_path(entry_group_path);
    let group = normalize_group_path(group_path);
    entry == group || entry.starts_with(&format!("{group}/"))
}

fn normalize_group_path(path: &str) -> String {
    path.trim_matches('/')
        .strip_prefix("Root/")
        .unwrap_or(path.trim_matches('/'))
        .to_string()
}

fn gui_error(kind: &'static str, error: impl std::fmt::Display) -> GuiError {
    GuiError {
        kind,
        message: error.to_string(),
    }
}

fn read_gui_config(app: &tauri::AppHandle) -> std::io::Result<GuiConfig> {
    let path = gui_config_path(app)?;
    if !path.exists() {
        return Ok(GuiConfig::default());
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents).unwrap_or_default())
}

fn write_gui_config(app: &tauri::AppHandle, config: &GuiConfig) -> std::io::Result<()> {
    let path = gui_config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(config)?;
    std::fs::write(path, contents)
}

fn gui_config_path(app: &tauri::AppHandle) -> std::io::Result<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("gui-config.json"))
        .map_err(|error| std::io::Error::other(error.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            backend_status,
            load_gui_config,
            remember_vault,
            clear_recent_vaults,
            inspect_vault,
            unlock_vault,
            search_entries,
            show_entry,
            totp_code,
            list_groups,
            audit_vault,
            add_entry,
            edit_entry,
            delete_entry,
            add_group,
            rename_group,
            delete_group,
            move_entry
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Anahtar GUI");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_vault_path() -> String {
        "../../../test-vaults/generated/phase3-base.kdbx".to_string()
    }

    #[test]
    fn backend_commands_read_generated_vault_without_revealing_password() {
        let path = generated_vault_path();
        if !std::path::Path::new(&path).exists() {
            return;
        }

        let info = inspect_vault(path.clone()).unwrap();
        assert!(info.file_size_bytes > 0);

        let entries = unlock_vault(path.clone(), "testpass".to_string(), None).unwrap();
        assert!(!entries.is_empty());

        let search = search_entries(
            path.clone(),
            "testpass".to_string(),
            None,
            "Github".to_string(),
        )
        .unwrap();
        assert_eq!(search.len(), 1);

        let detail = show_entry(
            path.clone(),
            "testpass".to_string(),
            None,
            "title".to_string(),
            "Github Test".to_string(),
            false,
        )
        .unwrap();
        assert_eq!(detail.title.as_deref(), Some("Github Test"));
        assert!(detail.password.is_none());

        let groups = list_groups(path.clone(), "testpass".to_string(), None).unwrap();
        assert!(groups.iter().any(|group| group.path == "Root/General/Web"));

        let audit = audit_vault(path, "testpass".to_string(), None).unwrap();
        let audit_json = serde_json::to_string(&audit).unwrap();
        assert!(!audit_json.contains("github-pass"));
    }

    #[test]
    fn empty_secret_to_none_preserves_non_empty_secret_whitespace() {
        assert_eq!(
            empty_secret_to_none(Some("  spaced  ".to_string())).as_deref(),
            Some("  spaced  ")
        );
        assert_eq!(empty_secret_to_none(Some(String::new())), None);
    }

    #[test]
    fn invalid_selector_returns_validation_error() {
        let err = selector_from_gui("unknown", "value".to_string()).unwrap_err();
        assert_eq!(err.kind, "validation_failed");
        assert_eq!(err.message, "unknown entry selector kind");
    }

    #[test]
    fn wrong_password_returns_safe_error() {
        let path = generated_vault_path();
        if !std::path::Path::new(&path).exists() {
            return;
        }

        let err = unlock_vault(path, "wrong-password".to_string(), None).unwrap_err();
        assert_eq!(err.kind, "unlock_failed");
        assert!(err.message.contains("failed to open database"));
        assert!(!err.message.contains("wrong-password"));
    }

    #[test]
    fn write_commands_update_generated_vault_copy_and_report_backup() {
        let source = generated_vault_path();
        if !std::path::Path::new(&source).exists() {
            return;
        }

        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("gui-write.kdbx");
        std::fs::copy(source, &target).unwrap();
        let backup_dir = temp.path().join("backups").to_string_lossy().to_string();
        let target = target.to_string_lossy().to_string();

        let add_report = add_entry(
            target.clone(),
            "testpass".to_string(),
            None,
            GuiAddEntryRequest {
                group_path: "General/Web".to_string(),
                title: "GUI Write Test".to_string(),
                username: Some("gui-user".to_string()),
                password: Some("gui-pass".to_string()),
                url: Some("https://gui.example.com".to_string()),
                notes: Some("created from GUI command test".to_string()),
                backup_dir: Some(backup_dir.clone()),
            },
        )
        .unwrap();
        assert!(add_report.backup_path.is_some());
        let entry_id = add_report.changed_entry_id.unwrap();

        let edit_report = edit_entry(
            target.clone(),
            "testpass".to_string(),
            None,
            entry_id.clone(),
            GuiEditEntryRequest {
                title: None,
                username: Some("gui-user-updated".to_string()),
                password: None,
                url: None,
                notes: None,
                backup_dir: Some(backup_dir.clone()),
            },
        )
        .unwrap();
        assert!(edit_report.backup_path.is_some());

        let delete_report = delete_entry(
            target,
            "testpass".to_string(),
            None,
            entry_id,
            Some(backup_dir),
        )
        .unwrap();
        assert!(delete_report.backup_path.is_some());
    }
}
