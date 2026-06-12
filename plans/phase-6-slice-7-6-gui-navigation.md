# Plan: Phase 6 Slice 7.6 GUI Navigation/Layout Cleanup

## Context

Anahtar GUI alpha currently works functionally, but the UI is a long one-page surface: backend status, unlock, browse/detail, groups, audit, and write actions are all stacked vertically. This makes local testing and Phase 8 packaging screenshots feel less productized, and keeps risky write/delete actions visible even during normal browse/copy use.

Benchmarking suggests Anahtar should follow a Strongbox/KeePassXC-style KDBX/database layout more than a full 1Password account-centric model:

- **Strongbox/KeePassXC pattern:** database/group-centric sidebar, entry list, detail/preview pane, tools for audit/reports/write actions.
- **1Password pattern:** polished three-pane model with sidebar navigation, item list, and item detail/actions.

Recommended outcome: a lightweight desktop three-region layout using the existing vanilla TypeScript frontend and `anahtar-app` backend boundary.

## Approach

Keep the current functionality and backend API unchanged, but reorganize the frontend into a product-like shell:

```text
Top session bar: unlock inputs, status, lock/inspect controls
Main app grid:
  Left sidebar: Browse, Groups, Audit, Write, Status
  Center pane: entry search/list or tool-specific content
  Right pane: entry detail/copy/reveal or contextual output
```

Initial active view should be **Browse**. Unlock remains globally visible at the top because it gates all other actions. The sidebar controls which major panel is visible; no active view is persisted to storage.

Narrowed view model:

1. **Browse**
   - search query/actions
   - entry list in center pane
   - selected entry detail + copy/reveal actions in right pane

2. **Groups**
   - load groups action
   - group list in center/right content area
   - no move/group mutation in this slice

3. **Audit**
   - run audit action
   - audit findings display

4. **Write**
   - backup directory
   - add/edit/delete forms
   - write report
   - delete confirmation unchanged

5. **Status**
   - backend status
   - inspect vault
   - command output/debug-style messages

Use accessible buttons for sidebar navigation with `aria-selected`; hide inactive panels with `hidden`. Keep dynamic vault/entry/secret data rendered via `textContent`/DOM node creation only. The existing static `shell.ts` template may remain static-only, with no user/secret interpolation.

## Files to modify

- `apps/anahtar-gui/src/shell.ts`
  - Replace stacked cards with top session bar + app layout/sidebar + view panels.
- `apps/anahtar-gui/src/state.ts`
  - Add `ActiveView` and `activeView` to `AppState`.
- `apps/anahtar-gui/src/render.ts`
  - Add `renderNavigationState`/panel visibility logic.
  - Keep existing entry/detail/group/audit/write renderers.
- `apps/anahtar-gui/src/main.ts`
  - Bind sidebar view buttons.
  - Switch views after common actions where useful: unlock -> Browse, write -> Browse or remain Write with report visible.
- `apps/anahtar-gui/src/dom.ts`
  - Add small helpers if needed for view buttons/panels.
- `apps/anahtar-gui/src/styles.css`
  - Add desktop layout styles: top bar, sidebar, panes, active nav button, responsive collapse.
- `goals/phase-6-gui-alpha/progress.jsonl`
  - Append implementation and verification evidence after completion.

## Reuse

- Existing frontend split from Slice 7.5:
  - `apps/anahtar-gui/src/shell.ts` for static shell rendering.
  - `apps/anahtar-gui/src/state.ts` for in-memory app/session state.
  - `apps/anahtar-gui/src/render.ts` for DOM rendering via `textContent` and nodes.
  - `apps/anahtar-gui/src/clipboard.ts` for clear-if-owned clipboard policy.
  - `apps/anahtar-gui/src/errors.ts` for structured error display.
- Existing Tauri/API boundary:
  - `apps/anahtar-gui/src/api.ts`
  - `apps/anahtar-gui/src-tauri/src/lib.rs`
- Existing backend service boundary remains unchanged:
  - `anahtar-app::AnahtarService`
  - `WriteMode::InPlace { backup_dir }`

## Steps

- [ ] Add `ActiveView = "browse" | "groups" | "audit" | "write" | "status"` to frontend state.
- [ ] Redesign `shell.ts` into top session bar + left sidebar + center/right panels.
- [ ] Add sidebar buttons and panel containers with stable IDs for existing handlers.
- [ ] Move current Browse controls/list/detail into Browse panel without changing entry selection/copy/reveal behavior.
- [ ] Move groups/audit/write/status cards into their own panels while preserving element IDs used by existing code.
- [ ] Implement `renderNavigationState(state)` to update active button and hide inactive panels.
- [ ] Bind sidebar buttons in `main.ts` and re-render navigation after state/view changes.
- [ ] Update CSS for a macOS-like desktop layout, including responsive fallback for narrow windows.
- [ ] Run safety scans for storage/logging/unsafe dynamic HTML.
- [ ] Append Slice 7.6 evidence to progress JSONL.
- [ ] Commit and push after verification passes.

## Verification

Run automated checks:

```bash
cd apps/anahtar-gui
npm run typecheck
npm run build:frontend
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm audit --audit-level=high
cd ../..
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --examples
```

Run safety scans:

```bash
rg -n "localStorage|sessionStorage|console\." apps/anahtar-gui/src apps/anahtar-gui/src-tauri/src -g '!target' -g '!node_modules' -g '!dist' || true
rg -n "innerHTML|insertAdjacentHTML" apps/anahtar-gui/src apps/anahtar-gui/src-tauri/src -g '!target' -g '!node_modules' -g '!dist' || true
```

Manual local GUI check:

```bash
cd apps/anahtar-gui
npm run dev
```

Use generated vault:

```text
../../../test-vaults/generated/phase3-base.kdbx
testpass
```

Manual acceptance checks:

- [ ] App opens with top unlock/session bar and sidebar visible.
- [ ] Unlock succeeds and Browse shows entry list + detail pane.
- [ ] Search/list/reset/select detail still work.
- [ ] Reveal is explicit and hidden by default.
- [ ] Copy username/password/URL/TOTP still works with owned clipboard clear policy.
- [ ] Groups view loads groups.
- [ ] Audit view runs audit without secret leakage.
- [ ] Write view add/edit/delete still uses confirmation/report/backup path.
- [ ] Status view shows backend status, inspect output, and command messages.
- [ ] No secret data is stored in browser storage or logged.
