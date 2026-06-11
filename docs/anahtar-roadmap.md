# Anahtar roadmap

This document is the high-level roadmap record for Anahtar. It captures the path from the CLI foundation toward a personal Strongbox/paid-password-manager replacement workflow.

## Roadmap baseline

### Phase 1 — Workspace and read-only CLI MVP

Status: complete

Goal: prove safe, non-mutating access to KDBX files.

Deliverables:

- Rust workspace
- `anahtar-core`
- `anahtar-cli`
- `inspect`
- `list`
- `search`
- `show`
- JSON output option for future GUI integration

### Phase 2 — Upgrade/save-as CLI

Status: complete

Goal: formalize non-destructive KDBX4.0/3.x → KDBX4.1 conversion.

Deliverables:

- `upgrade` command
- explicit output path
- temp file save
- reopen verification
- no overwrite unless forced
- Strongbox manual verification flow

### Phase 3 — Minimal write commands

Status: complete

Goal: make controlled modifications without risking the active vault.

Deliverables:

- `add`
- `edit`
- `delete`
- save-as first
- output protection
- temp save + reopen verification
- no in-place writes yet

### Phase 4 — Daily-use CLI polish

Status: complete

Goal: make CLI useful for real daily retrieval tasks.

Deliverables:

- TOML config
- default vault
- consistent `--vault` override
- password generator
- clipboard copy with timed clear
- TOTP display/copy
- `add --generate-password`
- module split/settlement before productization

### Phase 5 — CLI Password Manager Productization

Status: complete

Goal: make Anahtar a credible password-manager CLI product before starting GUI work.

Deliverables:

- credential material abstraction, including key-file support
- safe in-place update workflow as the default write path, with backup, temp save, replacement, and reopen verification
- stable explicit selector model
- group organization including list/add/rename/delete and entry move support
- audit/check commands that never print secrets
- CI/install/release polish including shell completion
- product threat model and safety documentation

Canonical plan:

- `docs/phase-plans/phase-5-cli-password-manager-productization.md`

### Phase 5.5 — UI Readiness Cleanup

Status: complete

Goal: reduce architectural friction before GUI work begins.

Deliverables:

- `anahtar-core` public module facade
- thinner CLI dispatch structure
- `anahtar-app` stateless service facade for GUI/CLI reuse
- GUI API contract documentation
- stronger CI/product smoke checks
- README/PLAN/product-readiness updates

Canonical plan:

- `docs/phase-plans/phase-5-5-ui-readiness-cleanup.md`

### Phase 6 — GUI Alpha

Status: accepted / ready to implement

Goal: build a macOS-first desktop GUI using the same capabilities stabilized by the CLI and exposed through `anahtar-app`.

Deliverables:

- Tauri + TypeScript GUI under `apps/anahtar-gui`
- open/configure vault
- optional key-file path
- unlock/list/search/detail workflows
- explicit reveal/copy flows for sensitive values
- GUI-owned clipboard clear behavior
- add/edit/delete using the CLI-proven safe write model
- group/audit UI if alpha complexity remains manageable
- macOS local packaging first, Windows/Linux later

Canonical plan:

- `docs/phase-plans/phase-6-gui-alpha.md`

## Strategic rules

- GUI code should call `anahtar-app::AnahtarService`, not reimplement KDBX traversal or write safety.
- `anahtar-core` remains the KDBX domain layer.
- `anahtar-cli` remains a supported product surface, not a throwaway prototype.
- Major strategy changes should be reflected here only when they affect phase ordering or product definition.
- Detailed implementation checklists live in per-phase plans.
