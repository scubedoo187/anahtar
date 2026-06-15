use anahtar_app::{AnahtarService, WriteMode};
use anahtar_core::{
    AddEntryRequest, EditEntryRequest, EntryDetail, EntrySelector, EntrySummary, GroupSummary,
    TotpCode, VaultCredentials, WriteReport,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct FfiResponse<T: Serialize> {
    ok: bool,
    data: Option<T>,
    error: Option<FfiError>,
}

#[derive(Debug, Serialize)]
struct FfiError {
    kind: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct BackendStatus {
    status: &'static str,
    version: &'static str,
    service: &'static str,
}

#[derive(Debug, Deserialize)]
struct VaultRequest {
    path: String,
    password: String,
    key_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    query: String,
}

#[derive(Debug, Deserialize)]
struct ShowEntryRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    selector_kind: String,
    selector_value: String,
    reveal_password: bool,
}

#[derive(Debug, Deserialize)]
struct TotpRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    selector_kind: String,
    selector_value: String,
}

#[derive(Debug, Deserialize)]
struct AddEntryInput {
    group_path: String,
    title: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
}

impl From<AddEntryInput> for AddEntryRequest {
    fn from(value: AddEntryInput) -> Self {
        Self {
            group_path: value.group_path,
            title: value.title,
            username: value.username,
            password: value.password,
            url: value.url,
            notes: value.notes,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EditEntryInput {
    title: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
}

impl From<EditEntryInput> for EditEntryRequest {
    fn from(value: EditEntryInput) -> Self {
        Self {
            title: value.title,
            username: value.username,
            password: value.password,
            url: value.url,
            notes: value.notes,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AddEntryFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    entry: AddEntryInput,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EditEntryFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    entry: EditEntryInput,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EntryIdFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GroupFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    group_path: String,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenameGroupFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    group_path: String,
    new_name: String,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoveEntryFfiRequest {
    path: String,
    password: String,
    key_file: Option<String>,
    entry_id: String,
    group_path: String,
    backup_dir: Option<String>,
}

#[no_mangle]
pub extern "C" fn anahtar_backend_status_json() -> *mut c_char {
    json_response(BackendStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        service: "anahtar-app",
    })
}

#[no_mangle]
pub extern "C" fn anahtar_unlock_vault_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: VaultRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::list(&request.path, &credentials)
            .map_err(|error| ffi_error("unlock_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_search_entries_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: SearchRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::search(&request.path, &credentials, &request.query)
            .map_err(|error| ffi_error("read_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_show_entry_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: ShowEntryRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        let selector = entry_selector(&request.selector_kind, request.selector_value);
        AnahtarService::show(
            &request.path,
            &credentials,
            &selector,
            request.reveal_password,
        )
        .map_err(|error| ffi_error("read_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_totp_code_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: TotpRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        let selector = entry_selector(&request.selector_kind, request.selector_value);
        AnahtarService::totp(&request.path, &credentials, &selector)
            .map_err(|error| ffi_error("totp_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_audit_vault_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: VaultRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::audit(&request.path, &credentials)
            .map_err(|error| ffi_error("audit_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_add_entry_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: AddEntryFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::add_entry(
            &request.path,
            &credentials,
            request.entry.into(),
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_edit_entry_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: EditEntryFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::edit_entry(
            &request.path,
            &credentials,
            &request.entry_id,
            request.entry.into(),
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_delete_entry_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: EntryIdFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::delete_entry(
            &request.path,
            &credentials,
            &request.entry_id,
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_add_group_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: GroupFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::add_group(
            &request.path,
            &credentials,
            &request.group_path,
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_rename_group_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: RenameGroupFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::rename_group(
            &request.path,
            &credentials,
            &request.group_path,
            &request.new_name,
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_delete_group_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: GroupFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::delete_group(
            &request.path,
            &credentials,
            &request.group_path,
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_move_entry_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: MoveEntryFfiRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::move_entry(
            &request.path,
            &credentials,
            &EntrySelector::Id(request.entry_id),
            &request.group_path,
            in_place(request.backup_dir),
        )
        .and_then(required_report)
        .map_err(|error| ffi_error("write_failed", error))
    })
}

#[no_mangle]
pub extern "C" fn anahtar_list_groups_json(request_json: *const c_char) -> *mut c_char {
    request_response(request_json, |request: VaultRequest| {
        let credentials = credentials(&request.password, request.key_file.as_deref());
        AnahtarService::groups(&request.path, &credentials)
            .map_err(|error| ffi_error("group_failed", error))
    })
}

/// Frees a string allocated by Anahtar FFI functions.
///
/// # Safety
///
/// `ptr` must be either null or a pointer returned by an Anahtar FFI function
/// that transfers ownership to the caller. It must not be freed more than once.
#[no_mangle]
pub unsafe extern "C" fn anahtar_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

fn request_response<Request, Response, F>(request_json: *const c_char, handler: F) -> *mut c_char
where
    Request: DeserializeOwned,
    Response: Serialize,
    F: FnOnce(Request) -> Result<Response, FfiError>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Response, FfiError> {
        let request = parse_request::<Request>(request_json)?;
        handler(request)
    }));

    match result {
        Ok(Ok(data)) => json_response(data),
        Ok(Err(error)) => json_error(error),
        Err(_) => json_error(FfiError {
            kind: "panic",
            message: "The Rust backend stopped unexpectedly.".to_string(),
        }),
    }
}

fn parse_request<T: DeserializeOwned>(request_json: *const c_char) -> Result<T, FfiError> {
    if request_json.is_null() {
        return Err(FfiError {
            kind: "validation_failed",
            message: "request is required".to_string(),
        });
    }
    let json = unsafe { CStr::from_ptr(request_json) }
        .to_str()
        .map_err(|_| FfiError {
            kind: "validation_failed",
            message: "request must be valid UTF-8".to_string(),
        })?;
    serde_json::from_str(json).map_err(|error| FfiError {
        kind: "validation_failed",
        message: error.to_string(),
    })
}

fn credentials(password: &str, key_file: Option<&str>) -> VaultCredentials {
    match key_file.filter(|value| !value.trim().is_empty()) {
        Some(key_file) => VaultCredentials::with_key_file(password.to_string(), key_file),
        None => VaultCredentials::password_only(password.to_string()),
    }
}

fn in_place(backup_dir: Option<String>) -> WriteMode {
    WriteMode::InPlace {
        backup_dir: backup_dir
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from),
    }
}

fn required_report(report: Option<WriteReport>) -> anahtar_core::Result<WriteReport> {
    report.ok_or_else(|| {
        anahtar_core::AnahtarError::VerificationFailed("write returned no report".to_string())
    })
}

fn entry_selector(kind: &str, value: String) -> EntrySelector {
    match kind {
        "id" => EntrySelector::Id(value),
        "title" => EntrySelector::Title(value),
        "url" => EntrySelector::Url(value),
        "username" => EntrySelector::Username(value),
        _ => EntrySelector::Auto(value),
    }
}

fn ffi_error(kind: &'static str, error: impl std::fmt::Display) -> FfiError {
    FfiError {
        kind,
        message: error.to_string(),
    }
}

fn json_response<T: Serialize>(data: T) -> *mut c_char {
    let response = FfiResponse {
        ok: true,
        data: Some(data),
        error: None,
    };
    json_to_c_string(&response)
}

fn json_error(error: FfiError) -> *mut c_char {
    let response: FfiResponse<()> = FfiResponse {
        ok: false,
        data: None,
        error: Some(error),
    };
    json_to_c_string(&response)
}

fn json_to_c_string<T: Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(json) => CString::new(json)
            .unwrap_or_else(|_| {
                CString::new(internal_error_json("nul byte in json response")).unwrap()
            })
            .into_raw(),
        Err(error) => CString::new(internal_error_json(&error.to_string()))
            .expect("static error json must not contain nul")
            .into_raw(),
    }
}

fn internal_error_json(message: &str) -> String {
    let escaped = message.replace('"', "\\\"");
    format!(
        r#"{{"ok":false,"data":null,"error":{{"kind":"serialization_failed","message":"{escaped}"}}}}"#
    )
}

#[allow(dead_code)]
fn _assert_dtos_are_serializable(_: EntrySummary, _: EntryDetail, _: GroupSummary, _: TotpCode) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn backend_status_returns_json() {
        let ptr = anahtar_backend_status_json();
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { anahtar_string_free(ptr) };

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["status"], "ok");
        assert_eq!(value["data"]["service"], "anahtar-app");
    }

    #[test]
    fn unlock_vault_returns_entries() {
        let vault = test_vault_path();
        let request = CString::new(format!(
            r#"{{"path":"{}","password":"testpass","key_file":null}}"#,
            vault.display()
        ))
        .unwrap();
        let ptr = anahtar_unlock_vault_json(request.as_ptr());
        let json = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { anahtar_string_free(ptr) };

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["ok"], true);
        assert!(value["data"].as_array().unwrap().len() >= 4);
    }

    #[test]
    fn wrong_password_returns_safe_error() {
        let vault = test_vault_path();
        let request = CString::new(format!(
            r#"{{"path":"{}","password":"wrong","key_file":null}}"#,
            vault.display()
        ))
        .unwrap();
        let ptr = anahtar_unlock_vault_json(request.as_ptr());
        let json = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { anahtar_string_free(ptr) };

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["kind"], "unlock_failed");
        assert!(!json.contains("wrong"));
    }

    fn test_vault_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("test-vaults/generated/phase3-base.kdbx")
    }
}
