use anahtar_app::AnahtarService;
use anahtar_core::{EntryDetail, EntrySelector, EntrySummary, VaultCredentials, VaultInfo};
use serde::Serialize;
use std::path::PathBuf;

type GuiResult<T> = Result<T, String>;

#[derive(Debug, Serialize)]
struct BackendStatus {
    app: &'static str,
    version: &'static str,
    service: &'static str,
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
fn inspect_vault(path: String) -> GuiResult<VaultInfo> {
    AnahtarService::inspect(path).map_err(safe_error)
}

#[tauri::command]
fn unlock_vault(
    path: String,
    password: String,
    key_file: Option<String>,
) -> GuiResult<Vec<EntrySummary>> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::list(path, &credentials).map_err(safe_error)
}

#[tauri::command]
fn search_entries(
    path: String,
    password: String,
    key_file: Option<String>,
    query: String,
) -> GuiResult<Vec<EntrySummary>> {
    let credentials = credentials_from_gui(password, key_file);
    AnahtarService::search(path, &credentials, &query).map_err(safe_error)
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
    AnahtarService::show(path, &credentials, &selector, reveal_password).map_err(safe_error)
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
        return Err("entry selector value is required".to_string());
    }

    match kind {
        "id" => Ok(EntrySelector::Id(value)),
        "title" => Ok(EntrySelector::Title(value)),
        "url" => Ok(EntrySelector::Url(value)),
        "username" => Ok(EntrySelector::Username(value)),
        "auto" => Ok(EntrySelector::Auto(value)),
        _ => Err("unknown entry selector kind".to_string()),
    }
}

fn safe_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            backend_status,
            inspect_vault,
            unlock_vault,
            search_entries,
            show_entry
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
            path,
            "testpass".to_string(),
            None,
            "title".to_string(),
            "Github Test".to_string(),
            false,
        )
        .unwrap();
        assert_eq!(detail.title.as_deref(), Some("Github Test"));
        assert!(detail.password.is_none());
    }

    #[test]
    fn wrong_password_returns_safe_error() {
        let path = generated_vault_path();
        if !std::path::Path::new(&path).exists() {
            return;
        }

        let err = unlock_vault(path, "wrong-password".to_string(), None).unwrap_err();
        assert!(err.contains("failed to open database"));
        assert!(!err.contains("wrong-password"));
    }
}
