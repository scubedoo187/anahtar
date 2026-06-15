#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
APP_DIR="$ROOT/apps/anahtar-macos"
BUNDLE_DIR="$APP_DIR/build/Anahtar.app"
EXECUTABLE="$APP_DIR/.build/release/Anahtar"

cd "$ROOT"
cargo build -p anahtar-ffi --release --target aarch64-apple-darwin
swift build --package-path "$APP_DIR" -c release

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS" "$BUNDLE_DIR/Contents/Resources"
cp "$EXECUTABLE" "$BUNDLE_DIR/Contents/MacOS/Anahtar"
cat > "$BUNDLE_DIR/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>Anahtar</string>
  <key>CFBundleIdentifier</key>
  <string>com.anahtar.native-alpha</string>
  <key>CFBundleName</key>
  <string>Anahtar</string>
  <key>CFBundleDisplayName</key>
  <string>Anahtar</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

echo "Built $BUNDLE_DIR"
