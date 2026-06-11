# Phase 6 — GUI Alpha

Status: ready to plan / next implementation phase

## Goal

Build the first Anahtar desktop GUI alpha on top of the Phase 5 CLI product capabilities and the Phase 5.5 UI-readiness boundary.

The GUI should not invent password-manager behavior. It should depend on `anahtar-app::AnahtarService` for application workflows and let `anahtar-core` own KDBX-specific logic, selector behavior, and write-safety mechanics.

## Current foundation

Phase 5.5 created the backend boundary Phase 6 should use:

```text
GUI/Tauri command layer
  -> anahtar-app::AnahtarService
    -> anahtar-core
```

Important files:

- `crates/anahtar-app/src/lib.rs` — stateless GUI/CLI service facade.
- `docs/gui-api-contract.md` — GUI/backend contract.
- `crates/anahtar-core/src/lib.rs` — public re-export surface.
- `crates/anahtar-cli/src/commands.rs` — reference product flows from CLI.
- `docs/threat-model.md` — security model to preserve in GUI.

## Preconditions

Phase 6 may start because these are now true:

- CLI productization is complete.
- Credential model supports password-only and password + key-file vaults.
- Safe in-place write model exists and is tested.
- Save-as write flows remain available.
- Selectors are explicit and duplicate-safe.
- Basic group organization and entry move are supported.
- Audit result types exist and avoid secret values.
- `anahtar-app` provides a stateless service facade.
- GUI API contract is documented.
- CI covers fmt, clippy, tests, examples, synthetic vault generation, install smoke, and completion smoke.

## Candidate stack

Tauri remains the preferred first GUI stack because it can reuse Rust directly and package desktop apps.

Target order:

1. macOS local development/package first.
2. Linux/Windows later after platform-specific file replacement and packaging checks.

Frontend technology should be chosen for small, maintainable UI rather than framework novelty. A conservative Tauri + TypeScript frontend is acceptable.

## Architecture rules

- GUI commands call `anahtar-app`, not low-level KDBX routines directly.
- `anahtar-core` owns KDBX open/save/traversal/write verification.
- GUI owns visual state, unlock modal state, and platform clipboard behavior.
- GUI must not persist master passwords.
- GUI may persist vault/key-file paths as user-local config.
- GUI must show backup paths after successful in-place writes.
- GUI write actions must use `WriteMode::InPlace` or `WriteMode::SaveAs`, never custom file replacement.
- GUI should use UUID selectors after a user selects an entry from a list.

## Alpha scope

Initial GUI features:

- Select/configure vault path.
- Select/configure optional key-file path.
- Unlock vault with master password.
- List entries.
- Search entries.
- View safe entry details without password by default.
- Reveal password only through explicit action.
- Copy username/password/TOTP with GUI-owned clipboard clear behavior.
- Add entry using safe write model.
- Edit entry using safe write model.
- Delete entry with confirmation using safe write model.
- List groups.
- Move entry to group if UI complexity remains manageable.
- Show audit findings.
- Display write reports, including backup path.

## Alpha non-goals

- Browser extension.
- Mobile app.
- Cloud sync engine.
- Shared vault collaboration.
- Background unlock daemon.
- Biometric unlock.
- Passkeys.
- Cross-platform installer polish beyond local macOS packaging.
- Full settings/preferences UX beyond what alpha needs.

## Workstream A — Tauri scaffold

Tasks:

- [ ] Add GUI crate/app scaffold under `crates/anahtar-gui` or `apps/anahtar-gui`.
- [ ] Wire Rust side to depend on `anahtar-app`.
- [ ] Add minimal frontend shell.
- [ ] Add local dev command documentation.
- [ ] Ensure workspace CI does not accidentally require GUI platform dependencies unless intended.

Acceptance criteria:

- [ ] GUI app launches locally.
- [ ] GUI can call a trivial Rust command.
- [ ] Existing CLI/core/app CI remains green.

## Workstream B — Unlock and vault session model

Tasks:

- [ ] Implement vault path picker/input.
- [ ] Implement optional key-file path picker/input.
- [ ] Implement master password unlock form.
- [ ] Call `AnahtarService::list` to validate unlock.
- [ ] Store unlocked session in memory only.
- [ ] Avoid logging password or protected fields.

Acceptance criteria:

- [ ] Password-only generated test vault unlocks.
- [ ] Invalid password/key-file errors are user-readable.
- [ ] Master password is not written to config or logs.

## Workstream C — Read/search/detail workflows

Tasks:

- [ ] Entry list view using `EntrySummary`.
- [ ] Search box using `AnahtarService::search`.
- [ ] Entry detail panel using `AnahtarService::show`.
- [ ] Default detail view masks password/protected fields.
- [ ] Explicit reveal action requests revealed detail.

Acceptance criteria:

- [ ] User can browse and search entries.
- [ ] Password is hidden by default.
- [ ] GUI uses UUID selector after list selection.

## Workstream D — Clipboard and TOTP

Tasks:

- [ ] Copy username.
- [ ] Copy password.
- [ ] Copy/display TOTP.
- [ ] Implement blocking or timer-based clear behavior appropriate for GUI.
- [ ] Clear only if clipboard still contains Anahtar's value.

Acceptance criteria:

- [ ] Copy flows work without printing secrets.
- [ ] Clipboard clear behavior is documented and visible to user.

## Workstream E — Write workflows

Tasks:

- [ ] Add entry form.
- [ ] Edit entry form.
- [ ] Delete entry confirmation.
- [ ] Use `AnahtarService` with `WriteMode::InPlace` for default writes.
- [ ] Offer save-as behavior later only if alpha UI remains simple.
- [ ] Display `WriteReport` after writes, including backup path.
- [ ] Refresh list after successful write.

Acceptance criteria:

- [ ] Add/edit/delete work against generated test vault copy.
- [ ] Backups are created for in-place writes.
- [ ] Failed writes do not destroy original vault.
- [ ] Delete requires explicit confirmation.

## Workstream F — Groups and audit

Tasks:

- [ ] Group list view.
- [ ] Entry move-to-group flow if simple enough for alpha.
- [ ] Audit findings panel using `AnahtarService::audit`.
- [ ] Ensure audit UI never shows secret values.

Acceptance criteria:

- [ ] User can inspect groups.
- [ ] User can see actionable audit findings.

## Workstream G — Packaging and product polish

Tasks:

- [ ] Document macOS local package command.
- [ ] Add basic app name/icon placeholder.
- [ ] Ensure no private vaults/assets enter packaged app.
- [ ] Document known alpha limitations.

Acceptance criteria:

- [ ] GUI can be packaged locally for macOS.
- [ ] README or GUI doc explains alpha install/run limitations.

## Code review findings to carry into Phase 6

- `anahtar-app::WriteMode::InPlace` should keep target path unambiguous: the service method path is the in-place target, and `WriteMode::InPlace` only carries `backup_dir`.
- `anahtar-core` now has a clean public module facade, but most implementation still lives in `internal.rs`. This is acceptable for Phase 6 because GUI depends on `anahtar-app`, not internal modules. A deeper physical core split can be done later if core churn grows.
- CLI remains the best reference for confirmation policy, prompt ordering, and write report display.
- GUI should avoid long-lived service state until there is a deliberate vault-session design.

## Exit criteria

- GUI can perform daily read/copy workflows through `anahtar-app`.
- GUI write actions use the same safety model as CLI.
- GUI can be packaged locally for macOS.
- GUI does not duplicate KDBX logic outside `anahtar-core`/`anahtar-app`.
- GUI follows `docs/gui-api-contract.md` credential and secret-handling rules.
