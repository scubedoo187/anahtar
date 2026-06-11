# Anahtar final goals

This document records Anahtar's intended product end-state after the CLI and GUI roadmaps mature enough to replace the user's current paid password-manager workflow.

## Product goal

Anahtar should become a personal, free-to-use KeePass/KDBX-compatible password manager for the user's own vaults.

The product should have two mature surfaces built on shared Rust code:

1. A capable CLI password manager.
2. A desktop GUI for daily interactive use.

Anahtar should preserve the advantages of an open KeePass-compatible file format while reducing dependence on proprietary subscription password-manager services.

## Current strategic target

The CLI is now productized enough to stand on its own. The next strategic target is a **macOS-first GUI alpha** that proves Anahtar can become a comfortable daily driver, not only a CLI tool.

Phase 6 should focus on daily usability and safe interaction patterns rather than adding new password-manager primitives. The GUI should expose the capabilities already proven by the CLI and `anahtar-app`.

## Expected final workflow

The user should be able to:

- Open local or synced `.kdbx` vaults.
- Unlock with master password and optional key-file material.
- Search entries quickly.
- Browse groups and entries visually.
- View entry details safely.
- Reveal sensitive fields only through explicit action.
- Copy username/password/URL/TOTP with secure clipboard behavior.
- Generate passwords.
- Add, edit, move, and delete entries.
- Manage basic groups.
- Run safe audit/check commands without printing secrets.
- Save safely with backup, temp file, replacement, and reopen verification.
- Keep using Strongbox/KeePass-compatible files when desired.
- Continue using an open file format instead of a proprietary subscription vault.

Possible later conveniences, not required for GUI alpha:

- OS keychain or biometric unlock assist.
- Release signing/notarization.
- Better cross-platform installers.
- Additional security-policy tooling such as `cargo audit`/`cargo deny`.

## CLI product goal

Anahtar CLI should remain a credible password-manager CLI product even after GUI work starts.

CLI success means:

- Default vault and credential material can be configured safely.
- Password-only and password + key-file workflows are supported.
- Read/copy/generate/TOTP workflows are practical for daily use.
- Write workflows operate safely on the configured vault by default with backup and verification.
- Save-as write workflows remain available for cautious/manual verification.
- Entry selectors are explicit, duplicate-safe, and scriptable.
- Basic group organization is possible, including list/add/rename/delete and entry move.
- Audit/check commands help find weak/reused/incomplete entries without exposing secrets.
- Installed binary usage is documented.
- CI verifies formatting, linting, tests, examples, install smoke, and completion generation.

## GUI product goal

The GUI should be a user-friendly surface over `anahtar-app`, not a separate KDBX implementation.

GUI alpha success means:

- The GUI can open/unlock/list/search/show entries through `anahtar-app::AnahtarService`.
- The GUI uses UUID selectors after list selection.
- Passwords and protected fields are hidden by default.
- Copy/reveal actions are explicit and avoid logging secrets.
- GUI write actions use the same safety model as CLI.
- Successful writes display backup/report information clearly.
- The GUI can be packaged locally for macOS.
- The GUI follows `docs/gui-api-contract.md`.

GUI maturity beyond alpha means:

- Daily read/copy/edit workflows feel faster than using the CLI for common tasks.
- The UI makes dangerous operations harder to perform accidentally.
- Error messages are actionable without revealing secrets.
- Windows/Linux packaging can follow after macOS alpha stabilizes.

## Technical goal

- Use Rust as the core implementation language.
- Keep `anahtar-core` as the KDBX domain logic layer.
- Keep `anahtar-app` as the stateless application-service boundary used by GUI and reusable by CLI where appropriate.
- Keep `anahtar-cli` responsible for CLI-specific concerns: parsing, prompts, config, terminal output, and CLI clipboard behavior.
- Use `keepass = 0.13.8` as the first KDBX backend.
- Read common KDBX versions, including KDBX4.0 and KDBX4.1.
- Standardize Anahtar writes on KDBX4.1 unless a better writer/backend is adopted later.
- Avoid custom cryptographic implementation.
- Keep output types UI-friendly, serializable where useful, and testable.

Accepted technical debt:

- `anahtar-core` currently has a public module facade while much of the physical implementation remains in `internal.rs`. This is acceptable while GUI alpha depends on `anahtar-app`; a deeper internal split can happen if core churn grows.

## Safety goal

- Never commit real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.
- Do not store master passwords in config, logs, crash reports, or long-lived service state.
- Treat key-file paths as user-local private config; protect key-file contents separately.
- Use safe write helpers for all mutations.
- Preserve save-as workflows for cautious/manual verification.
- For in-place writes, require:
  - timestamped backup,
  - temp save,
  - flush/sync where practical,
  - replacement through the core write helper,
  - temp and final reopen verification,
  - clear reporting of backup path and final target path,
  - confirmation for destructive operations.
- Clipboard clear must only clear if the clipboard still contains Anahtar's copied value.
- GUI must not bypass `anahtar-app`/`anahtar-core` mutation paths.

## Success definition

The project can be considered successful when the user can stop paying for the current paid password-manager workflow while still retaining:

- reliable access to all important credentials,
- Strongbox/KeePass compatibility,
- safe local backups,
- practical CLI workflows,
- practical desktop UX,
- no recurring service dependency.
