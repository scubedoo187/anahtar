# Phase 1 plan — Read-only CLI MVP

This is the active phase plan. Future phase plans should be created or updated only when that phase is about to start.

## Goal

Build the first real Anahtar implementation as a read-only CLI over `anahtar-core`.

This phase must not modify any KDBX vault.

## Scope

Commands:

- `anahtar inspect <vault>`
- `anahtar list <vault>`
- `anahtar search <vault> <query>`
- `anahtar show <vault> <entry-id-or-title>`

## Implementation checklist

- [x] Create root Rust workspace.
- [x] Create `crates/anahtar-core`.
- [x] Create `crates/anahtar-cli`.
- [x] Implement KDBX header inspection without password.
- [x] Implement password prompt using hidden TTY input.
- [x] Implement database open using `keepass::Database::open`.
- [x] Define safe output structs:
  - [x] `VaultInfo`
  - [x] `EntrySummary`
  - [x] `EntryDetail`
  - [x] `KdbxVersion`
- [x] Implement group/entry traversal.
- [x] Implement safe list output without passwords.
- [x] Implement case-insensitive search.
- [x] Implement show command with password hidden by default.
- [x] Add explicit `--reveal-password` only for intentional password display.
- [x] Add `--json` output for GUI reuse.

## Exit criteria

- [x] `inspect` reports KDBX4.0 for `assets/private-vault.backup.kdbx`.
- [x] `inspect` reports KDBX4.1 for `assets/private-vault.kdbx41.test.kdbx`.
- [x] `list` works on `assets/private-vault.kdbx41.test.kdbx` after password prompt.
- [x] `search` finds expected entries.
- [x] `show` displays non-secret fields by default.
- [x] No command prints passwords unless `--reveal-password` is explicitly passed.
- [x] `cargo test --workspace` passes.

## Verification notes

Use only test/backup files during this phase. Do not point the CLI at the local cloud-synced vault as a write target; this phase should not contain write code at all.
