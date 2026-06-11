# Anahtar GUI API Contract

Status: Phase 5.5 baseline for Phase 6 GUI alpha.

## Dependency boundary

The GUI should depend on `anahtar-app` for high-level workflows and on `anahtar-core` only for shared DTOs/types that are re-exported or explicitly needed.

Preferred stack:

```text
GUI/Tauri command layer
  -> anahtar-app::AnahtarService
    -> anahtar-core KDBX operations
```

The GUI should not reimplement KDBX traversal, selector matching, backup policy, or write verification.

## Credential handling

- The GUI collects the master password at unlock time.
- The GUI may collect or remember a key-file path, but not key-file contents.
- The master password must not be written to config, logs, crash reports, or persistent app state.
- `VaultCredentials` is passed per operation.
- `AnahtarService` is stateless and must not hold long-lived secrets.

## Read workflows

Use `AnahtarService`:

- `inspect(path)` -> `VaultInfo`
- `list(path, credentials)` -> `Vec<EntrySummary>`
- `search(path, credentials, query)` -> `Vec<EntrySummary>`
- `show(path, credentials, selector, reveal_password)` -> `EntryDetail`
- `groups(path, credentials)` -> `Vec<GroupSummary>`
- `audit(path, credentials)` -> `AuditReport`
- `totp(path, credentials, selector)` -> `TotpCode`

Secret display rules:

- Entry summaries never include passwords.
- `EntryDetail.password` is populated only when `reveal_password = true`.
- Protected custom fields stay masked unless `reveal_password = true`.
- Audit findings must never include secret values.

## Selector contract

GUI should prefer explicit selector variants:

- `EntrySelector::Id`
- `EntrySelector::Title`
- `EntrySelector::Url`
- `EntrySelector::Username`

`EntrySelector::Auto` exists for CLI/backward-compatible shorthand. GUI should use UUIDs once an entry has been selected from a list.

Duplicate selector errors are expected and should be shown as actionable validation messages rather than fatal crashes.

## Write workflows

Use `AnahtarService` write methods with `WriteMode`:

- `WriteMode::SaveAs { output_path, force }`
- `WriteMode::InPlace { target_path, backup_dir }`
- `WriteMode::DryRun`

Supported methods:

- `add_entry(...)`
- `edit_entry(...)`
- `delete_entry(...)`
- `add_group(...)`
- `rename_group(...)`
- `delete_group(...)`
- `move_entry(...)`

Write reports return:

- operation type,
- input/output paths,
- input/output version,
- input/output group and entry counts,
- changed entry/group id where available,
- backup path for in-place writes,
- final target path for in-place writes.

## In-place safety contract

In-place writes must use the core/app write helpers. They provide:

1. backup creation,
2. temp KDBX4.1 save,
3. temp reopen verification,
4. target replacement,
5. final target reopen verification,
6. backup restore attempt if final verification fails.

The GUI should display the backup path after successful in-place writes.

## Clipboard boundary

Clipboard behavior is platform/UI-specific.

- Core does not own clipboard operations.
- `anahtar-app` does not store clipboard state.
- GUI may implement its own copy/clear behavior using platform APIs.
- Clipboard clear should only clear if clipboard still contains Anahtar's copied value.

## Error display rules

Errors should be shown as user-actionable messages:

- unlock failure: generic password/key material or file-integrity message,
- missing key file: show path and correction guidance,
- duplicate selector: prompt user to select by UUID,
- output exists: offer overwrite/save-as decision,
- verification failure: show that backup was preserved or restore was attempted.

Do not display master passwords, entry passwords, or raw protected field values in error dialogs.

## Long-running operations

Opening and writing KDBX files can be slow depending on KDF settings and vault size. GUI commands should run off the UI thread and show progress/disabled controls where appropriate.

## What GUI must not do

- Must not store the master password in config.
- Must not bypass `anahtar-app`/`anahtar-core` write helpers for vault mutation.
- Must not print or log secret values.
- Must not treat key-file paths as secret, but should treat them as user-local/private config.
- Must not assume duplicate titles are unique selectors.
