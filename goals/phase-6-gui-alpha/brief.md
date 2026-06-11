# Phase 6 GUI Alpha

## Outcome

Build a macOS-first Tauri GUI alpha for Anahtar on top of `anahtar-app` without duplicating KDBX logic.

## Context

Anahtar has completed CLI productization and UI-readiness cleanup:

- `anahtar-core` owns KDBX domain operations, DTOs, selectors, audit, groups, TOTP, and safe write helpers.
- `anahtar-app::AnahtarService` is the stateless GUI/CLI application facade.
- `anahtar-cli` remains the reference product surface for prompt ordering, confirmation policy, and write report display.
- `docs/gui-api-contract.md` defines the GUI/backend boundary.
- `docs/phase-plans/phase-6-gui-alpha.md` is the canonical Phase 6 phase plan.
- GUI should be macOS-first and later extensible to Windows/Linux.

## Constraints

- GUI commands must call `anahtar-app`, not low-level KDBX traversal/write code directly.
- Do not store master passwords in config, logs, crash reports, or long-lived service state.
- Key-file paths may be treated as user-local config; key-file contents and master passwords are secret.
- Write actions must use `AnahtarService` and `WriteMode`, never custom file replacement.
- Passwords/protected fields must be hidden by default and revealed only by explicit user action.
- Clipboard clear must only clear if clipboard still contains Anahtar's copied value.
- Existing CLI/core/app behavior and CI must stay green.
- Public repo hygiene remains mandatory: no real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.

## Non-Goals

- Browser extension.
- Mobile app.
- Cloud sync engine.
- Shared vault collaboration.
- Background unlock daemon.
- Biometric unlock.
- Passkeys.
- Full cross-platform installer polish beyond local macOS packaging.
- Replacing the KDBX backend or implementing custom crypto.

## Ask Before

Ask the user before:

- Installing or adding a large frontend framework beyond minimal Tauri + TypeScript needs.
- Adding networked services, telemetry, analytics, crash reporting, or auto-update infrastructure.
- Changing credential storage policy or persisting any secret material.
- Performing destructive file operations outside generated/test vault copies.
- Altering `anahtar-app` or `anahtar-core` APIs in a way that breaks CLI behavior.
- Adding Windows/Linux packaging requirements to the alpha scope.

## Done Means

Phase 6 GUI alpha is done when Anahtar has a local macOS-launchable and packageable GUI that can unlock a generated/test vault, list/search/show entries, perform explicit reveal/copy flows, execute safe add/edit/delete writes through `anahtar-app`, show backup/write report information, expose basic group/audit views, and pass the required workspace and GUI verification checks with evidence recorded in `progress.jsonl`.
