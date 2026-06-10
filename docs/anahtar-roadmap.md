# Anahtar entire roadmap

This document is the fixed high-level roadmap record for Anahtar. It captures the full path from current spike state to a personal Strongbox-replacement workflow.

## Roadmap baseline

### Phase 1 — Workspace and read-only CLI MVP

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

Goal: formalize non-destructive KDBX4.0/3.x → KDBX4.1 conversion.

Deliverables:

- `upgrade` command
- explicit output path
- temp file save
- reopen verification
- no overwrite unless forced
- Strongbox manual verification flow

### Phase 3 — Minimal write commands

Goal: make controlled modifications without risking the active vault.

Deliverables:

- `add`
- `edit`
- `delete`
- save-as first
- backup + atomic write policy before any in-place operation

### Phase 4 — Daily-use CLI polish

Goal: make CLI useful for real daily retrieval tasks.

Deliverables:

- password generator
- clipboard copy with timed clear
- default vault config
- TOTP display if compatible
- shell completion

### Phase 5 — GUI alpha

Goal: build a desktop GUI using the same core.

Deliverables:

- Tauri-based GUI candidate
- open/unlock/search/list/detail/copy
- write UX only after CLI write model is stable
- macOS first, Windows/Linux later

## Roadmap rule

This roadmap is intentionally broad and should not be rewritten every time implementation details change. Detailed phase plans live separately and are updated as each phase begins/completes.
