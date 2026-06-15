# Phase 7 — Native macOS Transition Plan

Status: proposed before Phase 8 packaging

## Decision

Anahtar will move from the current Tauri GUI alpha toward a native macOS app using SwiftUI/AppKit, while keeping the existing Rust KDBX backend as the source of truth.

This is a deliberate direction change for GUI integrity and native macOS feel. The current Tauri app remains useful as a functional alpha/reference implementation until the native app reaches feature parity.

## Goals

- Make Anahtar feel like a first-class Mac app before packaging.
- Preserve the Rust-first KDBX/security architecture.
- Avoid duplicating KDBX traversal, write, TOTP, audit, and backup logic in Swift.
- Keep master password and secrets in memory only.
- Keep recent vault config limited to vault path/key-file path.
- Retain safe in-place write semantics with backup/reopen verification.

## Non-goals

- No browser extension, mobile app, cloud sync, passkeys, or biometric unlock in this phase.
- No custom crypto.
- No real-vault test data committed.
- No password/key-file contents persisted in macOS preferences, Keychain, logs, or crash reports.
- No Swift reimplementation of KDBX logic.

## Target architecture

```text
apps/anahtar-macos/          # native macOS app
  Anahtar.xcodeproj or Package.swift
  Sources/AnahtarMac/        # SwiftUI + small AppKit bridges

crates/anahtar-app/          # existing high-level Rust service, unchanged source of truth
crates/anahtar-ffi/          # new thin C ABI / JSON DTO bridge around anahtar-app
crates/anahtar-core/         # existing KDBX implementation
```

### Swift side

- SwiftUI for app lifecycle, screens, state, dialogs, and standard controls.
- AppKit where it improves native behavior:
  - `NSSplitViewController`/split-view equivalent for 3-pane layout if SwiftUI split view is insufficient.
  - `NSOpenPanel` for vault/key-file selection.
  - native menu commands and keyboard shortcuts.
  - standard alert sheets for destructive confirmations.
- No WebView for primary UI.

### Rust side

- Keep `anahtar-app::AnahtarService` as the application boundary.
- Add `crates/anahtar-ffi` as a small native bridge.
- FFI functions return structured JSON DTOs matching the existing GUI API where possible.
- FFI accepts credentials in memory only and never through process arguments.
- Rust still owns:
  - unlock/list/search/show
  - reveal/copy source values
  - add/edit/delete
  - group add/rename/delete
  - move entry
  - audit
  - TOTP
  - safe in-place write + backup verification

## Bridge strategy

Preferred initial bridge: C ABI static library with JSON request/response strings.

Why:

- Simpler than a full UniFFI setup for the first native alpha.
- Avoids passing secrets through CLI args or shell subprocesses.
- Keeps Rust DTO compatibility with current Tauri command layer.
- Allows Swift to treat Rust calls as synchronous service calls initially, then move them to background tasks.

Potential later upgrade:

- UniFFI if the C ABI JSON bridge becomes too hard to maintain.

## Native UX acceptance criteria

### Window/app behavior

- Real `.app` launches without dev server.
- Standard macOS app menu exists:
  - Anahtar
  - File
  - Edit
  - View
  - Window
  - Help
- Standard shortcuts:
  - `⌘O` open vault
  - `⌘F` focus search
  - `⌘L` lock vault
  - `⌘R` refresh
  - `⌘W` close window
  - `⌘Q` quit
- Uses native file picker for vault and key-file.
- Uses native sheets/alerts for destructive confirmation.
- App remembers recent vault paths only.

### Layout

- Native-feeling 3-pane split view:
  - Groups
  - Entries
  - Detail
- Browse-first after unlock.
- Group counts include descendant entries.
- Entry detail has inline copy/reveal/TOTP actions.
- Password reveal is a toggle and removes password from UI state when hidden.
- TOTP copy is disabled when no TOTP is available.

### Writes

- Add/edit/delete/group/move use Rust backend only.
- Writes use safe in-place mode.
- UI shows final target path and backup path after write.
- Generated fixture warning remains for `test-vaults/generated/phase3-base.kdbx`.

### Security

- No persisted master password.
- No persisted entry secrets/TOTP secrets.
- No logging of secrets.
- Clipboard clear policy remains: clear only if clipboard still contains Anahtar-owned value.
- Key-file path may be remembered; key-file contents are never stored.

## Migration slices

### Slice N1 — Native project scaffold

- Add `apps/anahtar-macos`.
- Create minimal SwiftUI app window.
- Add placeholder 3-pane layout.
- Add native app menu and shortcuts.
- Build locally with Xcode/xcodebuild.

Verification:

```bash
xcodebuild -project apps/anahtar-macos/Anahtar.xcodeproj -scheme Anahtar build
```

### Slice N2 — Rust FFI bridge scaffold

- Add `crates/anahtar-ffi`.
- Expose `anahtar_backend_status()` and memory-free helper.
- Link static Rust library into Swift app.
- Show backend status in native UI.

Verification:

- Swift app displays Rust backend version/service.
- Rust tests pass.

### Slice N3 — Unlock/list/search/show

- Port current Tauri DTO semantics to FFI JSON.
- Native unlock screen with `NSOpenPanel` file selection.
- In-memory session object in Swift.
- List/search/detail safe by default.

Verification:

- Generated test vault unlocks with `testpass`.
- Wrong password shows friendly error.
- No password persisted.

### Slice N4 — Clipboard/TOTP/reveal

- Native copy actions.
- Owned clipboard clear timer.
- Reveal/hide toggle.
- TOTP disabled when unavailable.

Verification:

- Clipboard clears only if unchanged.
- TOTP secrets/URIs are not shown in normal detail.

### Slice N5 — Writes/groups/audit

- Add/edit/delete entries.
- Add/rename/delete groups.
- Move entry via edit group path or native picker.
- Audit panel.
- Main UI shows backup/final target paths.

Verification:

- Mutates a copied generated vault.
- Reopen app and vault; writes persist.
- Backups are created.

### Slice N6 — Replace Tauri packaging path

- Document native app build/package flow.
- Decide whether Tauri GUI stays as legacy alpha or is removed later.
- Build `.app`/archive locally.

Verification:

- Native `.app` launches outside dev environment.
- Full workspace Rust validation passes.

## Risks

- FFI memory ownership and string lifetime bugs.
- macOS codesigning/notarization complexity.
- Swift/Rust build integration time.
- Native feature parity may delay Phase 8 packaging.
- Current Tauri app will temporarily duplicate UI effort until deprecated.

## Recommendation

Proceed with N1 and N2 first. Do not delete or destabilize the current Tauri GUI until the native app can unlock/list/show a vault through the Rust backend.

The first success milestone is:

> A native SwiftUI macOS window launches, calls Rust through `anahtar-ffi`, opens a generated vault, lists entries, and shows safe entry detail without storing secrets.
