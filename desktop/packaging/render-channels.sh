#!/usr/bin/env sh
# Renders the Homebrew cask, the Homebrew formula and the Scoop manifest
# from the templates, with the checksums of the built artifacts.
#
#   packaging/render-channels.sh <version> <dist dir> <out dir>
set -eu
version=$1
dist=$2
out=$3
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sha() { sha256sum "$dist/$1" | cut -d' ' -f1; }
mkdir -p "$out"
sed -e "s/{{VERSION}}/$version/g" \
    -e "s/{{SHA_MACOS_ARM64}}/$(sha "iibrain-$version-macos-arm64.dmg")/" \
    -e "s/{{SHA_MACOS_X86_64}}/$(sha "iibrain-$version-macos-x86_64.dmg")/" \
    "$here/homebrew/iibrain.rb.tmpl" > "$out/iibrain-cask.rb"
sed -e "s/{{VERSION}}/$version/g" \
    -e "s/{{SHA_LINUX_ARM64}}/$(sha "iibrain-$version-linux-arm64.tar.gz")/" \
    -e "s/{{SHA_LINUX_X86_64}}/$(sha "iibrain-$version-linux-x86_64.tar.gz")/" \
    "$here/homebrew/iibrain-linux.rb.tmpl" > "$out/iibrain-formula.rb"
sed -e "s/{{VERSION}}/$version/g" \
    -e "s/{{SHA_WINDOWS_X86_64}}/$(sha "iibrain-$version-windows-x86_64.zip")/" \
    "$here/scoop/iibrain.json.tmpl" > "$out/iibrain.json"
ls -1 "$out"
