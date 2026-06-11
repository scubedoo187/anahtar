# Phase 5.5 — UI Readiness Cleanup

Status: complete

## Goal

Prepare Anahtar for Phase 6 GUI work by reducing architectural friction before UI code starts. Phase 5 made the CLI product-grade; Phase 5.5 makes the codebase easier for a GUI to reuse without repeatedly reshaping core/CLI internals.

## Why this phase exists

Anahtar is functionally ready for GUI prototyping, but several structural issues could become expensive once UI work begins:

- `anahtar-core/src/lib.rs` is too large and mixes types, read flows, write flows, selectors, groups, audit, TOTP, and tests.
- `anahtar-cli/src/main.rs` still owns too much orchestration.
- There is no explicit app/service layer for GUI and CLI to share.
- CI is good but not yet product-readiness CI.
- GUI-facing API contracts are implicit rather than documented.

## Non-goals

- No GUI/Tauri implementation yet.
- No new user-facing password-manager features unless needed to preserve behavior during refactor.
- No KDBX backend change.
- No browser extension/mobile/sync work.
- No history rewrite or repo hygiene work beyond normal docs/code cleanup.

## Workstream A — Split `anahtar-core` modules

Goal: turn `anahtar-core` into a reusable library structure before GUI depends on it.

Target structure:

```text
crates/anahtar-core/src/
  lib.rs          # public module exports only
  types.rs        # public DTOs and request/report structs
  errors.rs       # AnahtarError and Result
  credentials.rs  # VaultCredentials and DatabaseKey construction
  inspect.rs      # inspect_header, KdbxVersion helpers if not in types
  entries.rs      # list/search/show/add/edit/delete entry flows
  selectors.rs    # EntrySelector and resolution helpers
  groups.rs       # group list/add/rename/delete/move
  audit.rs        # AuditReport/AuditFinding and audit logic
  write.rs        # save-as and safe in-place write helpers
  totp.rs         # TOTP code extraction
  util.rs         # small path/count/traversal helpers if needed
```

Tasks:

- [x] Move public data types to `types.rs`.
- [x] Move `AnahtarError` and `Result` to `errors.rs`.
- [x] Move `VaultCredentials` to `credentials.rs`.
- [x] Move inspect/version helpers to `inspect.rs`.
- [x] Move entry read/write operations to `entries.rs`.
- [x] Move selector matching/resolution to `selectors.rs`.
- [x] Move group operations to `groups.rs`.
- [x] Move audit logic to `audit.rs`.
- [x] Move save-as and in-place write helpers to `write.rs`.
- [x] Move TOTP logic to `totp.rs`.
- [x] Re-export stable public API from `lib.rs`.
- [x] Keep all existing public function names working where practical.
- [x] Keep tests passing after each module extraction chunk.

Acceptance criteria:

- [x] `lib.rs` is mostly module declarations and re-exports.
- [x] No behavior changes.
- [x] Existing CLI compiles without large import churn outside module paths/re-exports.
- [x] `cargo fmt --all`, `cargo test --workspace`, and clippy pass.

## Workstream B — Split CLI dispatch/orchestration

Goal: keep CLI as a thin product surface rather than a second application layer.

Target structure:

```text
crates/anahtar-cli/src/
  main.rs          # parse and dispatch only
  cli.rs           # clap args/enums
  commands.rs      # command handlers
  write_flow.rs    # CLI-specific in-place/save-as selection if still needed
  selectors.rs     # CLI EntrySelectorArgs -> core EntrySelector
  config.rs
  clipboard.rs
  generator.rs
  printing.rs
  prompts.rs
  vault.rs
```

Tasks:

- [x] Move command match arms from `main.rs` to `commands.rs`.
- [x] Move CLI selector conversion to `selectors.rs`.
- [x] Move CLI write helper wrappers to `write_flow.rs`.
- [x] Keep `main.rs` to `Cli::parse()` + `commands::run(...)`.
- [x] Avoid changing CLI syntax.

Acceptance criteria:

- [x] `main.rs` is small and stable.
- [x] Command handlers are easier to inspect and test later.
- [x] No CLI behavior changes.
- [x] Existing help output remains equivalent.

## Workstream C — Add app/service layer for GUI reuse

Goal: define a GUI-friendly high-level API that both CLI and future GUI can call.

Recommended crate:

```text
crates/anahtar-app/
```

Alternative: `crates/anahtar-core/src/service.rs`. Preferred: separate `anahtar-app` crate to keep KDBX domain logic separated from application orchestration.

Responsibilities:

- Hold no long-lived secrets by default.
- Accept explicit paths and credentials from caller.
- Expose high-level operations that compose core functions.
- Return serializable DTOs suitable for CLI JSON and GUI IPC.

Candidate API:

```rust
pub struct AnahtarService;

impl AnahtarService {
    pub fn inspect(path: &Path) -> Result<VaultInfo>;
    pub fn open_list(path: &Path, credentials: &VaultCredentials) -> Result<Vec<EntrySummary>>;
    pub fn search(path: &Path, credentials: &VaultCredentials, query: &str) -> Result<Vec<EntrySummary>>;
    pub fn show(path: &Path, credentials: &VaultCredentials, selector: EntrySelector, reveal: bool) -> Result<EntryDetail>;
    pub fn copy_value(...); // maybe CLI/GUI-specific clipboard stays outside
    pub fn add_entry(..., WriteMode) -> Result<WriteReport>;
    pub fn edit_entry(..., WriteMode) -> Result<WriteReport>;
    pub fn delete_entry(..., WriteMode) -> Result<WriteReport>;
    pub fn groups(...);
    pub fn audit(...);
}
```

Supporting types:

```rust
pub enum WriteMode {
    SaveAs { output_path: PathBuf, force: bool },
    InPlace { target_path: PathBuf, backup_dir: Option<PathBuf> },
    DryRun,
}
```

Tasks:

- [x] Create `crates/anahtar-app` crate.
- [x] Add it to workspace members.
- [x] Add dependency on `anahtar-core`.
- [x] Define `AnahtarService` or equivalent stateless service facade.
- [x] Define `WriteMode`.
- [x] Move reusable orchestration out of CLI into app crate where appropriate.
- [x] Refactor CLI to call `anahtar-app` for operations where it reduces duplication.
- [x] Keep clipboard and TTY prompts in CLI, not app/core.
- [x] Add app-level tests for representative read/write flows.

Acceptance criteria:

- [x] Future GUI can depend on `anahtar-app` instead of directly assembling low-level core calls.
- [x] CLI still works and is thinner.
- [x] No secrets are stored in long-lived service state.

## Workstream D — Strengthen CI/product checks

Goal: make CI catch issues that matter before UI work begins.

Current CI:

- fmt
- clippy
- tests

Add:

- synthetic vault generation before tests,
- examples check,
- CLI install smoke,
- CLI help/version/completion smoke.

Tasks:

- [x] Update `.github/workflows/ci.yml` to run synthetic vault generation.
- [x] Add `cargo check --workspace --examples`.
- [x] Add `cargo install --path crates/anahtar-cli --root <temp>` smoke.
- [x] Add `<temp>/bin/anahtar --version` smoke.
- [x] Add completion generation smoke, e.g. `anahtar completions bash`.
- [x] Consider matrix for macOS later; keep Ubuntu first unless dependency issues appear.

Acceptance criteria:

- [x] CI verifies the same checks used locally.
- [x] Generated-vault-dependent tests run in CI instead of silently skipping.
- [x] Install/completion breakages are caught before releases.

## Workstream E — GUI API contract documentation

Goal: make Phase 6 implementation targeted and avoid re-litigating API shape in UI code.

New doc:

```text
docs/gui-api-contract.md
```

Contents:

- Which crate GUI should depend on.
- Which operations are available.
- DTOs returned by read/list/show/audit/group commands.
- Write flow contract and backup behavior.
- Error handling strategy.
- Credential handling rules.
- Clipboard boundary: GUI owns clipboard interaction or calls a platform adapter, not core.
- Long-running command expectations.

Tasks:

- [x] Add `docs/gui-api-contract.md`.
- [x] Document read workflows.
- [x] Document write workflows.
- [x] Document credential/key-file handling.
- [x] Document error display rules.
- [x] Document what GUI must not do, e.g. store master password in config.

Acceptance criteria:

- [x] Phase 6 has a clear backend contract.
- [x] GUI can be planned around stable DTOs and service calls.

## Workstream F — Product status/readiness docs

Goal: communicate clearly what is product-ready and what remains alpha.

Tasks:

- [x] Update README status section: CLI productized, GUI upcoming.
- [x] Add note that macOS is first GUI target.
- [x] Document current limitations:
  - no browser extension,
  - no mobile,
  - no sync engine,
  - no background unlock daemon,
  - Windows replacement semantics need dedicated validation before Windows-first usage.
- [x] Add release/build notes if not already sufficient.

Acceptance criteria:

- [x] A new reader understands the project state without reading all phase plans.
- [x] Phase 6 has a clean starting point.

## Recommended implementation order

This phase should be implemented as small, reviewable commits. Each slice should preserve behavior and keep the full workspace green before moving on.

### Commit 1 — Core module skeleton and type/error extraction

- Move public DTO/request/report types to `types.rs`.
- Move `AnahtarError` and `Result` to `errors.rs`.
- Re-export from `lib.rs` to preserve current public API.
- Run fmt/test/clippy.

### Commit 2 — Core read-path extraction

- Move credentials/key construction to `credentials.rs`.
- Move inspect/version logic to `inspect.rs`.
- Move entry list/search/show selector-independent read logic to `entries.rs`.
- Move TOTP read logic to `totp.rs`.
- Keep function names and behavior stable through re-exports.
- Run fmt/test/clippy.

### Commit 3 — Core selector, group, audit extraction

- Move selector matching/resolution to `selectors.rs`.
- Move group operations to `groups.rs`.
- Move audit logic to `audit.rs`.
- Run fmt/test/clippy.

### Commit 4 — Core write-path extraction

- Move save-as and safe in-place write helpers to `write.rs`.
- Keep backup/temp/final verification behavior unchanged.
- Keep tests close to the modules where practical, or leave integration-style tests in `lib.rs` until stable.
- Run fmt/test/clippy and examples check.

### Commit 5 — CLI dispatch split

- Move command match arms from `main.rs` into `commands.rs`.
- Move CLI selector conversion into `selectors.rs`.
- Move CLI save-as/in-place write selection helpers into `write_flow.rs`.
- Keep CLI syntax, help, and output stable.
- Smoke-test key help surfaces and completion generation.

### Commit 6 — Add `anahtar-app` service layer

- Add `crates/anahtar-app` to the workspace.
- Define a stateless `AnahtarService` facade and `WriteMode`.
- Move reusable operation orchestration out of CLI where it cleanly fits.
- Keep TTY prompting, config resolution, and clipboard behavior outside app/core.
- Add representative app-level tests.

### Commit 7 — CI hardening

- Add synthetic vault generation to CI.
- Add `cargo check --workspace --examples`.
- Add temporary install smoke.
- Add `anahtar --version` and completion generation smoke.

### Commit 8 — GUI contract and product-readiness docs

- Add `docs/gui-api-contract.md`.
- Update README with CLI productized / GUI upcoming status.
- Document macOS-first GUI target and current product limitations.
- Update roadmap/final goals/plan as needed.

## Exit criteria

- [x] `anahtar-core` is modular and GUI-friendly.
- [x] CLI command handling is thin and not the only orchestration layer.
- [x] `anahtar-app` or equivalent service layer exists for GUI reuse.
- [x] CI includes generated test vaults, examples, install smoke, and completion smoke.
- [x] GUI API contract is documented.
- [x] README clearly describes CLI productized status and GUI alpha as next.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo test --workspace` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo check --workspace --examples` passes.

## Notes / decisions

- This phase should be completed before writing Tauri UI code.
- Keep this phase mostly refactor/documentation/CI. Avoid major new user-facing features.
- If `anahtar-app` becomes too thin, keep it anyway as the GUI contract boundary; it can grow naturally during Phase 6.
