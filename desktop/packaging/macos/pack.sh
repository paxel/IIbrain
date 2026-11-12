#!/usr/bin/env sh
# Packs the macOS release from a built binary: "IIbrain.app" in a .dmg,
# unsigned.
#
#   packaging/macos/pack.sh <version> <arch> <binary> <dist dir>
set -eu
version=$1
arch=$2
binary=$3
dist=$4
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
icons=$here/../icon
mkdir -p "$dist"
stage=$(mktemp -d)
app="$stage/dmgroot/IIbrain.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/iibrain"
chmod 0755 "$app/Contents/MacOS/iibrain"

# The icon: an iconset from the 1024 px PNG.
iconset="$stage/iibrain.iconset"
mkdir -p "$iconset"
# The ten names iconutil accepts: five sizes, each with its @2x twin.
for size in 16 32 128 256 512; do
  sips -z $size $size "$icons/icon.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z $double $double "$icons/icon.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/iibrain.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>IIbrain</string>
  <key>CFBundleDisplayName</key><string>IIbrain</string>
  <key>CFBundleIdentifier</key><string>io.github.paxel.iibrain</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleExecutable</key><string>iibrain</string>
  <key>CFBundleIconFile</key><string>iibrain</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
# Signed ad hoc, no Apple Developer certificate: an unsigned bundle reads
# as "damaged" on Apple silicon.
codesign --force --deep --sign - "$app"
ln -s /Applications "$stage/dmgroot/Applications"
hdiutil create -volname "IIbrain" -srcfolder "$stage/dmgroot" -ov -format UDZO \
  "$dist/iibrain-$version-macos-$arch.dmg" >/dev/null
rm -rf "$stage"
ls -1 "$dist"
