//! Core KDBX operations for Anahtar.
//!
//! This crate intentionally exposes safe output structures for CLI/GUI use.
//! Password fields are not included in summaries and are only returned by
//! explicit detail requests with `reveal_password = true`.

use keepass::{
    config::DatabaseVersion,
    db::{fields, Database, EntryRef, GroupRef},
    DatabaseKey,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::Uuid;

const KEEPASS_FILE_SIGNATURE: u32 = 0x9AA2_D903;
const KDBX_SIGNATURE: u32 = 0xB54B_FB67;
const KDB_SIGNATURE: u32 = 0xB54B_FB65;

#[derive(Debug, Error)]
pub enum AnahtarError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file is too short to be a KDBX database")]
    HeaderTooShort,
    #[error("unsupported or unknown KeePass signature: {signature1:#x} {signature2:#x}")]
    UnknownSignature { signature1: u32, signature2: u32 },
    #[error("failed to open database; check password/key material or file integrity")]
    OpenDatabase,
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("entry selector matches multiple entries; use UUID: {0}")]
    DuplicateEntrySelector(String),
    #[error("TOTP is not available for entry: {0}")]
    TotpUnavailable(String),
    #[error("group not found: {0}")]
    GroupNotFound(String),
    #[error("invalid group path: {0}")]
    InvalidGroupPath(String),
    #[error("input and output paths resolve to the same file: {0}")]
    InputOutputSame(PathBuf),
    #[error("output already exists: {0}")]
    OutputExists(PathBuf),
    #[error("temporary output already exists; remove it before retrying: {0}")]
    TempOutputExists(PathBuf),
    #[error("failed to save database")]
    SaveDatabase,
    #[error("save verification failed: {0}")]
    VerificationFailed(String),
    #[error("count mismatch after save: groups {input_groups}->{output_groups}, entries {input_entries}->{output_entries}")]
    CountMismatch {
        input_groups: usize,
        output_groups: usize,
        input_entries: usize,
        output_entries: usize,
    },
}

pub type Result<T> = std::result::Result<T, AnahtarError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KdbxVersion {
    Kdb1 { minor: u16, major: u16 },
    Kdbx { major: u16, minor: u16 },
}

impl std::fmt::Display for KdbxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KdbxVersion::Kdb1 { major, minor } => write!(f, "KDB {major}.{minor}"),
            KdbxVersion::Kdbx { major, minor } => write!(f, "KDBX {major}.{minor}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VaultCredentials {
    pub password: String,
    pub key_file: Option<PathBuf>,
}

impl VaultCredentials {
    pub fn password_only(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
            key_file: None,
        }
    }

    pub fn with_key_file(password: impl Into<String>, key_file: impl Into<PathBuf>) -> Self {
        Self {
            password: password.into(),
            key_file: Some(key_file.into()),
        }
    }

    fn to_database_key(&self) -> Result<DatabaseKey> {
        let mut key = DatabaseKey::new().with_password(&self.password);
        if let Some(path) = &self.key_file {
            let mut file = File::open(path)?;
            key = key
                .with_keyfile(&mut file)
                .map_err(|_| AnahtarError::OpenDatabase)?;
        }
        Ok(key)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultInfo {
    pub path: PathBuf,
    pub file_size_bytes: u64,
    pub version: KdbxVersion,
}

#[derive(Debug, Clone)]
pub enum EntrySelector {
    Id(String),
    Title(String),
    Url(String),
    Username(String),
    Auto(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct EntrySummary {
    pub id: String,
    pub group_path: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupSummary {
    pub id: String,
    pub path: String,
    pub name: String,
    pub entry_count: usize,
    pub child_group_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryDetail {
    pub id: String,
    pub group_path: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub has_totp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub custom_fields: Vec<CustomField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomField {
    pub key: String,
    pub value: String,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditFinding {
    pub kind: String,
    pub entry_id: String,
    pub title: Option<String>,
    pub group_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TotpCode {
    pub code: String,
    pub valid_for_seconds: u64,
    pub period_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpgradeReport {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_version: KdbxVersion,
    pub output_version: KdbxVersion,
    pub input_group_count: usize,
    pub input_entry_count: usize,
    pub output_group_count: Option<usize>,
    pub output_entry_count: Option<usize>,
    pub dry_run: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SaveAsOptions {
    pub output_path: PathBuf,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct InPlaceOptions {
    pub target_path: PathBuf,
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct AddEntryRequest {
    pub group_path: String,
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EditEntryRequest {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteOperation {
    Upgrade,
    Add,
    Edit,
    Delete,
    GroupAdd,
    GroupRename,
    GroupDelete,
    MoveEntry,
}

#[derive(Debug, Clone, Serialize)]
pub struct WriteReport {
    pub operation: WriteOperation,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_version: KdbxVersion,
    pub output_version: KdbxVersion,
    pub input_group_count: usize,
    pub input_entry_count: usize,
    pub output_group_count: usize,
    pub output_entry_count: usize,
    pub changed_entry_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_target_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
struct SaveAsVerification {
    expected_group_count: usize,
    expected_entry_count: usize,
}

pub fn inspect_header(path: impl AsRef<Path>) -> Result<VaultInfo> {
    let path = path.as_ref();
    let mut f = File::open(path)?;
    let mut header = [0u8; 12];
    f.read_exact(&mut header).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            AnahtarError::HeaderTooShort
        } else {
            e.into()
        }
    })?;

    let sig1 = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let sig2 = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    let minor = u16::from_le_bytes([header[8], header[9]]);
    let major = u16::from_le_bytes([header[10], header[11]]);

    let version = match (sig1, sig2) {
        (KEEPASS_FILE_SIGNATURE, KDBX_SIGNATURE) => KdbxVersion::Kdbx { major, minor },
        (KEEPASS_FILE_SIGNATURE, KDB_SIGNATURE) => KdbxVersion::Kdb1 { major, minor },
        _ => {
            return Err(AnahtarError::UnknownSignature {
                signature1: sig1,
                signature2: sig2,
            })
        }
    };

    Ok(VaultInfo {
        path: path.to_path_buf(),
        file_size_bytes: path.metadata()?.len(),
        version,
    })
}

pub fn open_database(path: impl AsRef<Path>, password: &str) -> Result<Database> {
    open_database_with_credentials(path, &VaultCredentials::password_only(password))
}

pub fn open_database_with_credentials(
    path: impl AsRef<Path>,
    credentials: &VaultCredentials,
) -> Result<Database> {
    let mut f = File::open(path)?;
    Database::open(&mut f, credentials.to_database_key()?).map_err(|_| AnahtarError::OpenDatabase)
}

pub fn database_version(db: &Database) -> KdbxVersion {
    match db.config.version {
        DatabaseVersion::KDB(minor) => KdbxVersion::Kdb1 { major: 1, minor },
        DatabaseVersion::KDB2(minor) => KdbxVersion::Kdbx { major: 2, minor },
        DatabaseVersion::KDB3(minor) => KdbxVersion::Kdbx { major: 3, minor },
        DatabaseVersion::KDB4(minor) => KdbxVersion::Kdbx { major: 4, minor },
    }
}

pub fn list_entries(db: &Database) -> Vec<EntrySummary> {
    let mut out = Vec::new();
    collect_summaries(db.root(), db.root().name.clone(), &mut out);
    out.sort_by(|a, b| a.group_path.cmp(&b.group_path).then(a.title.cmp(&b.title)));
    out
}

pub fn audit_database(db: &Database) -> AuditReport {
    let mut findings = Vec::new();
    let mut password_groups: HashMap<String, Vec<EntrySummary>> = HashMap::new();
    for entry in list_entries(db) {
        if entry.username.as_deref().unwrap_or("").is_empty() {
            findings.push(AuditFinding {
                kind: "missing_username".to_string(),
                entry_id: entry.id.clone(),
                title: entry.title.clone(),
                group_path: entry.group_path.clone(),
                message: "entry has no username".to_string(),
            });
        }
        if entry.url.as_deref().unwrap_or("").is_empty() {
            findings.push(AuditFinding {
                kind: "missing_url".to_string(),
                entry_id: entry.id.clone(),
                title: entry.title.clone(),
                group_path: entry.group_path.clone(),
                message: "entry has no url".to_string(),
            });
        }
        if let Some(detail) = find_entry(db, &entry.id) {
            if detail.get_otp().is_ok() {
                findings.push(AuditFinding {
                    kind: "totp_available".to_string(),
                    entry_id: entry.id.clone(),
                    title: entry.title.clone(),
                    group_path: entry.group_path.clone(),
                    message: "entry has TOTP configured".to_string(),
                });
            }
            if let Some(password) = detail.get_password() {
                password_groups
                    .entry(password.to_string())
                    .or_default()
                    .push(entry.clone());
                if password.len() < 12 {
                    findings.push(AuditFinding {
                        kind: "weak_password".to_string(),
                        entry_id: entry.id.clone(),
                        title: entry.title.clone(),
                        group_path: entry.group_path.clone(),
                        message: "password is shorter than 12 characters".to_string(),
                    });
                }
            }
        }
    }
    for entries in password_groups.values().filter(|entries| entries.len() > 1) {
        for entry in entries {
            findings.push(AuditFinding {
                kind: "reused_password".to_string(),
                entry_id: entry.id.clone(),
                title: entry.title.clone(),
                group_path: entry.group_path.clone(),
                message: format!("password is reused by {} entries", entries.len()),
            });
        }
    }
    AuditReport { findings }
}

pub fn search_entries(db: &Database, query: &str) -> Vec<EntrySummary> {
    let needle = query.to_lowercase();
    list_entries(db)
        .into_iter()
        .filter(|e| {
            [&e.title, &e.username, &e.url]
                .into_iter()
                .flatten()
                .any(|v| v.to_lowercase().contains(&needle))
                || entry_notes_by_id(db, &e.id).is_some_and(|n| n.to_lowercase().contains(&needle))
        })
        .collect()
}

pub fn show_entry(db: &Database, selector: &str, reveal_password: bool) -> Result<EntryDetail> {
    show_entry_by_selector(
        db,
        &EntrySelector::Auto(selector.to_string()),
        reveal_password,
    )
}

pub fn show_entry_by_selector(
    db: &Database,
    selector: &EntrySelector,
    reveal_password: bool,
) -> Result<EntryDetail> {
    let summary = resolve_entry_summary_by_selector(db, selector)?;
    let entry = find_entry(db, &summary.id)
        .ok_or_else(|| AnahtarError::EntryNotFound(selector_label(selector)))?;
    Ok(detail_from_entry(
        entry,
        summary.group_path,
        reveal_password,
    ))
}

pub fn totp_code(db: &Database, selector: &str) -> Result<TotpCode> {
    totp_code_by_selector(db, &EntrySelector::Auto(selector.to_string()))
}

pub fn totp_code_by_selector(db: &Database, selector: &EntrySelector) -> Result<TotpCode> {
    let summary = resolve_entry_summary_by_selector(db, selector)?;
    let entry = find_entry(db, &summary.id)
        .ok_or_else(|| AnahtarError::EntryNotFound(selector_label(selector)))?;
    let otp = entry
        .get_otp()
        .map_err(|_| AnahtarError::TotpUnavailable(selector_label(selector)))?;
    let code = otp
        .value_now()
        .map_err(|_| AnahtarError::TotpUnavailable(selector_label(selector)))?;
    Ok(TotpCode {
        code: code.code,
        valid_for_seconds: code.valid_for.as_secs(),
        period_seconds: code.period.as_secs(),
    })
}

pub fn upgrade_to_kdbx41(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    password: &str,
    force: bool,
    dry_run: bool,
) -> Result<UpgradeReport> {
    upgrade_to_kdbx41_with_credentials(
        input,
        output,
        &VaultCredentials::password_only(password),
        force,
        dry_run,
    )
}

pub fn upgrade_to_kdbx41_with_credentials(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    credentials: &VaultCredentials,
    force: bool,
    dry_run: bool,
) -> Result<UpgradeReport> {
    let input = input.as_ref();
    let output = output.as_ref();

    if canonical_for_compare(input)? == canonical_for_compare(output)? {
        return Err(AnahtarError::InputOutputSame(output.to_path_buf()));
    }

    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let warning = (input_version != (KdbxVersion::Kdbx { major: 4, minor: 1 }))
        .then(|| format!("input is {input_version}; output will be written as KDBX 4.1"));

    let mut report = UpgradeReport {
        input_path: input.to_path_buf(),
        output_path: output.to_path_buf(),
        input_version,
        output_version: KdbxVersion::Kdbx { major: 4, minor: 1 },
        input_group_count,
        input_entry_count,
        output_group_count: None,
        output_entry_count: None,
        dry_run,
        warning,
    };

    if dry_run {
        return Ok(report);
    }

    if output.exists() && !force {
        return Err(AnahtarError::OutputExists(output.to_path_buf()));
    }

    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        output,
        credentials,
        force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count,
        },
        |_| Ok(()),
    )?;

    report.output_group_count = Some(output_group_count);
    report.output_entry_count = Some(output_entry_count);
    Ok(report)
}

pub fn add_entry_save_as(
    input: impl AsRef<Path>,
    password: &str,
    options: SaveAsOptions,
    request: AddEntryRequest,
) -> Result<WriteReport> {
    add_entry_save_as_with_credentials(
        input,
        &VaultCredentials::password_only(password),
        options,
        request,
    )
}

pub fn add_entry_save_as_with_credentials(
    input: impl AsRef<Path>,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    request: AddEntryRequest,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();

    let group_id = group_id_by_path(&db, &request.group_path)?;
    let changed_entry_id = {
        let mut group = db
            .group_mut(group_id)
            .ok_or_else(|| AnahtarError::GroupNotFound(request.group_path.clone()))?;
        let mut entry = group.add_entry();
        entry.set_unprotected(fields::TITLE, request.title);
        if let Some(username) = request.username {
            entry.set_unprotected(fields::USERNAME, username);
        }
        if let Some(entry_password) = request.password {
            entry.set_protected(fields::PASSWORD, entry_password);
        }
        if let Some(url) = request.url {
            entry.set_unprotected(fields::URL, url);
        }
        if let Some(notes) = request.notes {
            entry.set_unprotected(fields::NOTES, notes);
        }
        entry.id().uuid().to_string()
    };

    let output_path = options.output_path;
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count + 1,
        },
        |saved| {
            let found = find_entry(saved, &changed_entry_id).is_some();
            found
                .then_some(())
                .ok_or_else(|| AnahtarError::EntryNotFound(changed_entry_id.clone()))
        },
    )?;

    Ok(WriteReport {
        operation: WriteOperation::Add,
        input_path: input.to_path_buf(),
        output_path,
        input_version,
        output_version: KdbxVersion::Kdbx { major: 4, minor: 1 },
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        changed_entry_id: Some(changed_entry_id),
        backup_path: None,
        final_target_path: None,
    })
}

pub fn edit_entry_save_as(
    input: impl AsRef<Path>,
    selector: &str,
    password: &str,
    options: SaveAsOptions,
    request: EditEntryRequest,
) -> Result<WriteReport> {
    edit_entry_save_as_with_credentials(
        input,
        selector,
        &VaultCredentials::password_only(password),
        options,
        request,
    )
}

pub fn edit_entry_save_as_with_credentials(
    input: impl AsRef<Path>,
    selector: &str,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    request: EditEntryRequest,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let entry_id = resolve_unique_entry_id(&db, selector)?;
    let changed_entry_id = entry_id.uuid().to_string();

    {
        let mut entry = db
            .entry_mut(entry_id)
            .ok_or_else(|| AnahtarError::EntryNotFound(selector.to_string()))?;
        if let Some(title) = request.title {
            entry.set_unprotected(fields::TITLE, title);
        }
        if let Some(username) = request.username {
            entry.set_unprotected(fields::USERNAME, username);
        }
        if let Some(entry_password) = request.password {
            entry.set_protected(fields::PASSWORD, entry_password);
        }
        if let Some(url) = request.url {
            entry.set_unprotected(fields::URL, url);
        }
        if let Some(notes) = request.notes {
            entry.set_unprotected(fields::NOTES, notes);
        }
    }

    let output_path = options.output_path;
    let changed_entry_id_for_verify = changed_entry_id.clone();
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count,
        },
        move |saved| {
            find_entry(saved, &changed_entry_id_for_verify)
                .map(|_| ())
                .ok_or_else(|| AnahtarError::EntryNotFound(changed_entry_id_for_verify.clone()))
        },
    )?;

    Ok(WriteReport {
        operation: WriteOperation::Edit,
        input_path: input.to_path_buf(),
        output_path,
        input_version,
        output_version: KdbxVersion::Kdbx { major: 4, minor: 1 },
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        changed_entry_id: Some(changed_entry_id),
        backup_path: None,
        final_target_path: None,
    })
}

pub fn delete_entry_save_as(
    input: impl AsRef<Path>,
    entry_id: &str,
    password: &str,
    options: SaveAsOptions,
) -> Result<WriteReport> {
    delete_entry_save_as_with_credentials(
        input,
        entry_id,
        &VaultCredentials::password_only(password),
        options,
    )
}

pub fn delete_entry_save_as_with_credentials(
    input: impl AsRef<Path>,
    entry_id: &str,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let uuid =
        Uuid::parse_str(entry_id).map_err(|_| AnahtarError::EntryNotFound(entry_id.to_string()))?;
    let entry_id = keepass::db::EntryId::from_uuid(uuid);
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    if db.entry(entry_id).is_none() {
        return Err(AnahtarError::EntryNotFound(uuid.to_string()));
    }

    db.entry_mut(entry_id)
        .ok_or_else(|| AnahtarError::EntryNotFound(uuid.to_string()))?
        .remove();

    let changed_entry_id = uuid.to_string();
    let output_path = options.output_path;
    let changed_entry_id_for_verify = changed_entry_id.clone();
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count.saturating_sub(1),
        },
        move |saved| {
            if find_entry(saved, &changed_entry_id_for_verify).is_some() {
                Err(AnahtarError::VerificationFailed(format!(
                    "deleted entry still present: {changed_entry_id_for_verify}"
                )))
            } else {
                Ok(())
            }
        },
    )?;

    Ok(WriteReport {
        operation: WriteOperation::Delete,
        input_path: input.to_path_buf(),
        output_path,
        input_version,
        output_version: KdbxVersion::Kdbx { major: 4, minor: 1 },
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        changed_entry_id: Some(changed_entry_id),
        backup_path: None,
        final_target_path: None,
    })
}

pub fn list_groups(db: &Database) -> Vec<GroupSummary> {
    let mut out = Vec::new();
    collect_group_summaries(db.root(), db.root().name.clone(), &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

pub fn add_group_save_as_with_credentials(
    input: impl AsRef<Path>,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    group_path: &str,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let (parent_path, name) = split_group_parent(group_path)?;
    let parent_id = if parent_path.is_empty() {
        db.root().id()
    } else {
        group_id_by_path(&db, parent_path)?
    };
    {
        let parent = db
            .group(parent_id)
            .ok_or_else(|| AnahtarError::GroupNotFound(parent_path.to_string()))?;
        if parent.group_by_name(name).is_some() {
            return Err(AnahtarError::InvalidGroupPath(format!(
                "group already exists: {group_path}"
            )));
        }
    }
    let changed_entry_id = {
        let mut parent = db
            .group_mut(parent_id)
            .ok_or_else(|| AnahtarError::GroupNotFound(parent_path.to_string()))?;
        let mut group = parent.add_group();
        group.edit(|g| g.name = name.to_string());
        group.id().uuid().to_string()
    };
    let output_path = options.output_path;
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count + 1,
            expected_entry_count: input_entry_count,
        },
        |_| Ok(()),
    )?;
    Ok(write_report(
        WriteOperation::GroupAdd,
        input,
        output_path,
        input_version,
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        Some(changed_entry_id),
    ))
}

pub fn rename_group_save_as_with_credentials(
    input: impl AsRef<Path>,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    group_path: &str,
    new_name: &str,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let group_id = group_id_by_path(&db, group_path)?;
    db.group_mut(group_id)
        .ok_or_else(|| AnahtarError::GroupNotFound(group_path.to_string()))?
        .edit(|g| g.name = new_name.to_string());
    let output_path = options.output_path;
    let changed_entry_id = group_id.uuid().to_string();
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count,
        },
        |_| Ok(()),
    )?;
    Ok(write_report(
        WriteOperation::GroupRename,
        input,
        output_path,
        input_version,
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        Some(changed_entry_id),
    ))
}

pub fn delete_group_save_as_with_credentials(
    input: impl AsRef<Path>,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    group_path: &str,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let group_id = group_id_by_path(&db, group_path)?;
    let removed_entries = db.group(group_id).map(|g| g.entries().count()).unwrap_or(0);
    db.group_mut(group_id)
        .ok_or_else(|| AnahtarError::GroupNotFound(group_path.to_string()))?
        .remove();
    let output_path = options.output_path;
    let changed_entry_id = group_id.uuid().to_string();
    let expected_groups = input_group_count.saturating_sub(1);
    let expected_entries = input_entry_count.saturating_sub(removed_entries);
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: expected_groups,
            expected_entry_count: expected_entries,
        },
        |_| Ok(()),
    )?;
    Ok(write_report(
        WriteOperation::GroupDelete,
        input,
        output_path,
        input_version,
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        Some(changed_entry_id),
    ))
}

pub fn move_entry_save_as_with_credentials(
    input: impl AsRef<Path>,
    credentials: &VaultCredentials,
    options: SaveAsOptions,
    selector: &EntrySelector,
    group_path: &str,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database_with_credentials(input, credentials)?;
    let input_version = database_version(&db);
    let input_group_count = count_groups(&db);
    let input_entry_count = db.iter_all_entries().count();
    let entry_id = resolve_unique_entry_id_by_selector(&db, selector)?;
    let group_id = group_id_by_path(&db, group_path)?;
    db.entry_mut(entry_id)
        .ok_or_else(|| AnahtarError::EntryNotFound(selector_label(selector)))?
        .move_to(group_id)
        .map_err(|_| AnahtarError::GroupNotFound(group_path.to_string()))?;
    let output_path = options.output_path;
    let changed_entry_id = entry_id.uuid().to_string();
    let (output_group_count, output_entry_count) = save_as_kdbx41_verified(
        &mut db,
        input,
        &output_path,
        credentials,
        options.force,
        SaveAsVerification {
            expected_group_count: input_group_count,
            expected_entry_count: input_entry_count,
        },
        |_| Ok(()),
    )?;
    Ok(write_report(
        WriteOperation::MoveEntry,
        input,
        output_path,
        input_version,
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        Some(changed_entry_id),
    ))
}

pub fn safe_in_place_write_without_backup_with_credentials<F>(
    credentials: &VaultCredentials,
    options: InPlaceOptions,
    save_as: F,
) -> Result<WriteReport>
where
    F: FnOnce(&Path, &Path) -> Result<WriteReport>,
{
    let target = options.target_path;
    let tmp = temp_in_place_path_for(&target)?;

    if tmp.exists() {
        return Err(AnahtarError::TempOutputExists(tmp));
    }

    let write_result = (|| -> Result<WriteReport> {
        let mut report = save_as(&target, &tmp)?;
        let verified = open_database_with_credentials(&tmp, credentials)?;
        if count_groups(&verified) != report.output_group_count
            || verified.iter_all_entries().count() != report.output_entry_count
        {
            return Err(AnahtarError::VerificationFailed(
                "temporary in-place verification count mismatch".to_string(),
            ));
        }

        replace_target_with_tmp_without_backup(&tmp, &target)?;

        let final_verified = open_database_with_credentials(&target, credentials)?;
        if count_groups(&final_verified) != report.output_group_count
            || final_verified.iter_all_entries().count() != report.output_entry_count
        {
            return Err(AnahtarError::VerificationFailed(
                "final in-place verification count mismatch".to_string(),
            ));
        }

        report.output_path = target.clone();
        report.backup_path = None;
        report.final_target_path = Some(target.clone());
        Ok(report)
    })();

    match write_result {
        Ok(report) => Ok(report),
        Err(err) => {
            let _ = std::fs::remove_file(&tmp);
            Err(err)
        }
    }
}

pub fn safe_in_place_write_with_credentials<F>(
    credentials: &VaultCredentials,
    options: InPlaceOptions,
    save_as: F,
) -> Result<WriteReport>
where
    F: FnOnce(&Path, &Path) -> Result<WriteReport>,
{
    let target = options.target_path;
    let backup = backup_path_for(&target, options.backup_dir.as_deref())?;
    let tmp = temp_in_place_path_for(&target)?;

    if tmp.exists() {
        return Err(AnahtarError::TempOutputExists(tmp));
    }

    if let Some(parent) = backup.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&target, &backup)?;

    let write_result = (|| -> Result<WriteReport> {
        let mut report = save_as(&target, &tmp)?;
        let verified = open_database_with_credentials(&tmp, credentials)?;
        if count_groups(&verified) != report.output_group_count
            || verified.iter_all_entries().count() != report.output_entry_count
        {
            return Err(AnahtarError::VerificationFailed(
                "temporary in-place verification count mismatch".to_string(),
            ));
        }

        replace_target_with_tmp(&tmp, &target, &backup)?;

        let final_verified = match open_database_with_credentials(&target, credentials) {
            Ok(db) => db,
            Err(err) => {
                let _ = restore_backup(&backup, &target);
                return Err(err);
            }
        };
        if count_groups(&final_verified) != report.output_group_count
            || final_verified.iter_all_entries().count() != report.output_entry_count
        {
            let _ = restore_backup(&backup, &target);
            return Err(AnahtarError::VerificationFailed(
                "final in-place verification count mismatch; restored target from backup"
                    .to_string(),
            ));
        }

        report.output_path = target.clone();
        report.backup_path = Some(backup.clone());
        report.final_target_path = Some(target.clone());
        Ok(report)
    })();

    match write_result {
        Ok(report) => Ok(report),
        Err(err) => {
            let _ = std::fs::remove_file(&tmp);
            Err(err)
        }
    }
}

fn replace_target_with_tmp_without_backup(tmp: &Path, target: &Path) -> Result<()> {
    match std::fs::rename(tmp, target) {
        Ok(()) => Ok(()),
        Err(err) => {
            #[cfg(windows)]
            {
                if target.exists() {
                    std::fs::remove_file(target)?;
                    std::fs::rename(tmp, target)?;
                    return Ok(());
                }
            }
            Err(err.into())
        }
    }
}

fn replace_target_with_tmp(tmp: &Path, target: &Path, _backup: &Path) -> Result<()> {
    match std::fs::rename(tmp, target) {
        Ok(()) => Ok(()),
        Err(err) => {
            #[cfg(windows)]
            {
                if target.exists() {
                    std::fs::remove_file(target)?;
                    if let Err(rename_err) = std::fs::rename(tmp, target) {
                        let _ = restore_backup(_backup, target);
                        return Err(rename_err.into());
                    }
                    return Ok(());
                }
            }
            Err(err.into())
        }
    }
}

fn restore_backup(backup: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::copy(backup, target).map(|_| ())
}

pub fn count_groups(db: &Database) -> usize {
    fn walk(group: GroupRef<'_>) -> usize {
        1 + group.groups().map(walk).sum::<usize>()
    }
    walk(db.root())
}

fn save_as_kdbx41_verified<F>(
    db: &mut Database,
    input: &Path,
    output: &Path,
    credentials: &VaultCredentials,
    force: bool,
    verification: SaveAsVerification,
    verify_saved: F,
) -> Result<(usize, usize)>
where
    F: FnOnce(&Database) -> Result<()>,
{
    ensure_input_output_distinct(input, output)?;

    if output.exists() && !force {
        return Err(AnahtarError::OutputExists(output.to_path_buf()));
    }

    db.config.version = DatabaseVersion::KDB4(1);
    let tmp = temp_path_for(output);
    if tmp.exists() {
        return Err(AnahtarError::TempOutputExists(tmp));
    }

    let write_result = (|| -> Result<(usize, usize)> {
        {
            let mut out = File::create(&tmp)?;
            db.save(&mut out, credentials.to_database_key()?)
                .map_err(|_| AnahtarError::SaveDatabase)?;
            out.flush()?;
            out.sync_all()?;
        }

        let verified = open_database_with_credentials(&tmp, credentials)?;
        let output_group_count = count_groups(&verified);
        let output_entry_count = verified.iter_all_entries().count();
        if output_group_count != verification.expected_group_count
            || output_entry_count != verification.expected_entry_count
        {
            return Err(AnahtarError::CountMismatch {
                input_groups: verification.expected_group_count,
                output_groups: output_group_count,
                input_entries: verification.expected_entry_count,
                output_entries: output_entry_count,
            });
        }
        verify_saved(&verified)?;

        if output.exists() && force {
            std::fs::remove_file(output)?;
        }
        std::fs::rename(&tmp, output)?;
        Ok((output_group_count, output_entry_count))
    })();

    match write_result {
        Ok(counts) => Ok(counts),
        Err(err) => {
            let _ = std::fs::remove_file(&tmp);
            Err(err)
        }
    }
}

fn ensure_input_output_distinct(input: &Path, output: &Path) -> Result<()> {
    if canonical_for_compare(input)? == canonical_for_compare(output)? {
        return Err(AnahtarError::InputOutputSame(output.to_path_buf()));
    }
    Ok(())
}

fn resolve_entry_summary_by_selector(
    db: &Database,
    selector: &EntrySelector,
) -> Result<EntrySummary> {
    let matches = list_entries(db)
        .into_iter()
        .filter(|e| entry_summary_matches(e, selector))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [summary] => Ok(summary.clone()),
        [] => Err(AnahtarError::EntryNotFound(selector_label(selector))),
        _ => Err(AnahtarError::DuplicateEntrySelector(format!(
            "{}; candidates: {}",
            selector_label(selector),
            matches
                .iter()
                .map(safe_candidate_summary)
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn resolve_unique_entry_id_by_selector(
    db: &Database,
    selector: &EntrySelector,
) -> Result<keepass::db::EntryId> {
    let summary = resolve_entry_summary_by_selector(db, selector)?;
    let uuid = Uuid::parse_str(&summary.id)
        .map_err(|_| AnahtarError::EntryNotFound(selector_label(selector)))?;
    Ok(keepass::db::EntryId::from_uuid(uuid))
}

fn resolve_unique_entry_id(db: &Database, selector: &str) -> Result<keepass::db::EntryId> {
    resolve_unique_entry_id_by_selector(db, &EntrySelector::Auto(selector.to_string()))
}

fn entry_summary_matches(entry: &EntrySummary, selector: &EntrySelector) -> bool {
    match selector {
        EntrySelector::Id(id) => entry.id.eq_ignore_ascii_case(id),
        EntrySelector::Title(title) => entry
            .title
            .as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case(title)),
        EntrySelector::Url(url) => entry
            .url
            .as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case(url) || v.contains(url)),
        EntrySelector::Username(username) => entry
            .username
            .as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case(username)),
        EntrySelector::Auto(value) => {
            entry.id.eq_ignore_ascii_case(value)
                || entry
                    .title
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case(value))
        }
    }
}

fn selector_label(selector: &EntrySelector) -> String {
    match selector {
        EntrySelector::Id(value) => format!("id={value}"),
        EntrySelector::Title(value) => format!("title={value}"),
        EntrySelector::Url(value) => format!("url={value}"),
        EntrySelector::Username(value) => format!("username={value}"),
        EntrySelector::Auto(value) => value.clone(),
    }
}

fn safe_candidate_summary(summary: &EntrySummary) -> String {
    format!(
        "id={} title={} username={} url={}",
        summary.id,
        summary.title.as_deref().unwrap_or(""),
        summary.username.as_deref().unwrap_or(""),
        summary.url.as_deref().unwrap_or("")
    )
}

fn group_id_by_path(db: &Database, group_path: &str) -> Result<keepass::db::GroupId> {
    if group_path.is_empty() || group_path.starts_with('/') || group_path.ends_with('/') {
        return Err(AnahtarError::InvalidGroupPath(group_path.to_string()));
    }
    let parts = group_path.split('/').collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(AnahtarError::InvalidGroupPath(group_path.to_string()));
    }
    if parts
        .first()
        .is_some_and(|part| part.eq_ignore_ascii_case(&db.root().name))
    {
        return Err(AnahtarError::InvalidGroupPath(group_path.to_string()));
    }

    db.root()
        .group_by_path(&parts)
        .map(|g| g.id())
        .ok_or_else(|| AnahtarError::GroupNotFound(group_path.to_string()))
}

fn backup_path_for(target: &Path, backup_dir: Option<&Path>) -> Result<PathBuf> {
    let backup_dir = backup_dir.map(Path::to_path_buf).unwrap_or_else(|| {
        target
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("anahtar-backups")
    });
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("vault");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            AnahtarError::VerificationFailed(format!("system clock before unix epoch: {err}"))
        })?
        .as_secs();

    for suffix in 0..1000 {
        let filename = if suffix == 0 {
            format!("{stem}.{timestamp}.kdbx")
        } else {
            format!("{stem}.{timestamp}.{suffix}.kdbx")
        };
        let candidate = backup_dir.join(filename);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(AnahtarError::TempOutputExists(
        backup_dir.join(format!("{stem}.{timestamp}.kdbx")),
    ))
}

fn temp_in_place_path_for(target: &Path) -> Result<PathBuf> {
    let file_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AnahtarError::InvalidGroupPath("vault path has no file name".to_string()))?;
    Ok(target.with_file_name(format!(".{file_name}.anahtar.tmp")))
}

fn temp_path_for(output: &Path) -> PathBuf {
    let mut tmp = output.to_path_buf();
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("kdbx");
    tmp.set_extension(format!("{ext}.tmp"));
    tmp
}

fn canonical_for_compare(path: &Path) -> std::io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }

    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
    })?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(parent.canonicalize()?.join(file_name))
}

#[allow(clippy::too_many_arguments)]
fn write_report(
    operation: WriteOperation,
    input: &Path,
    output_path: PathBuf,
    input_version: KdbxVersion,
    input_group_count: usize,
    input_entry_count: usize,
    output_group_count: usize,
    output_entry_count: usize,
    changed_entry_id: Option<String>,
) -> WriteReport {
    WriteReport {
        operation,
        input_path: input.to_path_buf(),
        output_path,
        input_version,
        output_version: KdbxVersion::Kdbx { major: 4, minor: 1 },
        input_group_count,
        input_entry_count,
        output_group_count,
        output_entry_count,
        changed_entry_id,
        backup_path: None,
        final_target_path: None,
    }
}

fn split_group_parent(group_path: &str) -> Result<(&str, &str)> {
    if group_path.is_empty() || group_path.starts_with('/') || group_path.ends_with('/') {
        return Err(AnahtarError::InvalidGroupPath(group_path.to_string()));
    }
    let (parent, name) = group_path.rsplit_once('/').unwrap_or(("", group_path));
    if name.is_empty() || parent.split('/').any(|p| p.is_empty()) && !parent.is_empty() {
        return Err(AnahtarError::InvalidGroupPath(group_path.to_string()));
    }
    Ok((parent, name))
}

fn collect_group_summaries(group: GroupRef<'_>, path: String, out: &mut Vec<GroupSummary>) {
    out.push(GroupSummary {
        id: group.id().uuid().to_string(),
        path: path.clone(),
        name: group.name.clone(),
        entry_count: group.entries().count(),
        child_group_count: group.groups().count(),
    });
    for child in group.groups() {
        let child_path = format!("{}/{}", path, child.name);
        collect_group_summaries(child, child_path, out);
    }
}

fn collect_summaries(group: GroupRef<'_>, path: String, out: &mut Vec<EntrySummary>) {
    for entry in group.entries() {
        out.push(summary_from_entry(entry, path.clone()));
    }
    for child in group.groups() {
        let child_path = if path.is_empty() {
            child.name.clone()
        } else {
            format!("{}/{}", path, child.name)
        };
        collect_summaries(child, child_path, out);
    }
}

fn summary_from_entry(entry: EntryRef<'_>, group_path: String) -> EntrySummary {
    EntrySummary {
        id: entry.id().uuid().to_string(),
        group_path,
        title: entry.get_title().map(ToOwned::to_owned),
        username: entry.get_username().map(ToOwned::to_owned),
        url: entry.get_url().map(ToOwned::to_owned),
    }
}

fn detail_from_entry(
    entry: EntryRef<'_>,
    group_path: String,
    reveal_password: bool,
) -> EntryDetail {
    let mut custom_fields = Vec::new();
    for (key, value) in &entry.fields {
        if matches!(
            key.as_str(),
            fields::TITLE | fields::USERNAME | fields::PASSWORD | fields::URL | fields::NOTES
        ) {
            continue;
        }
        let field_value = if value.is_protected() && !reveal_password {
            "<protected>".to_string()
        } else {
            value.as_str().to_string()
        };
        custom_fields.push(CustomField {
            key: key.clone(),
            value: field_value,
            protected: value.is_protected(),
        });
    }
    custom_fields.sort_by(|a, b| a.key.cmp(&b.key));

    EntryDetail {
        id: entry.id().uuid().to_string(),
        group_path,
        title: entry.get_title().map(ToOwned::to_owned),
        username: entry.get_username().map(ToOwned::to_owned),
        url: entry.get_url().map(ToOwned::to_owned),
        notes: entry.get(fields::NOTES).map(ToOwned::to_owned),
        has_totp: entry.get_otp().is_ok(),
        password: reveal_password
            .then(|| entry.get_password().map(ToOwned::to_owned))
            .flatten(),
        custom_fields,
    }
}

fn find_entry<'a>(db: &'a Database, selector: &str) -> Option<EntryRef<'a>> {
    let parsed_uuid = Uuid::parse_str(selector).ok();
    db.iter_all_entries().find(|e| {
        parsed_uuid.is_some_and(|u| e.id().uuid() == u)
            || e.get_title()
                .is_some_and(|t| t.eq_ignore_ascii_case(selector))
    })
}

fn entry_notes_by_id(db: &Database, id: &str) -> Option<String> {
    find_entry(db, id).and_then(|e| e.get(fields::NOTES).map(ToOwned::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;
    use keepass::db::Value;

    #[test]
    fn totp_code_does_not_expose_uri() {
        let mut db = Database::new();
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, "TOTP Example");
            e.set_protected(
                fields::OTP,
                "otpauth://totp/KeePassXC:none?secret=JBSWY3DPEHPK3PXP&period=30&digits=6&issuer=KeePassXC",
            );
        });

        let code = totp_code(&db, "TOTP Example").unwrap();
        assert_eq!(code.code.len(), 6);
        assert!(code.code.chars().all(|c| c.is_ascii_digit()));
        assert_eq!(code.period_seconds, 30);
    }

    #[test]
    fn protected_fields_are_hidden_unless_revealed() {
        let mut db = Database::new();
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, "Example");
            e.set_protected(fields::PASSWORD, "secret-password");
            e.set(
                "otp",
                Value::protected("otpauth://totp/example?secret=SECRET"),
            );
        });

        let hidden = show_entry(&db, "Example", false).unwrap();
        assert_eq!(hidden.password, None);
        assert_eq!(hidden.custom_fields[0].key, "otp");
        assert_eq!(hidden.custom_fields[0].value, "<protected>");

        let revealed = show_entry(&db, "Example", true).unwrap();
        assert_eq!(revealed.password.as_deref(), Some("secret-password"));
        assert_eq!(
            revealed.custom_fields[0].value,
            "otpauth://totp/example?secret=SECRET"
        );
    }

    #[test]
    fn upgrade_rejects_same_input_and_output_before_unlock() {
        let backup = Path::new("../../assets/masked-local-vault.backup.kdbx");
        if backup.exists() {
            let err = upgrade_to_kdbx41(backup, backup, "not-the-real-password", false, false)
                .expect_err("same input/output must be rejected before opening DB");
            assert!(matches!(err, AnahtarError::InputOutputSame(_)));
        }
    }

    #[test]
    fn write_commands_reject_same_input_and_output_before_unlock() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let add_err = add_entry_save_as(
                input,
                "wrong-password-should-not-be-used",
                SaveAsOptions {
                    output_path: input.to_path_buf(),
                    force: false,
                },
                AddEntryRequest {
                    group_path: "General/Web".to_string(),
                    title: "Should Not Add".to_string(),
                    username: None,
                    password: None,
                    url: None,
                    notes: None,
                },
            )
            .expect_err("add should reject same input/output before opening DB");
            assert!(matches!(add_err, AnahtarError::InputOutputSame(_)));

            let edit_err = edit_entry_save_as(
                input,
                "Github Test",
                "wrong-password-should-not-be-used",
                SaveAsOptions {
                    output_path: input.to_path_buf(),
                    force: false,
                },
                EditEntryRequest {
                    username: Some("nope".to_string()),
                    ..Default::default()
                },
            )
            .expect_err("edit should reject same input/output before opening DB");
            assert!(matches!(edit_err, AnahtarError::InputOutputSame(_)));

            let delete_err = delete_entry_save_as(
                input,
                "not-a-uuid",
                "wrong-password-should-not-be-used",
                SaveAsOptions {
                    output_path: input.to_path_buf(),
                    force: false,
                },
            )
            .expect_err("delete should reject same input/output before opening DB");
            assert!(matches!(delete_err, AnahtarError::InputOutputSame(_)));
        }
    }

    #[test]
    fn delete_entry_save_as_removes_entry_and_verifies_reopen() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let before = open_database(input, "testpass").unwrap();
            let target = show_entry(&before, "Github Test", false).unwrap().id;
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("delete.kdbx");
            let report = delete_entry_save_as(
                input,
                &target,
                "testpass",
                SaveAsOptions {
                    output_path: output.clone(),
                    force: false,
                },
            )
            .unwrap();
            assert_eq!(report.operation, WriteOperation::Delete);
            assert_eq!(report.input_entry_count, 4);
            assert_eq!(report.output_entry_count, 3);
            assert_eq!(report.input_group_count, report.output_group_count);

            let reopened = open_database(&output, "testpass").unwrap();
            let err =
                show_entry(&reopened, &target, false).expect_err("deleted entry should be absent");
            assert!(matches!(err, AnahtarError::EntryNotFound(_)));
        }
    }

    #[test]
    fn delete_entry_save_as_rejects_non_uuid_selector() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("delete.kdbx");
            let err = delete_entry_save_as(
                input,
                "Github Test",
                "testpass",
                SaveAsOptions {
                    output_path: output,
                    force: false,
                },
            )
            .expect_err("delete should require UUID selector");
            assert!(matches!(err, AnahtarError::EntryNotFound(_)));
        }
    }

    #[test]
    fn edit_entry_save_as_updates_only_provided_fields() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("edit.kdbx");
            let before = open_database(input, "testpass").unwrap();
            let original = show_entry(&before, "Github Test", true).unwrap();
            let report = edit_entry_save_as(
                input,
                "Github Test",
                "testpass",
                SaveAsOptions {
                    output_path: output.clone(),
                    force: false,
                },
                EditEntryRequest {
                    username: Some("updated-user".to_string()),
                    notes: Some("updated notes".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(report.operation, WriteOperation::Edit);
            assert_eq!(report.input_entry_count, 4);
            assert_eq!(report.output_entry_count, 4);

            let reopened = open_database(&output, "testpass").unwrap();
            let detail =
                show_entry(&reopened, report.changed_entry_id.as_deref().unwrap(), true).unwrap();
            assert_eq!(detail.title, original.title);
            assert_eq!(detail.url, original.url);
            assert_eq!(detail.password, original.password);
            assert_eq!(detail.username.as_deref(), Some("updated-user"));
            assert_eq!(detail.notes.as_deref(), Some("updated notes"));
        }
    }

    #[test]
    fn duplicate_title_error_shows_safe_candidate_summaries() {
        let path = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if path.exists() {
            let db = open_database(path, "testpass").unwrap();
            let err = show_entry_by_selector(
                &db,
                &EntrySelector::Title("Duplicate Title".to_string()),
                false,
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("candidates:"));
            assert!(err.contains("duplicate-web-user"));
            assert!(!err.contains("duplicate-web-pass"));
        }
    }

    #[test]
    fn group_operations_save_as_work_on_synthetic_vault() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let credentials = VaultCredentials::password_only("testpass");

            let group_added = dir.path().join("group-added.kdbx");
            add_group_save_as_with_credentials(
                input,
                &credentials,
                SaveAsOptions {
                    output_path: group_added.clone(),
                    force: false,
                },
                "General/API",
            )
            .unwrap();
            let db = open_database(&group_added, "testpass").unwrap();
            assert!(list_groups(&db)
                .iter()
                .any(|g| g.path == "Root/General/API"));

            let renamed = dir.path().join("group-renamed.kdbx");
            rename_group_save_as_with_credentials(
                &group_added,
                &credentials,
                SaveAsOptions {
                    output_path: renamed.clone(),
                    force: false,
                },
                "General/API",
                "Services",
            )
            .unwrap();
            let db = open_database(&renamed, "testpass").unwrap();
            assert!(list_groups(&db)
                .iter()
                .any(|g| g.path == "Root/General/Services"));

            let moved = dir.path().join("moved.kdbx");
            move_entry_save_as_with_credentials(
                &renamed,
                &credentials,
                SaveAsOptions {
                    output_path: moved.clone(),
                    force: false,
                },
                &EntrySelector::Title("Github Test".to_string()),
                "General/Services",
            )
            .unwrap();
            let db = open_database(&moved, "testpass").unwrap();
            assert_eq!(
                show_entry(&db, "Github Test", false).unwrap().group_path,
                "Root/General/Services"
            );

            let deleted = dir.path().join("deleted-group.kdbx");
            delete_group_save_as_with_credentials(
                &moved,
                &credentials,
                SaveAsOptions {
                    output_path: deleted.clone(),
                    force: false,
                },
                "General/Services",
            )
            .unwrap();
            let db = open_database(&deleted, "testpass").unwrap();
            assert!(!list_groups(&db)
                .iter()
                .any(|g| g.path == "Root/General/Services"));
        }
    }

    #[test]
    fn audit_database_never_reports_secret_values() {
        let mut db = Database::new();
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, "Weak Missing");
            e.set_protected(fields::PASSWORD, "secret");
        });
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, "Reuse 1");
            e.set_protected(fields::PASSWORD, "same-secret");
        });
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, "Reuse 2");
            e.set_protected(fields::PASSWORD, "same-secret");
        });
        let json = serde_json::to_string(&audit_database(&db)).unwrap();
        assert!(json.contains("weak_password"));
        assert!(json.contains("reused_password"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("same-secret"));
    }

    #[test]
    fn edit_entry_save_as_rejects_duplicate_title_selector() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("edit.kdbx");
            let err = edit_entry_save_as(
                input,
                "Duplicate Title",
                "testpass",
                SaveAsOptions {
                    output_path: output,
                    force: false,
                },
                EditEntryRequest {
                    username: Some("updated-user".to_string()),
                    ..Default::default()
                },
            )
            .expect_err("duplicate title selector should fail");
            assert!(matches!(err, AnahtarError::DuplicateEntrySelector(_)));
        }
    }

    #[test]
    fn safe_in_place_write_creates_backup_and_updates_target() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("target.kdbx");
            let backup_dir = dir.path().join("backups");
            std::fs::copy(input, &target).unwrap();
            let credentials = VaultCredentials::password_only("testpass");
            let report = safe_in_place_write_with_credentials(
                &credentials,
                InPlaceOptions {
                    target_path: target.clone(),
                    backup_dir: Some(backup_dir.clone()),
                },
                |source, output| {
                    add_entry_save_as_with_credentials(
                        source,
                        &credentials,
                        SaveAsOptions {
                            output_path: output.to_path_buf(),
                            force: false,
                        },
                        AddEntryRequest {
                            group_path: "General/Web".to_string(),
                            title: "In-place Test".to_string(),
                            username: None,
                            password: None,
                            url: None,
                            notes: None,
                        },
                    )
                },
            )
            .unwrap();

            assert_eq!(report.final_target_path.as_ref(), Some(&target));
            assert!(report.backup_path.as_ref().unwrap().exists());
            let updated = open_database(&target, "testpass").unwrap();
            assert_eq!(updated.iter_all_entries().count(), 5);
            let backup = open_database(report.backup_path.unwrap(), "testpass").unwrap();
            assert_eq!(backup.iter_all_entries().count(), 4);
        }
    }

    #[test]
    fn safe_in_place_write_preserves_original_on_preflight_failure() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("target.kdbx");
            std::fs::copy(input, &target).unwrap();
            let temp = dir.path().join(".target.kdbx.anahtar.tmp");
            std::fs::write(&temp, b"collision").unwrap();
            let credentials = VaultCredentials::password_only("testpass");
            let result = safe_in_place_write_with_credentials(
                &credentials,
                InPlaceOptions {
                    target_path: target.clone(),
                    backup_dir: Some(dir.path().join("backups")),
                },
                |_source, _output| unreachable!(),
            );
            assert!(result.is_err());
            let original = open_database(&target, "testpass").unwrap();
            assert_eq!(original.iter_all_entries().count(), 4);
        }
    }

    #[test]
    fn add_entry_save_as_adds_one_entry_and_verifies_reopen() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("add.kdbx");
            let report = add_entry_save_as(
                input,
                "testpass",
                SaveAsOptions {
                    output_path: output.clone(),
                    force: false,
                },
                AddEntryRequest {
                    group_path: "General/Web".to_string(),
                    title: "Added By Test".to_string(),
                    username: Some("added-user".to_string()),
                    password: Some("added-pass".to_string()),
                    url: Some("https://example.com/added".to_string()),
                    notes: Some("added notes".to_string()),
                },
            )
            .unwrap();
            assert_eq!(report.operation, WriteOperation::Add);
            assert_eq!(report.input_entry_count, 4);
            assert_eq!(report.output_entry_count, 5);
            assert_eq!(report.input_group_count, report.output_group_count);

            let reopened = open_database(&output, "testpass").unwrap();
            let detail =
                show_entry(&reopened, report.changed_entry_id.as_deref().unwrap(), true).unwrap();
            assert_eq!(detail.title.as_deref(), Some("Added By Test"));
            assert_eq!(detail.username.as_deref(), Some("added-user"));
            assert_eq!(detail.password.as_deref(), Some("added-pass"));
            assert_eq!(detail.group_path, "Root/General/Web");
        }
    }

    #[test]
    fn add_entry_save_as_rejects_missing_group() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("add.kdbx");
            let err = add_entry_save_as(
                input,
                "testpass",
                SaveAsOptions {
                    output_path: output,
                    force: false,
                },
                AddEntryRequest {
                    group_path: "General/Missing".to_string(),
                    title: "Added By Test".to_string(),
                    username: None,
                    password: None,
                    url: None,
                    notes: None,
                },
            )
            .expect_err("missing group should fail");
            assert!(matches!(err, AnahtarError::GroupNotFound(_)));
        }
    }

    #[test]
    fn upgrade_still_works_after_save_helper_refactor() {
        let input = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if input.exists() {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("upgraded.kdbx");
            let report = upgrade_to_kdbx41(input, &output, "testpass", false, false).unwrap();
            assert_eq!(report.input_entry_count, 4);
            assert_eq!(report.output_entry_count, Some(4));
            assert_eq!(
                inspect_header(&output).unwrap().version,
                KdbxVersion::Kdbx { major: 4, minor: 1 }
            );
            let reopened = open_database(&output, "testpass").unwrap();
            assert_eq!(count_groups(&reopened), 4);
            assert_eq!(reopened.iter_all_entries().count(), 4);
        }
    }

    #[test]
    fn generated_phase3_vault_opens() {
        let path = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if path.exists() {
            let db = open_database(path, "testpass").unwrap();
            assert_eq!(
                database_version(&db),
                KdbxVersion::Kdbx { major: 4, minor: 1 }
            );
            assert_eq!(count_groups(&db), 4);
            assert_eq!(db.iter_all_entries().count(), 4);
            assert_eq!(search_entries(&db, "Github Test").len(), 1);
            assert_eq!(search_entries(&db, "Duplicate Title").len(), 2);
        }
    }

    #[test]
    fn vault_credentials_password_only_opens_generated_vault() {
        let path = Path::new("../../test-vaults/generated/phase3-base.kdbx");
        if path.exists() {
            let credentials = VaultCredentials::password_only("testpass");
            let db = open_database_with_credentials(path, &credentials).unwrap();
            assert_eq!(db.iter_all_entries().count(), 4);
        }
    }

    #[test]
    fn vault_credentials_password_plus_key_file_opens_synthetic_vault() {
        let dir = tempfile::tempdir().unwrap();
        let key_file = dir.path().join("test.keyx");
        std::fs::write(
            &key_file,
            b"<KeyFile><Key><Data>NXyYiJMHg3ls+eBmjbAjWec9lcOToJiofbhNiFMTJMw=</Data></Key></KeyFile>",
        )
        .unwrap();

        let vault = dir.path().join("key-file-vault.kdbx");
        let mut db = Database::new();
        db.root_mut().edit(|root| root.name = "Root".to_string());
        db.root_mut().add_entry().edit(|entry| {
            entry.set_unprotected(fields::TITLE, "Key File Test");
            entry.set_protected(fields::PASSWORD, "secret");
        });

        let mut key_reader = File::open(&key_file).unwrap();
        let key = DatabaseKey::new()
            .with_password("testpass")
            .with_keyfile(&mut key_reader)
            .unwrap();
        let mut out = File::create(&vault).unwrap();
        db.save(&mut out, key).unwrap();

        let credentials = VaultCredentials::with_key_file("testpass", &key_file);
        let reopened = open_database_with_credentials(&vault, &credentials).unwrap();
        assert_eq!(reopened.iter_all_entries().count(), 1);
    }

    #[test]
    fn inspect_known_assets() {
        let backup = Path::new("../../assets/masked-local-vault.backup.kdbx");
        if backup.exists() {
            let info = inspect_header(backup).unwrap();
            assert_eq!(info.version, KdbxVersion::Kdbx { major: 4, minor: 0 });
        }
        let upgraded = Path::new("../../assets/masked-local-vault.kdbx41.test.kdbx");
        if upgraded.exists() {
            let info = inspect_header(upgraded).unwrap();
            assert_eq!(info.version, KdbxVersion::Kdbx { major: 4, minor: 1 });
        }
    }
}
