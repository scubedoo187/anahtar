# Phase 4 Settlement Before Phase 5

Status: complete

## Context

Phase 4 is functionally complete, but the CLI code and a few operational policies should be settled before starting Phase 5. The goal of this mini-phase is not to add user-facing features; it is to make the Phase 4 result safer and easier to extend.

The review found four items to settle:

1. `crates/anahtar-cli/src/main.rs` is too large after Phase 4.
2. Default vault path storage should be stable across working directories.
3. Clipboard clear behavior should be explicitly chosen and documented.
4. Clipboard commands should not make headless CI brittle.

## Approach

Implement a narrow cleanup/policy pass with only one intentional behavior change:

- **Behavior change:** `anahtar config set vault <path>` will store a canonical absolute path and fail clearly if the path does not exist.
- **No behavior change:** CLI command names, flags, output format, clipboard blocking clear, password generation, TOTP, and save-as write safety remain unchanged.
- **Refactor:** split CLI code into focused modules without changing the public CLI.
- **Docs:** document the settled vault path and clipboard policies.

## Files to modify

Expected code files:

- `crates/anahtar-cli/src/main.rs`
- `crates/anahtar-cli/src/cli.rs` — new
- `crates/anahtar-cli/src/config.rs` — new
- `crates/anahtar-cli/src/vault.rs` — new
- `crates/anahtar-cli/src/clipboard.rs` — new
- `crates/anahtar-cli/src/generator.rs` — new
- `crates/anahtar-cli/src/prompts.rs` — new
- `crates/anahtar-cli/src/printing.rs` — new

Expected docs/journal files:

- `README.md`
- `docs/phase-plans/phase-4-settlement-before-phase-5.md`
- `journals/2026-06-10.md` or the current date's journal

## Reuse

Reuse existing Phase 4 implementations rather than rewriting behavior:

- Existing config logic in `crates/anahtar-cli/src/main.rs`:
  - `AnahtarConfig`
  - `config_path`
  - `load_config`
  - `save_config`
  - `validate_config`
  - `handle_config`
- Existing vault/output safety helpers:
  - `resolve_vault`
  - `preflight_output`
  - `validate_uuid_selector`
  - `ensure_edit_has_change`
- Existing clipboard logic:
  - `copy_with_clear`
- Existing generator logic:
  - `generate_password`
- Existing prompt/printing helpers:
  - `prompt_password`
  - `prompt_entry_password_with_confirmation`
  - `confirm_delete`
  - `print_*`

## Decisions locked for this pass

### 1. Module split

Split by responsibility:

```text
main.rs        # parse CLI and dispatch
cli.rs         # Clap structs/enums
config.rs      # config model, load/save, validation, config subcommands
vault.rs       # vault resolution, output preflight, selector/edit validation
clipboard.rs   # clipboard copy and clear
 generator.rs  # password generation
prompts.rs     # password/delete prompts
printing.rs    # human/json output helpers
```

If `main.rs` still feels too large after the first split, add `commands.rs`; otherwise skip it to avoid unnecessary indirection.

### 2. Vault path storage

`config set vault <path>` will canonicalize the path before saving.

Rules:

- Path must exist and must be a file.
- Stored path is absolute/canonical.
- `config get vault` prints the stored canonical path.
- Default vault commands should work from a different current working directory.

### 3. Clipboard clear UX

Keep the current blocking behavior.

Reason: predictable clear behavior is more important than a non-blocking UX right now.

### 4. Clipboard CI/platform policy

Do not require a real clipboard in normal automated tests.

Rules:

- `cargo test --workspace` must not depend on clipboard availability.
- Clipboard remains manually verified on desktop environments.
- Future clipboard integration tests, if added, should be opt-in with an env var such as `ANAHTAR_TEST_CLIPBOARD=1`.

## Steps

- [x] Move Clap CLI types from `main.rs` to `cli.rs`.
- [x] Move config types/helpers and config subcommand handling to `config.rs`.
- [x] Change `config set vault` to canonicalize and require an existing file.
- [x] Move vault/output validation helpers to `vault.rs`.
- [x] Move clipboard helper to `clipboard.rs` without changing blocking clear behavior.
- [x] Move password generator to `generator.rs`.
- [x] Move prompt helpers to `prompts.rs`.
- [x] Move output formatting helpers to `printing.rs`.
- [x] Keep `main.rs` as the command dispatch layer.
- [x] Update README with:
  - [x] canonical absolute vault path behavior
  - [x] blocking clipboard clear behavior
  - [x] clipboard environment/CI caveat
- [x] Update journal with settlement completion notes.

## Verification

Automated checks:

```bash
cargo fmt --all
cargo test --workspace
cargo run -q -p anahtar-cli -- --help
cargo run -q -p anahtar-cli -- config show
```

Manual/smoke checks:

```bash
cargo run -q -p anahtar-cli -- config set vault test-vaults/generated/phase3-base.kdbx
cargo run -q -p anahtar-cli -- config get vault
```

Expected:

- Stored vault path is absolute.
- Missing vault path fails with a clear error.
- Directory vault path fails with a clear error.
- `anahtar search github` works using the default vault.
- Running from a different working directory still resolves the configured vault.

Clipboard behavior remains as previously verified:

```bash
cargo run -q -p anahtar-cli -- generate --copy --clear-after 2
cargo run -q -p anahtar-cli -- copy-password "Github Test" --clear-after 2
```

Expected:

- Command waits for the timeout.
- Clipboard is cleared only if it still contains Anahtar's copied value.

## Non-goals

- No GUI work.
- No in-place vault writes.
- No background clipboard daemon/process.
- No key-file support changes.
- No secret storage in config.
- No changes to KDBX read/write behavior.
