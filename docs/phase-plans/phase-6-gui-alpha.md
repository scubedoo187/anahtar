# Phase 6 — GUI Alpha

Status: accepted / ready to implement

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

Frontend choice for alpha: Tauri + TypeScript with the smallest practical UI stack. Avoid framework novelty and keep the first GUI simple. The GUI app should live under `apps/anahtar-gui` so product apps remain separate from reusable Rust crates under `crates/`.

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

- [ ] Add GUI app scaffold under `apps/anahtar-gui`.
- [ ] Keep reusable Rust crates under `crates/`; do not make the GUI app the service boundary.
- [ ] Wire Tauri Rust side to depend on `anahtar-app`.
- [ ] Add minimal frontend shell.
- [ ] Add a trivial command, e.g. `backend_status`, to prove frontend ↔ Rust IPC.
- [ ] Add local dev command documentation.
- [ ] Keep root workspace CI independent from GUI platform dependencies unless explicitly added.

Acceptance criteria:

- [ ] GUI app launches locally on macOS.
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

## Narrow implementation order

Implement Phase 6 as small, reviewable slices. Each slice should preserve existing CLI/core behavior.

### Slice 1 — GUI scaffold only

- Create `apps/anahtar-gui` Tauri + TypeScript app.
- Add a minimal window with an Anahtar title/status area.
- Add a Rust command that returns backend/app version/status.
- Document local dev command.
- Do not implement vault unlock yet.

Validation:

- [ ] GUI launches locally.
- [ ] Frontend can call the trivial backend command.
- [ ] Existing workspace fmt/test/clippy/examples still pass.

### Slice 2 — Backend command surface skeleton

- Add Tauri commands that wrap `anahtar-app` read-only operations at the boundary:
  - inspect,
  - unlock/list validation,
  - search,
  - show safe detail.
- Define frontend TypeScript types matching current DTOs.
- Add safe error mapping for user display.

Validation:

- [ ] Generated test vault can be inspected/listed via backend command.
- [ ] Invalid unlock returns a safe generic error.
- [ ] No secret fields are logged.

### Slice 3 — Unlock/session UI

- Add vault path input/picker.
- Add optional key-file path input/picker.
- Add master password field.
- Validate unlock with `AnahtarService::list`.
- Keep password in memory only.

Validation:

- [ ] Generated test vault unlocks.
- [ ] Wrong password fails safely.
- [ ] Password is not persisted.

### Slice 4 — Read/search/detail UI

- Add entry list.
- Add search.
- Add detail panel.
- Use UUID selectors after list selection.
- Hide password and protected fields by default.
- Add explicit reveal action.

Validation:

- [ ] User can browse/search/show entries.
- [ ] Password remains hidden unless explicitly revealed.

### Slice 5 — Clipboard/TOTP UI

- Add copy username/password/url actions.
- Add TOTP display/copy if entry supports it.
- Implement GUI-owned clear timer.
- Clear only if clipboard still contains Anahtar's copied value.

Validation:

- [ ] Copy actions do not print secrets.
- [ ] Clipboard clear behavior works and is visible to user.

### Slice 6 — Minimal write UI

- Add entry form.
- Edit entry form.
- Delete entry confirmation.
- Use `AnahtarService` with `WriteMode::InPlace { backup_dir }`.
- Display write report and backup path.
- Refresh list after writes.

Validation:

- [ ] Add/edit/delete work on generated-vault copy.
- [ ] Backups are created.
- [ ] Failed writes preserve original vault.

### Slice 7 — Groups/audit

- Add group list.
- Add move-to-group if UI remains simple.
- Add audit findings panel.

Validation:

- [ ] User can inspect groups.
- [ ] User can view non-secret audit findings.

### Slice 8 — macOS alpha packaging

- Add package command docs.
- Add app name/icon placeholder.
- Ensure package excludes local/private vaults.
- Document alpha limitations.

Validation:

- [ ] GUI packages locally on macOS.
- [ ] README or GUI doc explains alpha run/package limitations.

## Exit criteria

- GUI can perform daily read/copy workflows through `anahtar-app`.
- GUI write actions use the same safety model as CLI.
- GUI can be packaged locally for macOS.
- GUI does not duplicate KDBX logic outside `anahtar-core`/`anahtar-app`.
- GUI follows `docs/gui-api-contract.md` credential and secret-handling rules.
