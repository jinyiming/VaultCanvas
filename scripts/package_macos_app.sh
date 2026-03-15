#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/dist/VaultCanvas.app"
MACOS_DIR="$APP_DIR/Contents/MacOS"
RES_DIR="$APP_DIR/Contents/Resources"
BIN_SRC="$ROOT_DIR/target/release/vaultcanvas_macos_native"
BIN_DST="$MACOS_DIR/VaultCanvas"
ICON_SRC="$ROOT_DIR/apps/native_gui/assets/logo.icns"
PLIST="$APP_DIR/Contents/Info.plist"

cd "$ROOT_DIR"

cargo build -p vaultcanvas_macos_native --release

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RES_DIR"

cp "$BIN_SRC" "$BIN_DST"
chmod +x "$BIN_DST"

if [[ -f "$ICON_SRC" ]]; then
  cp "$ICON_SRC" "$RES_DIR/AppIcon.icns"
  ICON_KEY="<key>CFBundleIconFile</key><string>AppIcon</string>"
else
  ICON_KEY=""
fi

cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>zh_CN</string>
  <key>CFBundleDisplayName</key>
  <string>VaultCanvas</string>
  <key>CFBundleExecutable</key>
  <string>VaultCanvas</string>
  <key>CFBundleIdentifier</key>
  <string>com.vaultcanvas.app</string>
  ${ICON_KEY}
  <key>CFBundleName</key>
  <string>VaultCanvas</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0.0</string>
  <key>CFBundleVersion</key>
  <string>1.0.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

echo "macOS app bundle ready: $APP_DIR"
