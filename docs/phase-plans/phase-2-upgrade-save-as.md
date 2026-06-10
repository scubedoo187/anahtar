# Phase 2 plan — Upgrade/save-as CLI

This phase adds the first write path, limited to explicit non-destructive KDBX4.1 save-as upgrade.

## Goal

Implement `anahtar upgrade <input> <output>` so KDBX3/KDBX4.0/KDBX4.1 inputs can be written to a new KDBX4.1 output file without modifying the input.

## Implementation checklist

- [x] Add `upgrade` command to CLI.
- [x] Require explicit input and output paths.
- [x] Prompt password with hidden input.
- [x] Open input database through `anahtar-core`.
- [x] Count input groups and entries.
- [x] Warn when input is not already KDBX4.1.
- [x] Set output format to KDBX4.1.
- [x] Save to a temp file.
- [x] Flush/sync temp file.
- [x] Reopen temp file with same password.
- [x] Compare group/entry counts.
- [x] Rename temp file to final output.
- [x] Refuse to overwrite existing output unless `--force` is passed.
- [x] Add `--dry-run`.
- [x] Print Strongbox manual verification instructions.

## Exit criteria

- [x] `anahtar upgrade` creates a KDBX4.1 output file without modifying input.
- [x] Output file reopens with same password.
- [x] Group/entry counts match.
- [x] Strongbox opens generated output manually.
- [x] Existing output path is protected unless `--force` is passed.

## Hardening completed

- [x] Reject `input == output` before unlock/save to prevent accidental in-place upgrade or `--force` deleting the input.
- [x] Refuse to proceed if the internal temp output path already exists.
- [x] Cleanup temp file on save/reopen/count/rename failure.
- [x] Added regression test for same input/output rejection.

## Notes

Phase 2 validation was completed manually by the user against `assets/private-vault.backup.kdbx` → `assets/private-vault.phase2.kdbx`.

Observed result:

- dry run produced expected KDBX4.0 → KDBX4.1 warning and counts.
- real upgrade produced KDBX4.1 output.
- output inspect reported KDBX 4.1.
- duplicate output was protected by default.
- `--force` regenerated output successfully.

Phase 2 is now formally complete.
