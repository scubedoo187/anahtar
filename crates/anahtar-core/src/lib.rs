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
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
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

#[derive(Debug, Clone, Serialize)]
pub struct VaultInfo {
    pub path: PathBuf,
    pub file_size_bytes: u64,
    pub version: KdbxVersion,
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
pub struct EntryDetail {
    pub id: String,
    pub group_path: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
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
    let mut f = File::open(path)?;
    Database::open(&mut f, DatabaseKey::new().with_password(password))
        .map_err(|_| AnahtarError::OpenDatabase)
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
    let summary = resolve_entry_summary(db, selector)?;
    let entry = find_entry(db, &summary.id)
        .ok_or_else(|| AnahtarError::EntryNotFound(selector.to_string()))?;
    Ok(detail_from_entry(
        entry,
        summary.group_path,
        reveal_password,
    ))
}

pub fn totp_code(db: &Database, selector: &str) -> Result<TotpCode> {
    let summary = resolve_entry_summary(db, selector)?;
    let entry = find_entry(db, &summary.id)
        .ok_or_else(|| AnahtarError::EntryNotFound(selector.to_string()))?;
    let otp = entry
        .get_otp()
        .map_err(|_| AnahtarError::TotpUnavailable(selector.to_string()))?;
    let code = otp
        .value_now()
        .map_err(|_| AnahtarError::TotpUnavailable(selector.to_string()))?;
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
    let input = input.as_ref();
    let output = output.as_ref();

    if canonical_for_compare(input)? == canonical_for_compare(output)? {
        return Err(AnahtarError::InputOutputSame(output.to_path_buf()));
    }

    let mut db = open_database(input, password)?;
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
        password,
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
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database(input, password)?;
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
        password,
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
    })
}

pub fn edit_entry_save_as(
    input: impl AsRef<Path>,
    selector: &str,
    password: &str,
    options: SaveAsOptions,
    request: EditEntryRequest,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let mut db = open_database(input, password)?;
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
        password,
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
    })
}

pub fn delete_entry_save_as(
    input: impl AsRef<Path>,
    entry_id: &str,
    password: &str,
    options: SaveAsOptions,
) -> Result<WriteReport> {
    let input = input.as_ref();
    ensure_input_output_distinct(input, &options.output_path)?;
    let uuid =
        Uuid::parse_str(entry_id).map_err(|_| AnahtarError::EntryNotFound(entry_id.to_string()))?;
    let entry_id = keepass::db::EntryId::from_uuid(uuid);
    let mut db = open_database(input, password)?;
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
        password,
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
    })
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
    password: &str,
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
            db.save(&mut out, DatabaseKey::new().with_password(password))
                .map_err(|_| AnahtarError::SaveDatabase)?;
            out.flush()?;
            out.sync_all()?;
        }

        let verified = open_database(&tmp, password)?;
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

fn resolve_entry_summary(db: &Database, selector: &str) -> Result<EntrySummary> {
    list_entries(db)
        .into_iter()
        .find(|e| {
            e.id == selector
                || e.title
                    .as_deref()
                    .is_some_and(|t| t.eq_ignore_ascii_case(selector))
        })
        .ok_or_else(|| AnahtarError::EntryNotFound(selector.to_string()))
}

fn resolve_unique_entry_id(db: &Database, selector: &str) -> Result<keepass::db::EntryId> {
    if let Ok(uuid) = Uuid::parse_str(selector) {
        let entry_id = keepass::db::EntryId::from_uuid(uuid);
        if db.entry(entry_id).is_some() {
            return Ok(entry_id);
        }
        return Err(AnahtarError::EntryNotFound(selector.to_string()));
    }

    let matches = db
        .iter_all_entries()
        .filter(|entry| {
            entry
                .get_title()
                .is_some_and(|title| title.eq_ignore_ascii_case(selector))
        })
        .map(|entry| entry.id())
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [entry_id] => Ok(*entry_id),
        [] => Err(AnahtarError::EntryNotFound(selector.to_string())),
        _ => Err(AnahtarError::DuplicateEntrySelector(selector.to_string())),
    }
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
        let backup = Path::new("../../assets/private-vault.backup.kdbx");
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
    fn inspect_known_assets() {
        let backup = Path::new("../../assets/private-vault.backup.kdbx");
        if backup.exists() {
            let info = inspect_header(backup).unwrap();
            assert_eq!(info.version, KdbxVersion::Kdbx { major: 4, minor: 0 });
        }
        let upgraded = Path::new("../../assets/private-vault.kdbx41.test.kdbx");
        if upgraded.exists() {
            let info = inspect_header(upgraded).unwrap();
            assert_eq!(info.version, KdbxVersion::Kdbx { major: 4, minor: 1 });
        }
    }
}
