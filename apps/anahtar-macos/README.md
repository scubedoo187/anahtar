# Anahtar Native macOS App

This is the native SwiftUI/AppKit transition app for Anahtar. It uses Rust through `crates/anahtar-ffi`; Swift owns native macOS UI only and must not reimplement KDBX logic.

## Build for development

```bash
cargo build -p anahtar-ffi --release --target aarch64-apple-darwin
swift build --package-path apps/anahtar-macos
```

The Swift package links the Rust static library from `target/aarch64-apple-darwin/release`.

## Build a local `.app`

```bash
apps/anahtar-macos/scripts/build-app.sh
open apps/anahtar-macos/build/Anahtar.app
```

The local bundle is unsigned and intended for alpha smoke testing only. Production signing/notarization is deferred.

## Current native alpha scope

- Native SwiftUI app shell with app menu commands.
- Native file picker for vault and key-file selection.
- Rust FFI bridge for backend status, unlock/list/search/show, TOTP, groups, audit, and writes.
- Clipboard copy uses `NSPasteboard` and clears only if Anahtar still owns the clipboard value.
- Writes go through Rust `AnahtarService` with safe in-place backup behavior.

## Safety reminders

- Master password is kept in process memory only for the current unlocked session.
- Recent vault persistence is not implemented in the native app yet.
- Do write testing against copied generated vaults, not the regenerated fixture or real vaults.
