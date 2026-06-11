# Codex Goal Prompt: Phase 6 GUI Alpha

After every critical document in this folder is approved with Plannotator, paste or set this goal:

```text
/goal Build the Phase 6 macOS-first Anahtar GUI alpha.

Use `goals/phase-6-gui-alpha/` as the durable source of truth. Read `brief.md`, `plan.md`, `verification.md`, and `blockers.md` before changing code. Append concrete progress and proof to `goals/phase-6-gui-alpha/progress.jsonl` after each meaningful step; do not rewrite that file.

Outcome: create a Tauri + TypeScript GUI under `apps/anahtar-gui` that uses `anahtar-app::AnahtarService` for password-manager workflows and does not duplicate KDBX logic. The alpha must be able to launch locally on macOS, call Rust backend commands, unlock a generated/test vault, list/search/show entries safely, perform explicit reveal/copy flows, run minimal add/edit/delete writes through `WriteMode::InPlace { backup_dir }`, show backup/write report information, expose basic groups/audit views, and package locally for macOS.

Relevant files and docs:
- `goals/phase-6-gui-alpha/brief.md`
- `goals/phase-6-gui-alpha/plan.md`
- `goals/phase-6-gui-alpha/verification.md`
- `goals/phase-6-gui-alpha/blockers.md`
- `docs/phase-plans/phase-6-gui-alpha.md`
- `docs/gui-api-contract.md`
- `docs/threat-model.md`
- `crates/anahtar-app/src/lib.rs`
- `crates/anahtar-core/src/lib.rs`
- `crates/anahtar-cli/src/commands.rs`

Constraints:
- GUI commands call `anahtar-app`, not low-level KDBX routines directly.
- Do not store master passwords in config, logs, crash reports, or long-lived service state.
- Hide passwords/protected fields by default; reveal only through explicit user action.
- Use `AnahtarService`/`WriteMode` for mutations; never custom file replacement.
- Test writes only on generated/test vault copies unless the user explicitly approves otherwise.
- Preserve existing CLI/core/app behavior.
- Keep public repo hygiene: no real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.

Non-goals: browser extension, mobile app, cloud sync, shared vault collaboration, background unlock daemon, biometric unlock, passkeys, full cross-platform installer polish, KDBX backend replacement, custom crypto.

Implementation slices: follow `plan.md` in order: scaffold, backend command skeleton, unlock/session UI, read/search/detail UI, clipboard/TOTP, minimal write UI, groups/audit, macOS alpha packaging. Stop rather than stretching this goal into production signing/notarization or cross-platform packaging.

Verification: run and record the checks from `verification.md`, including workspace fmt/test/clippy/examples, synthetic vault generation, CLI install/version/completion smoke, GUI local launch, and GUI package command when available. Every acceptance item needs observable evidence in `progress.jsonl`.

Ask before doing anything listed in `blockers.md`, especially adding large frontend frameworks, persisting secrets, touching real vault/key files, broad dependency/tooling installation, force pushes/history rewrites, or expanding scope beyond macOS GUI alpha.

Do not mark the goal complete until all acceptance criteria in `plan.md` are backed by evidence and any remaining limitations are explicitly documented for the user.
```
