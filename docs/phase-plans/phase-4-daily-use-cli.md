# Phase 4 plan — Daily-use CLI polish

Phase 4 turns the working read/write CLI into a daily-use tool. Since there are no public users yet, this phase also normalizes the CLI shape across read, upgrade, and write commands.

## Goal

Add default vault configuration, consistent `--vault` handling, clipboard copy, password generation, and TOTP retrieval.

By the end of Phase 4, common daily workflows should look like:

```bash
anahtar config set vault /path/to/vault.kdbx
anahtar search github
anahtar copy-password github
anahtar generate --copy
anahtar totp github
```

## CLI shape decision

All vault-targeting commands should use:

```bash
--vault <path>
```

If `--vault` is omitted, Anahtar resolves the vault from config.

Resolution order:

1. command-level `--vault <path>`
2. configured default vault
3. error with instruction:

```text
No vault provided and no default vault configured.
Run: anahtar config set vault /path/to/vault.kdbx
```

## New command shape

### Read commands

```bash
anahtar inspect
anahtar inspect --vault <path>

anahtar list
anahtar list --vault <path>

anahtar search <query>
anahtar search --vault <path> <query>

anahtar show <selector>
anahtar show --vault <path> <selector>
anahtar show <selector> --reveal-password
```

### Upgrade

```bash
anahtar upgrade --output <output>
anahtar upgrade --vault <input> --output <output>
```

### Write commands

```bash
anahtar add \
  --output <output> \
  --group "General/Web" \
  --title "Github" \
  --password-prompt

anahtar add \
  --vault <input> \
  --output <output> \
  --group "General/Web" \
  --title "Github" \
  --password-prompt

anahtar edit <selector> \
  --output <output> \
  --username "new-user"

anahtar edit --vault <input> <selector> \
  --output <output> \
  --username "new-user"

anahtar delete <entry-id> \
  --output <output>

anahtar delete --vault <input> <entry-id> \
  --output <output>
```

### Config commands

```bash
anahtar config show
anahtar config get vault
anahtar config set vault <path>
anahtar config set generator-length 32
anahtar config set clipboard-clear-after 30
```

### Copy commands

```bash
anahtar copy-password <selector>
anahtar copy-username <selector>
anahtar copy-url <selector>
```

Optional vault override:

```bash
anahtar copy-password --vault <path> <selector>
```

### Password generator

```bash
anahtar generate
anahtar generate --length 24
anahtar generate --copy
```

### TOTP

```bash
anahtar totp <selector>
anahtar totp --copy <selector>
```

## Config schema

Config format: TOML.

Preferred location should use the platform config directory via a crate such as `directories`.

Example:

```toml
vault = "/path/to/default.kdbx"
generator_length = 32
clipboard_clear_after_seconds = 30
```

Defaults:

- `generator_length = 32`
- `clipboard_clear_after_seconds = 30`

## Implementation checklist

### Config foundation

- [x] Add config dependencies, likely `toml` and `directories`.
- [x] Add `AnahtarConfig` struct.
- [x] Implement config path resolution.
- [x] Implement config load with defaults.
- [x] Implement config save.
- [x] Validate `generator_length` is reasonable.
- [x] Validate `clipboard_clear_after_seconds` is reasonable.

### Config CLI

- [x] Add `anahtar config show`.
- [x] Add `anahtar config get vault`.
- [x] Add `anahtar config set vault <path>`.
- [x] Add `anahtar config set generator-length <n>`.
- [x] Add `anahtar config set clipboard-clear-after <seconds>`.

### Vault resolution and CLI refactor

- [x] Add vault resolution helper:
  - [x] command `--vault` first,
  - [x] config default second,
  - [x] helpful error third.
- [x] Refactor `inspect` to use optional `--vault`.
- [x] Refactor `list` to use optional `--vault`.
- [x] Refactor `search` to use optional `--vault` and positional query.
- [x] Refactor `show` to use optional `--vault` and positional selector.
- [x] Refactor `upgrade` to use optional `--vault` plus required `--output`.
- [x] Refactor `add` to use optional `--vault` plus required `--output`.
- [x] Refactor `edit` to use optional `--vault` plus required `--output`.
- [x] Refactor `delete` to use optional `--vault` plus required `--output`.
- [x] Update README examples to the new CLI shape.

### Clipboard copy

- [x] Add clipboard dependency, recommended `arboard`.
- [x] Implement shared clipboard helper.
- [x] Implement clear-after behavior.
- [x] Clear clipboard only if it still contains the value Anahtar copied.
- [x] Add `copy-password <selector>` without printing secret.
- [x] Add `copy-username <selector>`.
- [x] Add `copy-url <selector>`.
- [x] Support `--clear-after <seconds>` override.
- [x] Use config default clear timeout when override is omitted.

### Password generator

- [x] Add secure random password generator.
- [x] Default length is 32.
- [x] `--length` overrides default length.
- [x] Config `generator_length` sets default length.
- [x] Include lowercase, uppercase, digits, and symbols by default.
- [x] Implement `anahtar generate`.
- [x] Implement `anahtar generate --copy`.
- [x] Implement `add --generate-password`.
- [x] Ensure generated password is not included in JSON write reports.

### TOTP

- [x] Inspect supported OTP field patterns from `keepass` crate / KeePassXC / Strongbox fixtures.
- [x] Implement TOTP extraction without printing OTP URI.
- [x] Implement `anahtar totp <selector>`.
- [x] Show code and remaining validity time.
- [x] Implement `anahtar totp --copy <selector>`.
- [x] Reuse clipboard clear behavior for copied TOTP code.

### Documentation and journal

- [x] Update README for config/default vault/copy/generate/totp.
- [x] Update phase plan as tasks complete.
- [x] Add journal entries for Phase 4 progress and completion.

## Exit criteria

- [x] Config TOML is read and written.
- [x] Default vault can be set, read, and shown.
- [x] All vault commands support `--vault`.
- [x] All vault commands use default vault when `--vault` is omitted.
- [x] Read commands work with new CLI shape.
- [x] `upgrade`, `add`, `edit`, and `delete` work with new CLI shape.
- [x] `copy-password` copies password without printing it.
- [x] `copy-username` works.
- [x] `copy-url` works.
- [x] Clipboard clears after configured/default timeout.
- [x] `generate` default length is 32.
- [x] `generate --length` works.
- [x] `generate --copy` works.
- [x] `add --generate-password` works.
- [x] `totp <selector>` displays code without exposing OTP URI.
- [x] `totp --copy` works.
- [x] `cargo fmt --all` passes.
- [x] `cargo test --workspace` passes.

## Decisions accepted for Phase 4

| Topic | Decision |
|---|---|
| Config format | TOML |
| Default vault | supported |
| Vault CLI shape | `--vault <path>` optional; config fallback |
| CLI consistency | refactor read, upgrade, and write commands together |
| Clipboard crate | `arboard` preferred |
| Clipboard clear default | 30 seconds |
| Generator default length | 32 |
| Generator length | configurable via config and `--length` |
| TOTP | included in Phase 4 |
| Interactive picker | deferred |

## Risk notes

- Clipboard clear requires the CLI process to stay alive until timeout expires.
- TOTP field compatibility may require fixture-driven investigation.
- CLI shape changes are breaking, but acceptable now because there are no public users yet.
- Write commands must preserve the Phase 3 safety model after refactor.
