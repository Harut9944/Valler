#!/usr/bin/env bash
# Packages a Linux build as valler-<version>-linux-<arch>.flatpak, a single-file
# bundle: flatpak install --user valler-<version>-linux-<arch>.flatpak
#
# usage: ci/package-flatpak.sh <path/to/valler> <arch> <version> <out-dir>
#   <arch> is x86_64 or aarch64. Needs flatpak and flatpak-builder; the
#   Freedesktop runtime and SDK are installed from Flathub if missing.
set -euo pipefail

bin="$1"; arch="$2"; version="$3"; out="$4"
here="$(cd "$(dirname "$0")/.." && pwd)"
app_id="io.github.harut9944.Valler"
name="valler-${version}-linux-${arch}"
work="$(mktemp -d)"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

# flatpak-builder reads its sources next to the manifest.
cp "$here/ci/flatpak/${app_id}.yml" "$here/ci/valler.desktop" "$work/"
install -m 755 "$bin" "$work/valler"
"$bin" --export-icon "$work/valler.png" 256

flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
cd "$work"
flatpak-builder --user --arch="$arch" --install-deps-from=flathub --force-clean \
  --default-branch=stable --repo=repo build "${app_id}.yml"
flatpak build-bundle --arch="$arch" --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
  repo "$out/${name}.flatpak" "$app_id" stable
echo "$out/${name}.flatpak"
