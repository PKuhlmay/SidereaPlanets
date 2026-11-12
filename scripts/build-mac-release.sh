#!/usr/bin/env bash
# Builds the release bundle of the Mac app (MAC-010): the Rust interface for arm64 and x86_64, the app as a universal binary,
# signed ad hoc (or with a Developer ID, see below), and a zip next to it. Nothing here is published or notarized; /release mac
# calls this script for the checks and the bundle, and mac/README.md ("Bauen und veröffentlichen") explains the rest.
#
#   scripts/build-mac-release.sh           build/release/SidereaPlanets.app and build/release/SidereaPlanets-<version>-mac.zip
#   scripts/build-mac-release.sh --dmg     additionally a disk image, build/release/SidereaPlanets-<version>-mac.dmg
#
# SIGN_IDENTITY: the codesign identity. Default "-" is ad hoc ("Sign to Run Locally"); another one (for example
# "Developer ID Application: Name (TEAMID)") also turns on the hardened runtime that notarization needs.
# Everything is written below build/ (git-ignored). Runs with the bash 3.2 of macOS. Takes a few minutes: the core is built twice.
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

make_dmg=0
case "${1:-}" in
  "") ;;
  --dmg) make_dmg=1 ;;
  *) echo "usage: scripts/build-mac-release.sh [--dmg]" >&2; exit 2 ;;
esac

identity="${SIGN_IDENTITY:--}"
derived="$root/build/mac-release"
out="$root/build/release"
name="SidereaPlanets"

echo "== Rust interface, release, arm64 + x86_64"
scripts/build-ffi.sh --release

echo "== xcodegen"
(cd mac && xcodegen generate)

echo "== xcodebuild Release, universal"
rm -rf "$derived"
xcodebuild -project mac/SidereaPlanets.xcodeproj -scheme "$name" -configuration Release \
  -destination 'generic/platform=macOS' ARCHS="arm64 x86_64" ONLY_ACTIVE_ARCH=NO \
  -derivedDataPath "$derived" build | tail -n 5

built="$derived/Build/Products/Release/$name.app"
[ -d "$built" ] || { echo "no app at $built" >&2; exit 1; }

rm -rf "$out"
mkdir -p "$out"
ditto "$built" "$out/$name.app"
app="$out/$name.app"

echo "== codesign ($identity)"
if [ "$identity" = "-" ]; then
  codesign --force --deep --sign - "$app"
else
  codesign --force --deep --options runtime --timestamp --sign "$identity" "$app"
fi
codesign --verify --deep --strict "$app"

version=$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" "$app/Contents/Info.plist")
echo "== architectures"
lipo -info "$app/Contents/MacOS/$name"

echo "== zip"
zip="$out/$name-$version-mac.zip"
ditto -c -k --sequesterRsrc --keepParent "$app" "$zip"

if [ "$make_dmg" = 1 ]; then
  echo "== dmg"
  dmg="$out/$name-$version-mac.dmg"
  hdiutil create -volname "$name $version" -srcfolder "$app" -ov -format UDZO "$dmg" >/dev/null
  [ "$identity" = "-" ] || codesign --force --sign "$identity" --timestamp "$dmg"
fi

echo "build-mac-release.sh ok: $app (version $version, signed with $identity)"
echo "  $zip"
[ "$make_dmg" = 1 ] && echo "  $dmg"
exit 0
