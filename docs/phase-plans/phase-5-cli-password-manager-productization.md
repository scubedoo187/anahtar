# Phase 5 — CLI Password Manager Productization

Status: complete

## Goal

Turn Anahtar from a daily-use KDBX CLI MVP into a credible **password manager CLI product** before starting GUI work.

The reason for this phase is architectural: GUI work should mostly assemble already-stable core capabilities instead of discovering missing password-manager functionality one screen at a time.

## Product definition for this phase

By the end of Phase 5, Anahtar should be usable as a primary CLI password manager for a local/synced KeePass-compatible `.kdbx` vault, while still preserving Anahtar's safety-first write model.

## Scope

### In scope

- Credential material beyond password-only unlock.
- Safe in-place update workflow with backup and verification.
- Stable selector semantics.
- Basic group/move management.
- Audit/check commands.
- CI/install/release polish.
- Documentation of CLI product behavior and threat model.

### Out of scope

- GUI implementation.
- Browser extension.
- Mobile app.
- Cloud sync engine.
- Shared vaults.
- Passkeys.
- Background unlock daemon.
- Biometric unlock.
- Full import/export migration system.
- Attachment management.

## Phase 5 design principles

- Keep `anahtar-core` as the reusable business logic layer.
- GUI-facing functionality should be available from core first.
- Preserve save-as flows even after in-place writes are added.
- Safe in-place writes are the default product workflow once implemented; save-as remains available for cautious/manual workflows.
- No secrets in CLI args, logs, or normal output.
- Prefer explicit selectors over ambiguous title matching.
- Keep KDBX write output standardized on KDBX 4.1.

## Workstream 1 — Credential material / key-file support

Goal: support common KeePass unlock workflows, especially password + key file.

Tasks:

- [x] Inspect `keepass = 0.13.8` credential API for password + key-file support.
- [x] Add core credential abstraction, e.g. `VaultCredentials`.
- [x] Keep master password prompted through TTY.
- [x] Add optional CLI `--key-file <path>` to vault-unlocking commands.
- [x] Add optional config field for default key file.
- [x] Make `config set key-file` canonicalize and require an existing file, matching vault path behavior.
- [x] Ensure key-file path is never treated as secret but is still user-local config.
- [x] Add tests/fixtures where possible.
- [x] Update README with key-file usage.

Acceptance criteria:

- [x] Password-only vaults still work.
- [x] Password + key-file vaults can be opened if supported by backend.
- [x] CLI and config fallback behavior are consistent with `--vault`.
- [x] Invalid/missing key-file paths fail clearly before unlock.

## Workstream 2 — Safe in-place write workflow

Goal: make CLI write operations practical for daily use without abandoning safety.

Current state: write commands require `--output` save-as and never modify input.

Target behavior:

- Save-as remains supported:

```bash
anahtar add --output out.kdbx ...
```

- Configured/default vault can be updated safely:

```bash
anahtar add --group General/Web --title Example --generate-password
anahtar edit --id <uuid> --set-username new@example.com
anahtar delete --id <uuid>
```

Required write algorithm:

1. Resolve target vault.
2. Create timestamped backup in a configured/default backup location.
3. Save modified database to temp file in same filesystem/directory when possible.
4. Reopen temp file and verify counts/target state.
5. Flush/sync where possible.
6. Atomically replace target vault.
7. Reopen final target and verify.
8. Leave backup intact.
9. On failure, preserve original and report cleanup state.

Tasks:

- [x] Implement default backup path policy: create timestamped backups next to the vault under a sibling `anahtar-backups/` directory.
- [x] Add optional config override for backup directory.
- [x] Add core safe in-place save helper.
- [x] Add write report fields for backup path and final target path.
- [x] Refactor `add/edit/delete` so safe in-place update is the default when `--output` is omitted.
- [x] Preserve `--output` save-as behavior for explicit non-mutating writes.
- [x] Add `--dry-run` where useful.
- [x] Add `--yes`/confirmation policy for destructive in-place operations.
- [x] Add tests for backup creation and original preservation on preflight failures.
- [x] Add tests for final reopen verification.
- [x] Update README.
- [x] Update threat model.

Acceptance criteria:

- [x] Existing save-as write commands still work.
- [x] In-place update creates a backup before replacing the vault.
- [x] In-place update uses temp save + reopen verification + final reopen verification.
- [x] Delete remains confirmation-protected unless explicitly bypassed.
- [x] Failure cases do not destroy the original vault.

## Workstream 3 — Stable selectors and duplicate handling

Goal: make entry targeting predictable for CLI and future GUI calls.

Current state: selectors accept UUID or exact title in several flows, but title duplicates can be ambiguous.

Target selector model:

```bash
anahtar show --id <uuid>
anahtar show --title "Github Test"
anahtar show --url github.com
anahtar show --username user@example.com
```

Potential shorthand can remain:

```bash
anahtar show github
```

…but explicit selectors should be preferred and documented.

Tasks:

- [x] Define `EntrySelector` in core.
- [x] Support explicit selector variants: id, title, url, username.
- [x] Normalize command syntax around explicit selectors.
- [x] Keep backward-compatible positional selector initially if practical.
- [x] Improve duplicate-match errors to show safe candidate summaries.
- [x] Ensure copy/totp/edit/delete all use the same selector logic.
- [x] Add tests for duplicate title behavior.
- [x] Update README examples to prefer explicit selectors.

Acceptance criteria:

- [x] UUID targeting is unambiguous.
- [x] Duplicate title errors are actionable and do not reveal secrets.
- [x] All commands share consistent selector behavior.
- [x] Future GUI can call the same core selector resolver.

## Workstream 4 — Group and entry organization

Goal: support basic vault organization, not just entry CRUD.

Commands to consider:

```bash
anahtar group list
anahtar group add "General/API"
anahtar group rename "General/API" "General/Services"
anahtar group delete "General/Old" --yes
anahtar move --id <uuid> --group "General/API"
```

Tasks:

- [x] Inspect keepass crate group mutation support.
- [x] Add core group summary type.
- [x] Implement group list.
- [x] Implement group add with save-as/in-place support.
- [x] Implement entry move.
- [x] Implement group rename.
- [x] Implement group delete with confirmation unless `--yes`.
- [x] Add duplicate/missing group error handling.
- [x] Add tests with generated synthetic vault.
- [x] Update README.

Acceptance criteria:

- [x] User can list groups.
- [x] User can create a group.
- [x] User can rename a group.
- [x] User can delete a group with confirmation.
- [x] User can move an entry to a group.
- [x] Group operations use the same write safety model as entry writes.

## Workstream 5 — Audit/check commands

Goal: add product-grade password-manager utility beyond CRUD.

Commands to consider:

```bash
anahtar audit
anahtar audit weak
anahtar audit reused
anahtar audit missing-url
anahtar audit missing-username
anahtar audit totp
```

Tasks:

- [x] Add safe audit result types to core.
- [x] Implement weak-password check without printing passwords.
- [x] Implement reused-password grouping without printing passwords.
- [x] Implement missing username/url checks.
- [x] Implement entries-with-totp / entries-missing-totp checks if practical.
- [x] Add JSON output.
- [x] Add tests using synthetic vault.
- [x] Update README.

Acceptance criteria:

- [x] Audit commands never print secret values.
- [x] JSON output is suitable for GUI or scripts.
- [x] Audit output is actionable from CLI.

## Workstream 6 — Install, CI, and release polish

Goal: make Anahtar easier to install and safer to evolve.

Tasks:

- [x] Add GitHub Actions workflow for fmt, clippy, tests.
- [x] Document `cargo install --path crates/anahtar-cli` or workspace install command.
- [x] Add release profile notes.
- [x] Add shell completion generation for common shells.
- [x] Add `anahtar --version` verification to docs.
- [x] Decide whether to add `cargo audit` / `cargo deny` now or in a security hardening phase.

Acceptance criteria:

- [x] CI runs fmt/clippy/test on push/PR.
- [x] Local install instructions work.
- [x] User can run installed `anahtar` without Cargo.

## Workstream 7 — Threat model and product docs

Goal: document what Anahtar protects and what it does not protect.

Tasks:

- [x] Add `docs/threat-model.md`.
- [x] Document master password handling.
- [x] Document key-file handling.
- [x] Document clipboard risks and blocking clear behavior.
- [x] Document terminal scrollback risks for `--reveal-password` and JSON output.
- [x] Document backup/in-place write safety model.
- [x] Document public repo hygiene rules.

Acceptance criteria:

- [x] README links to threat model.
- [x] In-place write docs clearly explain backups and failure behavior.
- [x] Clipboard limitations are clear.

## Suggested implementation order

1. Stable selectors.
2. Key-file credential abstraction.
3. Group list/add and entry move.
4. Safe in-place write workflow.
5. Audit/check commands.
6. CI/install/release polish.
7. Threat model/product docs pass.

Rationale:

- Selectors affect many commands, so settle them first.
- Credential abstraction should be in place before adding many new vault commands.
- Group/move operations expose write model needs before in-place update is finalized.
- Safe in-place write should land once save-as variants and selector semantics are stable.
- Audit commands are low-risk once read model is stable.

## Exit criteria

- [x] Anahtar can unlock password-only and, if backend supports it, password + key-file vaults.
- [x] Anahtar can safely update the configured vault in place with backup and verification.
- [x] Save-as write flows remain available and tested.
- [x] Selectors are explicit, consistent, and duplicate-safe.
- [x] Basic group organization is supported.
- [x] Audit commands provide useful non-secret findings.
- [x] CI verifies fmt, clippy, and tests.
- [x] README describes installed CLI usage, safety model, and product workflows.
- [x] Threat model is documented.

## Open questions before implementation

Resolved decisions:

1. Safe in-place writes should be the default once implemented. This is core to productization. Save-as remains available through explicit `--output`.
2. Backups should default to a timestamped file under a sibling `anahtar-backups/` directory next to the vault, with optional config override.
3. `config set key-file` should store a canonical absolute path and require an existing file, matching vault behavior.
4. Group rename/delete are required in Phase 5.
5. Shell completion is included in Phase 5.
