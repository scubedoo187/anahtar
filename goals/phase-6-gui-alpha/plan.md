# Plan: Phase 6 GUI Alpha

## Solution Overview

Build the first Anahtar desktop GUI as a macOS-first Tauri + TypeScript app under `apps/anahtar-gui`. The GUI will be a thin interactive product shell over `anahtar-app::AnahtarService`; it will not reimplement KDBX parsing, selector resolution, audit logic, or write safety.

The alpha should prove the complete daily-use loop: unlock a vault, browse/search entries, inspect details safely, copy/reveal secrets explicitly, run minimal writes with backup reporting, inspect groups, view audit findings, and package locally for macOS.

## Why This Approach

Anahtar already has a productized CLI and a Phase 5.5 service boundary. Starting the GUI on `anahtar-app` preserves the safe write model and avoids a second password-manager implementation. Tauri is appropriate because it keeps Rust close to the app boundary, can package desktop apps, and supports a small TypeScript frontend without forcing a large application framework.

Keeping the GUI under `apps/anahtar-gui` separates product apps from reusable Rust crates under `crates/`. This preserves the workspace architecture while allowing GUI-specific tooling and packaging to evolve independently.

## How It Will Work

The intended call path is:

```text
Tauri frontend
  -> Tauri command handlers
    -> anahtar-app::AnahtarService
      -> anahtar-core
```

The frontend owns UI state, unlock form state, clipboard interaction, and visual confirmations. The Rust command layer maps requests into `VaultCredentials`, `EntrySelector`, and `WriteMode`, then calls `AnahtarService`. The backend returns DTOs or safe display errors. Master passwords are accepted only for the current in-memory session and must not be persisted.

Default write actions use `WriteMode::InPlace { backup_dir }`; the service method path is the target vault. Save-as can be deferred unless needed for alpha usability. Every successful write should show `WriteReport` information, especially backup path.

## Slices

| Slice | Purpose | Main files or systems | Done when | Risks |
| --- | --- | --- | --- | --- |
| 1 | GUI scaffold only | `apps/anahtar-gui`, Tauri config, minimal frontend | App launches locally and calls `backend_status` | Tauri setup may introduce workspace/CI dependency friction |
| 2 | Backend command skeleton | Tauri command handlers, TS DTO types | inspect/list validation/search/show commands work on generated vault | Error mapping could leak too much detail if not reviewed |
| 3 | Unlock/session UI | vault/key-file inputs, password field, in-memory session | generated vault unlocks and wrong password fails safely | accidental persistence/logging of password |
| 4 | Read/search/detail UI | entry list, search, detail panel | browse/search/show works; password hidden by default | reveal flows may overexpose protected fields |
| 5 | Clipboard/TOTP UI | frontend clipboard adapter/timers, TOTP command | copy flows work and clear only owned values | clipboard APIs differ by platform/session |
| 6 | Minimal write UI | add/edit/delete forms, write report UI | add/edit/delete work on test copy and show backups | destructive actions need clear confirmation |
| 7 | Groups/audit | group list, move if simple, audit panel | groups/audit visible without secret leakage | scope creep into full group management |
| 8 | macOS alpha packaging | Tauri build/package docs, app metadata | app packages locally and limitations are documented | packaging can distract from alpha functionality |

## Sequencing

Implement slices in order. Do not start write UI before read/unlock flows are stable. Do not package before the alpha can demonstrate at least read/copy workflows. If Tauri setup requires significant dependency or CI changes, pause after Slice 1 for review.

## Phase Boundaries

This goal ends at a local macOS GUI alpha. Create a later goal for any of the following:

- production installer/signing/notarization,
- Windows/Linux packaging,
- biometric/keychain unlock assist,
- browser extension/mobile/sync,
- major redesign of `anahtar-core` internals,
- broad UX redesign after alpha feedback.

## Steering Notes

- Prefer a boring, clear UI over visual polish.
- Make dangerous actions explicit and reversible through backups.
- Use generated/test vault copies for write validation.
- Keep CLI behavior stable; CLI is still a product surface.
- Treat `docs/gui-api-contract.md` as binding unless explicitly revised.

## Acceptance Criteria

- [ ] `apps/anahtar-gui` exists with a documented local dev command and a locally launchable Tauri window. Evidence: command output and/or screenshot path in `progress.jsonl`.
- [ ] Frontend can call a trivial Rust backend command. Evidence: command name, returned payload, and screenshot/log path.
- [ ] GUI backend command surface wraps `anahtar-app` for inspect, unlock/list validation, search, and safe show detail. Evidence: command names and generated-vault results.
- [ ] Unlock UI supports vault path, optional key-file path, and master password without persisting the password. Evidence: manual check plus code reference.
- [ ] Entry list/search/detail workflows work with generated test vault data. Evidence: screenshot paths or manual check entries.
- [ ] Password/protected fields are hidden by default and revealed only by explicit action. Evidence: manual check and relevant UI/code reference.
- [ ] Copy username/password/TOTP flows do not print secrets and clear only if clipboard still contains Anahtar's value. Evidence: manual check description.
- [ ] Add/edit/delete use `AnahtarService` with `WriteMode::InPlace { backup_dir }`, show write reports, and create backups. Evidence: generated-vault copy before/after paths and backup path.
- [ ] Delete requires explicit confirmation. Evidence: screenshot or manual check.
- [ ] Group list and audit findings are available without secret leakage. Evidence: manual check and example output/screenshot path.
- [ ] GUI can be packaged locally for macOS. Evidence: package command, status, and artifact path.
- [ ] Existing workspace checks pass. Evidence: command results appended to `progress.jsonl`.

## Required Evidence

Append evidence to `goals/phase-6-gui-alpha/progress.jsonl` after every meaningful slice. Each evidence record should include timestamp, slice, command or manual check, result, and artifact path when available.
