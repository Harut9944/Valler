#!/usr/bin/env bash
# Packages a Linux build as valler-<version>-linux-<arch>.tar.gz with the
# program, a launcher entry, the icon and a small per-user installer.
#
# usage: ci/package-linux.sh <path/to/valler> <arch> <version> <out-dir>
set -euo pipefail

bin="$1"; arch="$2"; version="$3"; out="$4"
here="$(cd "$(dirname "$0")/.." && pwd)"
name="valler-${version}-linux-${arch}"
stage="$(mktemp -d)/${name}"
mkdir -p "$stage" "$out"

install -m 755 "$bin" "$stage/valler"
install -m 644 "$here/assets/valler.png" "$stage/valler.png"
install -m 644 "$here/README.md" "$stage/README.md"

install -m 644 "$here/ci/valler.desktop" "$stage/valler.desktop"

cat > "$stage/install.sh" <<'EOF'
#!/usr/bin/env bash
# Installs Valler for the current user (no root needed):
#   ~/.local/bin/valler, a launcher entry and the icon.
set -euo pipefail
cd "$(dirname "$0")"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
mkdir -p "$HOME/.local/bin" "$data/applications" "$data/icons/hicolor/256x256/apps"
install -m 755 valler "$HOME/.local/bin/valler"
install -m 644 valler.desktop "$data/applications/valler.desktop"
install -m 644 valler.png "$data/icons/hicolor/256x256/apps/valler.png"
echo "Valler is installed. Start it from your app menu, or run: valler"
EOF
chmod 755 "$stage/install.sh"

tar -C "$(dirname "$stage")" -czf "$out/${name}.tar.gz" "$name"
echo "$out/${name}.tar.gz"
