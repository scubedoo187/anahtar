# Anahtar test vaults

These vaults are synthetic development fixtures for Anahtar write-command testing.

## Password

All generated Phase 3 test vaults use:

```text
testpass
```

This password is intentionally public because these vaults must never contain real secrets.

## Purpose

Use these vaults to test:

- `add`
- `edit`
- `delete`
- KDBX4.1 save-as writes
- output reopen verification
- Strongbox compatibility smoke tests

## Safety rules

- Never put real credentials in this directory.
- Do not use the personal cloud-synced vault for iterative write-command development.
- Generated output files should be treated as disposable.
- The personal vault may only be used for final smoke testing after generated test-vault flows pass.

## Planned structure

```text
test-vaults/
  README.md
  generated/
    phase3-base.kdbx
    outputs/
      phase3-add.kdbx
      phase3-edit.kdbx
      phase3-delete.kdbx
  strongbox/
    phase3-strongbox-base.kdbx
```
