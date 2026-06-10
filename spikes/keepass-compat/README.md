# keepass compatibility spike

Purpose: validate whether Rust `keepass = 0.13.8` is viable for Anahtar's KDBX core.

## What this spike tests

- Open official `sseemayer/keepass-rs` fixture vaults.
- Count groups/entries and print sample titles.
- Native save/reopen for KDBX4.1 fixtures.
- Confirm KDBX4.1 writer XML markers that upstream says KeePassXC requires.
- Explicit upgrade save experiment: KDBX3/KDBX4.0 -> set config version to KDBX4.1 -> mutate -> save -> reopen.

## Run

```bash
cd spikes/keepass-compat
./fetch-fixtures.sh
cargo run -- fixtures
```

Generic one-file KDBX4.1 upgrade spike:

```bash
cargo run --bin upgrade_asset -- <input.kdbx> <output.kdbx>
```

Synthetic fixture passwords are embedded in `src/main.rs`; no real secrets are used.

## Current result

Last run: 2026-06-10

```text
SUMMARY: 7 fixtures passed basic open/save/reopen checks
```

Key finding:

- `keepass` can read tested KDBX3, KDBX4.0, KDBX4.1 files.
- Native `Database::save` only supports `DatabaseVersion::KDB4(1)`.
- KDBX3 and KDBX4.0 can be experimentally upgraded to KDBX4.1 by setting `db.config.version = DatabaseVersion::KDB4(1)` before save; the resulting file reopens via `keepass`.
- This does **not yet prove Strongbox compatibility**. Manual app validation is still required.

Generated outputs are under `spikes/keepass-compat/out/`.
