#!/usr/bin/env bash
# Packages a Linux build as valler-<version>-linux-<arch>.AppImage: one file
# that runs on most distributions (chmod +x, then start it).
#
# usage: ci/package-appimage.sh <path/to/valler> <arch> <version> <out-dir>
#   <arch> is x86_64 or aarch64. Needs appimagetool on PATH, or set
#   APPIMAGETOOL to its path.
set -euo pipefail

bin="$1"; arch="$2"; version="$3"; out="$4"
here="$(cd "$(dirname "$0")/.." && pwd)"
tool="${APPIMAGETOOL:-appimagetool}"
name="valler-${version}-linux-${arch}"
appdir="$(mktemp -d)/Valler.AppDir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
  "$appdir/usr/share/icons/hicolor/256x256/apps" "$out"

install -m 755 "$bin" "$appdir/usr/bin/valler"
"$bin" --export-icon "$appdir/usr/share/icons/hicolor/256x256/apps/valler.png" 256
install -m 644 "$here/ci/valler.desktop" "$appdir/usr/share/applications/valler.desktop"

# appimagetool takes the launcher entry and icon from the AppDir's top level.
ln -s usr/share/applications/valler.desktop "$appdir/valler.desktop"
ln -s usr/share/icons/hicolor/256x256/apps/valler.png "$appdir/valler.png"
ln -s valler.png "$appdir/.DirIcon"
ln -s usr/bin/valler "$appdir/AppRun"

# Runs without FUSE (CI containers often lack it).
ARCH="$arch" APPIMAGE_EXTRACT_AND_RUN=1 VERSION="$version" \
  "$tool" --no-appstream "$appdir" "$out/${name}.AppImage"
chmod 755 "$out/${name}.AppImage"
echo "$out/${name}.AppImage"
