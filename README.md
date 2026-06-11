# Anahtar

Anahtar is a personal KeePass/KDBX CLI experiment. The current focus is a safe CLI-first workflow that can later be reused by a desktop GUI.

## Current status

Completed:

- Phase 1: read-only CLI
- Phase 2: non-destructive KDBX4.1 `upgrade` / save-as flow
- Phase 3: minimal write commands with save-as
- Phase 4: daily-use CLI polish
  - TOML config/default vault
  - consistent `--vault` CLI shape
  - clipboard copy
  - password generation
  - TOTP retrieval

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

## Config and default vault

Set a default vault:

```bash
cargo run -q -p anahtar-cli -- config set vault test-vaults/generated/phase3-base.kdbx
```

`config set vault` requires the vault file to exist and stores its canonical absolute path. This makes the default vault stable when Anahtar is run from another working directory.

Show config:

```bash
cargo run -q -p anahtar-cli -- config show
```

Set defaults:

```bash
cargo run -q -p anahtar-cli -- config set generator-length 32
cargo run -q -p anahtar-cli -- config set clipboard-clear-after 30
```

All vault commands accept `--vault <path>` to override the default.

## Clipboard behavior

Clipboard commands use the operating system clipboard through `arboard`, so they require a desktop/session environment with clipboard access. Headless CI or SSH-only environments may not support clipboard commands.

Copy commands intentionally wait until the clear timeout expires. Anahtar only clears the clipboard if it still contains the exact value Anahtar copied.

For faster shell workflows, lower the timeout:

```bash
cargo run -q -p anahtar-cli -- config set clipboard-clear-after 10
```

## CLI examples

Inspect header without unlocking:

```bash
cargo run -q -p anahtar-cli -- inspect --vault test-vaults/generated/phase3-base.kdbx
```

List entries:

```bash
cargo run -q -p anahtar-cli -- list --vault test-vaults/generated/phase3-base.kdbx
```

Search entries:

```bash
cargo run -q -p anahtar-cli -- search --vault test-vaults/generated/phase3-base.kdbx github
```

Show an entry without revealing its password:

```bash
cargo run -q -p anahtar-cli -- show --vault test-vaults/generated/phase3-base.kdbx "Github Test"
```

Copy fields:

```bash
cargo run -q -p anahtar-cli -- copy-password --vault test-vaults/generated/phase3-base.kdbx "Github Test"
cargo run -q -p anahtar-cli -- copy-username --vault test-vaults/generated/phase3-base.kdbx "Github Test"
cargo run -q -p anahtar-cli -- copy-url --vault test-vaults/generated/phase3-base.kdbx "Github Test"
```

Generate a password:

```bash
cargo run -q -p anahtar-cli -- generate
cargo run -q -p anahtar-cli -- generate --length 40
cargo run -q -p anahtar-cli -- generate --copy
```

Show a TOTP code without exposing the OTP URI:

```bash
cargo run -q -p anahtar-cli -- totp --vault path/to/vault-with-otp.kdbx "Entry Title"
```

Add an entry using save-as:

```bash
cargo run -q -p anahtar-cli -- add \
  --vault test-vaults/generated/phase3-base.kdbx \
  --output test-vaults/generated/outputs/phase3-add.kdbx \
  --group 'General/Web' \
  --title 'Anahtar Test Entry' \
  --username 'anahtar@example.com' \
  --generate-password \
  --url 'https://example.com'
```

Edit an entry using save-as:

```bash
cargo run -q -p anahtar-cli -- edit \
  --vault test-vaults/generated/outputs/phase3-add.kdbx \
  '<entry-uuid>' \
  --output test-vaults/generated/outputs/phase3-edit.kdbx \
  --username 'updated-anahtar@example.com'
```

Delete an entry by UUID using save-as:

```bash
cargo run -q -p anahtar-cli -- delete \
  --vault test-vaults/generated/outputs/phase3-edit.kdbx \
  '<entry-uuid>' \
  --output test-vaults/generated/outputs/phase3-delete.kdbx
```

Upgrade/save-as to KDBX4.1:

```bash
cargo run -q -p anahtar-cli -- upgrade \
  --vault path/to/input.kdbx \
  --output path/to/output.kdbx
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
- `docs/phase-plans/phase-4-daily-use-cli.md`
