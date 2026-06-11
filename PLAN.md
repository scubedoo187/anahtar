# Anahtar development plan

## Current status

Anahtar is a Rust-first KeePass/KDBX-compatible password manager project.

Completed phases:

- Phase 1 — Read-only CLI MVP
- Phase 2 — KDBX4.1 upgrade/save-as workflow
- Phase 3 — Minimal save-as write commands
- Phase 4 — Daily-use CLI polish
- Phase 4 settlement before productization
- Phase 5 — CLI Password Manager Productization
- Phase 5.5 — UI Readiness Cleanup

Next phase:

- Phase 6 — GUI Alpha

## Current architecture

```text
crates/
  anahtar-core/   # KDBX domain operations and safe public DTOs
  anahtar-app/    # stateless GUI/CLI application service facade
  anahtar-cli/    # CLI product surface, prompts, config, printing, clipboard
```

Recommended GUI call path:

```text
GUI/Tauri command layer
  -> anahtar-app::AnahtarService
    -> anahtar-core
```

## Canonical planning documents

- Roadmap: `docs/anahtar-roadmap.md`
- Final goals: `docs/anahtar-final-goals.md`
- Threat model: `docs/threat-model.md`
- GUI API contract: `docs/gui-api-contract.md`
- Phase 5 plan: `docs/phase-plans/phase-5-cli-password-manager-productization.md`
- Phase 5.5 plan: `docs/phase-plans/phase-5-5-ui-readiness-cleanup.md`
- Phase 6 plan: `docs/phase-plans/phase-6-gui-alpha.md`

Older phase plan files remain as historical implementation records.

## Current product policy

- Use `keepass = 0.13.8`; do not implement custom crypto.
- Write output is standardized on KDBX 4.1.
- CLI master passwords are prompted through TTY, never plain CLI args.
- Key-file paths may be stored as user-local config; key-file contents and master passwords must not be stored.
- Default write behavior for mutable commands is safe in-place update when `--output` is omitted.
- Explicit `--output` remains available for save-as workflows.
- In-place writes use backup creation, temp save, temp reopen verification, target replacement, and final reopen verification.
- Backups default to sibling `anahtar-backups/`, optionally overridden by config.
- Clipboard handling is platform/surface-specific and must clear only if clipboard still contains Anahtar's copied value.
- Public repo hygiene: never commit real `.kdbx`, `.kdb`, `.key`, or `.keyx` files.

## Phase 6 readiness summary

Phase 6 may start because:

- `anahtar-app::AnahtarService` exists as the GUI-facing service layer.
- `docs/gui-api-contract.md` defines the GUI/backend boundary.
- CLI command flows remain available as a reference implementation.
- CI covers fmt, clippy, tests, examples, generated synthetic vaults, install smoke, version smoke, and completion smoke.
- README describes CLI productized status and GUI alpha as next.

Known accepted technical debt before Phase 6:

- `anahtar-core` exposes module-specific public facades, but much of the physical implementation remains in `internal.rs` to preserve behavior during Phase 5.5. This is acceptable for GUI alpha because GUI should depend on `anahtar-app`, not internal core modules. A deeper internal physical split can happen later if core churn grows.

## Standard verification

Before major changes:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --examples
```

Product smoke checks:

```bash
tmp="$(mktemp -d)"
cargo install --path crates/anahtar-cli --root "$tmp" --quiet
"$tmp/bin/anahtar" --version
"$tmp/bin/anahtar" completions bash >/tmp/anahtar-completions.bash
rm -rf "$tmp"
```

Repo hygiene check:

```bash
rg -n "<absolute-user-path-patterns>" README.md docs PLAN.md crates .github -g '!target' || true
git ls-files assets journals '*.kdbx' '*.kdb' '*.key' '*.keyx' .DS_Store
```

## Non-goals for Phase 6 alpha

- Browser extension
- Mobile app
- Cloud sync engine
- Shared vault collaboration
- Background unlock daemon
- Biometric unlock
- Passkeys
- Cross-platform installer polish beyond local macOS packaging
