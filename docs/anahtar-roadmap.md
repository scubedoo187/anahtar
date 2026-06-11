# Anahtar entire roadmap

This document is the high-level roadmap record for Anahtar. It captures the path from the current CLI MVP toward a personal Strongbox/paid-password-manager replacement workflow.

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
- module split/settlement before next phase

### Phase 5 — CLI Password Manager Productization

Status: next

Goal: make Anahtar a credible password-manager CLI product before starting GUI work.

Rationale: GUI work should mostly assemble stable core capabilities. If password-manager functionality is still missing, the GUI phase will repeatedly force core/CLI rewrites.

Deliverables:

- credential material abstraction, including key-file support if supported by backend
- safe in-place update workflow as the default write path, with backup, temp save, atomic replace, and reopen verification
- stable explicit selector model
- group organization including list/add/rename/delete and entry move support
- audit/check commands that never print secrets
- CI/install/release polish including shell completion
- product threat model and safety documentation

Canonical plan:

- `docs/phase-plans/phase-5-cli-password-manager-productization.md`

### Phase 6 — GUI alpha

Status: deferred until Phase 5 completion

Goal: build a desktop GUI using the same core capabilities stabilized by the CLI productization phase.

Deliverables:

- Tauri-based GUI candidate
- open/configure vault
- unlock/search/list/detail/copy
- add/edit/delete using the CLI-proven safe write model
- group/audit UI if Phase 5 exposes stable APIs
- macOS local packaging first, Windows/Linux later

Canonical plan:

- `docs/phase-plans/phase-6-gui-alpha.md`

## Roadmap rule

This roadmap is intentionally broad and should not be rewritten every time implementation details change. Detailed phase plans live separately and are updated as each phase begins/completes.

Major strategy changes should be reflected here only when they affect phase ordering or product definition.
