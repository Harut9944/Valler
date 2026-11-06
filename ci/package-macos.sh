#!/usr/bin/env bash
# Wraps a macOS build in Valler.app (with an .icns icon and Info.plist),
# signs it ad hoc and zips it as valler-<version>-macos-<arch>.zip.
#
# usage: ci/package-macos.sh <path/to/valler> <arch> <version> <out-dir>
set -euo pipefail

bin="$1"; arch="$2"; version="$3"; out="$4"
here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
app="$work/Valler.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$out"

install -m 755 "$bin" "$app/Contents/MacOS/valler"

# Icon: every size macOS asks for, from the 1024 px master.
iconset="$work/valler.iconset"
mkdir -p "$iconset"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" "$here/assets/valler.png" --out "$iconset/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z "$d" "$d" "$here/assets/valler.png" --out "$iconset/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/valler.icns"

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Valler</string>
  <key>CFBundleDisplayName</key><string>Valler</string>
  <key>CFBundleIdentifier</key><string>app.valler.Valler</string>
  <key>CFBundleExecutable</key><string>valler</string>
  <key>CFBundleIconFile</key><string>valler</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSHumanReadableCopyright</key><string>GPL-3.0-or-later</string>
</dict>
</plist>
EOF

# Ad-hoc signature: required for Apple silicon to run the binary. (A real
# Developer ID signature and notarization need the maintainer's certificate.)
codesign --force --deep --sign - "$app"

zip="$out/valler-${version}-macos-${arch}.zip"
ditto -c -k --keepParent "$app" "$zip"
echo "$zip"
