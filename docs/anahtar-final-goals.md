# Anahtar final goals

This document records the expected end-state once the full roadmap is complete.

## Product goal

Anahtar should become a personal, free-to-use KeePass-compatible password manager that can replace the user's current paid password-manager workflow for their own vault.

The product should have two mature surfaces:

1. A capable CLI password manager.
2. A desktop GUI built on the same Rust core.

## Expected final workflow

The user should be able to:

- Open local or synced `.kdbx` vaults.
- Unlock with master password and optional key-file material.
- Optionally use OS keychain/biometric assist later.
- Search entries quickly.
- Use stable selectors for scriptable CLI flows.
- View entry details safely.
- Copy username/password/TOTP with secure clipboard behavior.
- Generate passwords.
- Add, edit, move, and delete entries.
- Manage basic groups.
- Run safe audit/check commands without printing secrets.
- Save safely with backup, temp file, atomic replacement, and reopen verification.
- Continue using an open KeePass-compatible file format instead of a proprietary subscription vault.

## CLI product goal

Before GUI work, Anahtar CLI should be strong enough to stand on its own as a password-manager CLI product.

CLI success means:

- Default vault and credential material can be configured safely.
- Password-only and password + key-file workflows are supported if the backend allows it.
- Read/copy/generate/TOTP workflows are practical for daily use.
- Write workflows operate safely on the configured vault by default with backup and verification.
- Save-as write workflows remain available for cautious/manual verification.
- Entry selectors are explicit, duplicate-safe, and scriptable.
- Basic group organization is possible, including list/add/rename/delete and entry move.
- Audit/check commands help find weak/reused/incomplete entries without exposing secrets.
- Installed binary usage is documented.
- CI verifies formatting, linting, and tests.

## GUI product goal

The GUI should be a user-friendly surface over the same `anahtar-core` capabilities, not a separate implementation.

GUI success means:

- The GUI can open/unlock/search/list/detail/copy using core APIs.
- GUI write actions use the same safety model as CLI.
- The GUI can be packaged locally for macOS first.
- Windows/Linux packaging can follow after the macOS alpha is stable.

## Technical goal

- Use Rust as the core implementation language.
- Maintain `anahtar-core` as the single KDBX business logic layer.
- Reuse `anahtar-core` from both CLI and GUI.
- Use `keepass = 0.13.8` as the first core KDBX backend.
- Read common KDBX versions, including KDBX4.0 and KDBX4.1.
- Standardize Anahtar writes on KDBX4.1 unless a better writer/backend is adopted later.
- Avoid custom cryptographic implementation.
- Keep output types UI-friendly and testable.

## Safety goal

- Never modify the active cloud-synced vault directly during early development.
- Prefer save-as until safe in-place write has backup, temp save, atomic replacement, and reopen verification.
- Before any in-place write is considered product-ready, require:
  - timestamped backup,
  - temp file in same directory or otherwise safe filesystem policy,
  - flush/sync where practical,
  - atomic replacement where practical,
  - reopen verification,
  - clear reporting of backup path and final target path,
  - confirmation for destructive operations.

## Success definition

The project can be considered successful when the user can stop paying for the current paid password-manager workflow while still retaining:

- reliable access to all important credentials,
- Strongbox/KeePass compatibility,
- safe local backups,
- practical CLI workflows,
- practical desktop UX,
- no recurring service dependency.
