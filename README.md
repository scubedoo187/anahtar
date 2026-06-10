# Anahtar

Anahtar is a personal KeePass/KDBX CLI experiment. The current focus is a safe CLI-first workflow that can later be reused by a desktop GUI.

## Current status

Completed:

- Phase 1: read-only CLI
  - `inspect`
  - `list`
  - `search`
  - `show`
- Phase 2: non-destructive KDBX4.1 `upgrade` / save-as flow
- Phase 3: minimal write commands with save-as
  - `add`
  - `edit`
  - `delete`

All write commands currently write a new KDBX4.1 output file and do **not** modify the input vault in place.

## Safety model

- Do not commit real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.
- `assets/` is ignored and is intended only for local/private vault copies.
- Generated test vault binaries are ignored and can be regenerated locally.
- Passwords are prompted through TTY, not accepted as plain CLI arguments.
- Write commands use temp-file save + reopen verification before final rename.
- Existing output files are protected unless `--force` is passed.
- `input == output` is rejected for write commands.

## Build and test

```bash
cargo fmt --all
cargo test --workspace
```

Run the CLI through Cargo:

```bash
cargo run -q -p anahtar-cli -- --help
```

## Generate synthetic Phase 3 test vault

The generated test vault contains only fake credentials and uses the public test password `testpass`.

```bash
cargo run -q -p anahtar-core --example generate_phase3_vault
```

Output:

```text
test-vaults/generated/phase3-base.kdbx
```

This path is ignored by git.

## CLI examples

Inspect header without unlocking:

```bash
cargo run -q -p anahtar-cli -- inspect test-vaults/generated/phase3-base.kdbx
```

List entries:

```bash
cargo run -q -p anahtar-cli -- list test-vaults/generated/phase3-base.kdbx
```

Search entries:

```bash
cargo run -q -p anahtar-cli -- search test-vaults/generated/phase3-base.kdbx github
```

Show an entry without revealing its password:

```bash
cargo run -q -p anahtar-cli -- show test-vaults/generated/phase3-base.kdbx "Github Test"
```

Add an entry using save-as:

```bash
cargo run -q -p anahtar-cli -- add \
  test-vaults/generated/phase3-base.kdbx \
  --output test-vaults/generated/outputs/phase3-add.kdbx \
  --group 'General/Web' \
  --title 'Anahtar Test Entry' \
  --username 'anahtar@example.com' \
  --password-prompt \
  --url 'https://example.com'
```

Edit an entry using save-as:

```bash
cargo run -q -p anahtar-cli -- edit \
  test-vaults/generated/outputs/phase3-add.kdbx \
  '<entry-uuid>' \
  --output test-vaults/generated/outputs/phase3-edit.kdbx \
  --username 'updated-anahtar@example.com'
```

Delete an entry by UUID using save-as:

```bash
cargo run -q -p anahtar-cli -- delete \
  test-vaults/generated/outputs/phase3-edit.kdbx \
  '<entry-uuid>' \
  --output test-vaults/generated/outputs/phase3-delete.kdbx
```

## Project layout

```text
crates/
  anahtar-core/   # KDBX core operations and tests
  anahtar-cli/    # CLI wrapper
docs/             # roadmap, phase plans, research notes
journals/         # dated work log
spikes/           # research/prototype code
test-vaults/      # README plus ignored local/generated fixtures
```

## Planning docs

- `docs/anahtar-roadmap.md`
- `docs/anahtar-final-goals.md`
- `docs/phase-plans/phase-1-readonly-cli.md`
- `docs/phase-plans/phase-2-upgrade-save-as.md`
- `docs/phase-plans/phase-3-minimal-write.md`
