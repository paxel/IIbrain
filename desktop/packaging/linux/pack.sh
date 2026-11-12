#!/usr/bin/env sh
# Packs the Linux release from a built binary: the tarball the Homebrew
# formula installs.
#
#   packaging/linux/pack.sh <version> <arch> <binary> <dist dir>
#
# arch is x86_64 or arm64 (the artifact names).
set -eu
version=$1
arch=$2
binary=$3
dist=$4
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
icons=$here/../icon
root=$here/../../..
entry=io.github.paxel.iibrain.desktop
mkdir -p "$dist"
stage=$(mktemp -d)

# Everything flat, as the formula expects.
cp "$binary" "$stage/iibrain"
chmod 0755 "$stage/iibrain"
cp "$here/$entry" "$here/install-icon.sh" "$stage/"
chmod 0755 "$stage/install-icon.sh"
cp "$icons/icon_256.png" "$stage/"
cp "$root/LICENSE-APACHE" "$root/LICENSE-MIT" "$root/THIRD-PARTY-NOTICES.md" "$stage/"
tar -C "$stage" -czf "$dist/iibrain-$version-linux-$arch.tar.gz" .
rm -rf "$stage"
ls -1 "$dist"
