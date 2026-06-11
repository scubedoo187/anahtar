# Phase 3 plan — Minimal write commands with save-as

Phase 3 introduces content-changing commands for the first time. The priority is safety over convenience: every command writes to a new output file and never modifies the input vault in place.

## Goal

Implement minimal `add`, `edit`, and `delete` commands for KDBX vaults using the same KDBX4.1 save-as safety model established in Phase 2.

All Phase 3 write commands must:

- require an explicit output path,
- reject `input == output`,
- refuse to overwrite existing output unless `--force` is passed,
- write KDBX4.1 output,
- save to a temp file first,
- reopen and verify the temp file,
- then rename to the final output path.

## Scope

Commands:

- `anahtar add <input> --output <output> ...`
- `anahtar edit <input> <entry-id-or-title> --output <output> ...`
- `anahtar delete <input> <entry-id-or-title> --output <output> ...`

Non-goals for this phase:

- in-place save,
- recycle-bin delete,
- browser extension,
- GUI write support,
- automatic group creation,
- password values passed directly as CLI arguments.

## Command design

### `add`

Proposed CLI:

```bash
anahtar add <input> \
  --output <output> \
  --group "General/Web" \
  --title "Github" \
  --username "user@example.com" \
  --password-prompt \
  --url "https://github.com" \
  --notes "optional notes"
```

MVP behavior:

- `--output` is required.
- `--group` must refer to an existing group path.
- Missing group is an error.
- `--title` is required.
- Password mode must be explicit:
  - `--password-prompt` prompts for entry password and confirmation.
  - `--no-password` creates an entry without a password.
  - Passing both is an error.
  - Passing neither is an error.
- Password is not accepted as a plain CLI argument.
- Output is KDBX4.1.

Group path policy:

- Root group name is omitted.
- `/` is the delimiter.
- Example: `--group "General/Web"`.
- `--group "Root/General/Web"` is not supported in Phase 3.
- Group names containing `/` are not supported in Phase 3.

Future options, not Phase 3:

- `--create-group`
- `--password-stdin`
- custom fields
- attachments

### `edit`

Proposed CLI:

```bash
anahtar edit <input> <entry-id-or-title> \
  --output <output> \
  --title "New Title" \
  --username "new-user" \
  --url "https://new.example.com" \
  --notes "new notes"
```

Password edit:

```bash
anahtar edit <input> <entry-id-or-title> \
  --output <output> \
  --password-prompt
```

MVP behavior:

- UUID selector is recommended.
- Title selector may be supported for convenience but must be handled carefully if duplicates exist.
- Only fields explicitly provided are modified.
- `--notes ""` means set notes to an empty string.
- Password behavior:
  - no password option means keep the existing password unchanged.
  - `--password-prompt` means replace password after prompt + confirmation.
  - password deletion/clearing is not supported in Phase 3.
- Output is KDBX4.1.

Duplicate selector policy:

- If a UUID matches, use that entry.
- If a title matches exactly one entry, use it.
- If a title matches multiple entries, fail and ask the user to use UUID.

### `delete`

Proposed CLI:

```bash
anahtar delete <input> <entry-id> \
  --output <output>
```

Confirmation:

```text
Delete entry?
Title: Github
Username: user@example.com
URL: https://github.com

Type DELETE to confirm:
```

Automation option:

```bash
--yes
```

MVP behavior:

- Phase 3 delete accepts UUID only; title-based delete is not supported.
- Hard delete from the output file.
- Input file remains untouched, so recovery is possible from input.
- Recycle-bin behavior is explicitly deferred to a later phase.
- Output is KDBX4.1.

## Core API plan

Add write-oriented structures in `anahtar-core`:

```rust
pub struct SaveAsOptions {
    pub output_path: PathBuf,
    pub force: bool,
}

pub struct WriteReport {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_version: KdbxVersion,
    pub output_version: KdbxVersion,
    pub input_group_count: usize,
    pub input_entry_count: usize,
    pub output_group_count: usize,
    pub output_entry_count: usize,
    pub changed_entry_id: Option<String>,
    pub operation: WriteOperation,
}

pub enum WriteOperation {
    Add,
    Edit,
    Delete,
}
```

Add core functions:

```rust
add_entry_save_as(...)
edit_entry_save_as(...)
delete_entry_save_as(...)
```

Refactor Phase 2 save-as code into a shared helper:

```rust
save_as_kdbx41_verified(...)
```

The shared helper should handle:

- input/output same-file rejection,
- output exists protection,
- temp path collision protection,
- temp save,
- flush/sync,
- reopen verification,
- count verification,
- operation-specific verification,
- final rename,
- temp cleanup on failure.

## Test vault strategy

Phase 3 should use dedicated test vaults for write-command development. The personal vault should be reserved for final smoke testing only.

Recommended local structure:

```text
test-vaults/
  README.md
  generated/
    phase3-base.kdbx
    outputs/
      phase3-add.kdbx
      phase3-edit.kdbx
      phase3-delete.kdbx
  strongbox/
    phase3-strongbox-base.kdbx
```

Recommended generated test vault password:

```text
testpass
```

Recommended generated test vault contents:

```text
Root
  General
    Web
      Github Test
      Duplicate Title
    Email
      Email Test
      Duplicate Title
```

Entry examples:

- `Github Test`
  - username: `github-user`
  - password: `github-pass`
  - url: `https://github.com`
- `Email Test`
  - username: `email-user`
  - password: `email-pass`
  - url: `https://mail.example.com`
- two `Duplicate Title` entries for duplicate-selector behavior tests

Use two fixture types:

1. **Anahtar-generated test vault**
   - Used for automated and repeated add/edit/delete development.
   - Can be regenerated deterministically enough for local testing.
2. **Strongbox-generated test vault**
   - Used for compatibility smoke testing.
   - Confirms Anahtar can modify files created by the actual target app.

Personal vault policy:

- Do not use the personal vault for iterative write-command development.
- Use `assets/masked-local-vault.kdbx41.test.kdbx` only for final smoke tests after test-vault flows pass.

## Implementation checklist

### Test vault setup

- [x] Create `test-vaults/README.md` documenting password, purpose, and safety rules.
- [x] Create generated Phase 3 base vault with password `testpass`.
- [x] Add groups `General/Web` and `General/Email`.
- [x] Add baseline test entries.
- [x] Add duplicate-title test entries.
- [x] Confirm generated base vault opens in Anahtar.
- [x] Confirm generated base vault opens in Strongbox.
- [x] Optionally create a Strongbox-generated base vault for compatibility smoke tests. Deferred; generated base vault already opens in Strongbox.

### Shared write foundation

- [x] Extract common KDBX4.1 save-as verification helper from `upgrade_to_kdbx41`.
- [x] Keep Phase 2 `upgrade` behavior unchanged after refactor.
- [x] Add `WriteReport` / `WriteOperation` / `SaveAsOptions`.
- [x] Add tests proving `upgrade` still works after refactor.

### Add command

- [x] Implement existing group path lookup.
- [x] Implement `add_entry_save_as` in core.
- [x] Add CLI `add` command.
- [x] Prompt entry password with confirmation when `--password-prompt` is set.
- [x] Reject missing required fields.
- [x] Verify output entry count is input count + 1.
- [x] Verify added entry exists after reopen.
- [x] Add unit/integration tests.

### Edit command

- [x] Implement unique selector resolution:
  - [x] UUID exact match,
  - [x] title match only if unique,
  - [x] duplicate title error.
- [x] Implement `edit_entry_save_as` in core.
- [x] Add CLI `edit` command.
- [x] Modify only explicitly provided fields.
- [x] Support `--password-prompt` for password update.
- [x] Verify output entry count is unchanged.
- [x] Verify edited fields persist after reopen.
- [x] Add unit/integration tests.

### Delete command

- [x] Implement `delete_entry_save_as` in core.
- [x] Add CLI `delete` command.
- [x] Show entry summary before confirmation.
- [x] Require typing `DELETE` unless `--yes` is passed.
- [x] Verify output entry count is input count - 1.
- [x] Verify deleted entry is absent after reopen.
- [x] Add unit/integration tests.

### CLI UX and documentation

- [x] Ensure write commands never print passwords.
- [x] Ensure write command JSON reports contain no secrets, notes content, custom field values, or entry password values.
- [x] JSON reports may include operation, ids, paths, versions, and counts only.
- [x] Add `--json` report output for write commands.
- [x] Document examples for add/edit/delete.
- [x] Update journal when Phase 3 starts and completes.

## Exit criteria

- [x] `add` creates a KDBX4.1 output file without modifying input.
- [x] Added entry appears after reopen.
- [x] Added output opens in Strongbox.
- [x] `edit` creates a KDBX4.1 output file without modifying input.
- [x] Edited fields persist after reopen.
- [x] Edited output opens in Strongbox.
- [x] `delete` creates a KDBX4.1 output file without modifying input.
- [x] Deleted entry is absent after reopen.
- [x] Deleted output opens in Strongbox.
- [x] Existing output protection works for all write commands.
- [x] `input == output` is rejected for all write commands.
- [x] All write commands use temp save + reopen verification.
- [x] `cargo fmt --all` passes.
- [x] `cargo test --workspace` passes.

## Manual verification flow

Use the dedicated generated test vault first:

```bash
test-vaults/generated/phase3-base.kdbx
```

Use the personal KDBX4.1 test vault only after the generated test-vault flow passes:

```bash
assets/masked-local-vault.kdbx41.test.kdbx
```

### Add verification

```bash
cargo run -q -p anahtar-cli -- add \
  'test-vaults/generated/phase3-base.kdbx' \
  --output 'assets/masked-local-vault.phase3.add.kdbx' \
  --group 'General/Web' \
  --title 'Anahtar Test Entry' \
  --username 'anahtar@example.com' \
  --password-prompt \
  --url 'https://example.com' \
  --notes 'Created during Anahtar Phase 3 verification'
```

Then:

```bash
cargo run -q -p anahtar-cli -- search \
  'assets/masked-local-vault.phase3.add.kdbx' \
  'Anahtar Test Entry'
```

Open `assets/masked-local-vault.phase3.add.kdbx` in Strongbox.

### Edit verification

```bash
cargo run -q -p anahtar-cli -- edit \
  'assets/masked-local-vault.phase3.add.kdbx' \
  '<new-entry-id>' \
  --output 'assets/masked-local-vault.phase3.edit.kdbx' \
  --username 'updated-anahtar@example.com' \
  --notes 'Updated during Anahtar Phase 3 verification'
```

Open `assets/masked-local-vault.phase3.edit.kdbx` in Strongbox.

### Delete verification

```bash
cargo run -q -p anahtar-cli -- delete \
  'assets/masked-local-vault.phase3.edit.kdbx' \
  '<new-entry-id>' \
  --output 'assets/masked-local-vault.phase3.delete.kdbx'
```

Open `assets/masked-local-vault.phase3.delete.kdbx` in Strongbox and confirm the test entry is gone.

## Decisions accepted for Phase 3

| Topic | Phase 3 decision |
|---|---|
| Output mode | `--output` required |
| Overwrite | default deny, `--force` required |
| Add password mode | exactly one of `--password-prompt` or `--no-password` |
| Edit password mode | no option keeps existing password; `--password-prompt` replaces; clearing unsupported |
| Password CLI arg | plain password CLI argument is not allowed |
| Group handling for add | existing group only |
| Group path format | root omitted, `/` delimiter, slash in group name unsupported |
| Edit selector | UUID preferred; title only if unique |
| Delete selector | UUID only |
| Delete behavior | hard delete + confirmation; recycle bin deferred |
| Count verification | add +1 entry, edit same count, delete -1 entry; group count same |
| Write JSON report | no secrets, no notes content, no custom field values |
| Output format | KDBX4.1 |
| In-place save | not allowed |
