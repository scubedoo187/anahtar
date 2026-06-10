# Anahtar final goals

This document records the expected end-state once the full roadmap is complete.

## Product goal

Anahtar should become a personal, free-to-use KeePass-compatible password manager that can replace the user's current paid paid password-manager workflow for their own vault.

Expected final workflow:

- Open local or synced `.kdbx` vaults.
- Unlock with the master password.
- Optionally use OS keychain/biometric assist later.
- Search entries quickly.
- View entry details safely.
- Copy username/password/TOTP with secure clipboard behavior.
- Add, edit, and delete entries.
- Save safely with backup, temp file, atomic replacement, and reopen verification.
- Continue using an open KeePass-compatible file format instead of a proprietary subscription vault.

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
- Treat `assets/private-vault.backup.kdbx` as backup/reference input.
- Treat `assets/private-vault.kdbx41.test.kdbx` as the first working KDBX4.1 test vault.
- Prefer save-as over in-place writes.
- Before any in-place write is allowed, require:
  - timestamped backup,
  - temp file in same directory,
  - flush/sync where possible,
  - atomic rename,
  - reopen verification,
  - clear user confirmation.

## Success definition

The project can be considered successful when the user can stop paying for the current paid password-manager workflow while still retaining:

- reliable access to all important credentials,
- Strongbox/KeePass compatibility,
- safe local backups,
- practical desktop UX,
- no recurring service dependency.
