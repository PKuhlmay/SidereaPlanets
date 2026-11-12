#!/usr/bin/env bash
# Builds the Rust interface (crates/ffi) for the Mac app: the static library, the Swift bindings (UniFFI library mode) and
# mac/Packages/SidereaPlanetsKit/SidereaPlanetsFFI.xcframework, and copies the generated Swift file into the package (MAC-004).
#
#   scripts/build-ffi.sh             debug, aarch64-apple-darwin only (fast; what deploy-mac.sh and `swift test` need)
#   scripts/build-ffi.sh --release   release for aarch64 and x86_64, joined to one universal library with lipo
#
# The outputs are git-ignored: the XCFramework and Sources/SidereaPlanetsBindings/SidereaPlanetsBindings.swift. Runs with the bash 3.2 of macOS.
set -euo pipefail
# rustup from Homebrew is keg-only, so its bin folder is not on the PATH of a git hook.
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
# The library is linked into an app that needs macOS 14; building for the same target keeps the linker quiet.
export MACOSX_DEPLOYMENT_TARGET="14.0"

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

release=0
case "${1:-}" in
  "") ;;
  --release) release=1 ;;
  *) echo "usage: scripts/build-ffi.sh [--release]" >&2; exit 2 ;;
esac

package="$root/mac/Packages/SidereaPlanetsKit"
work="$root/build/ffi"
lib="libplanets_ffi.a"

if [ "$release" = 1 ]; then
  profile="release"
  targets="aarch64-apple-darwin x86_64-apple-darwin"
  cargo_flags="--release"
else
  profile="debug"
  targets="aarch64-apple-darwin"
  cargo_flags=""
fi

for target in $targets; do
  echo "== cargo build -p planets-ffi ($profile, $target)"
  # shellcheck disable=SC2086
  cargo build -p planets-ffi --target "$target" $cargo_flags
done

rm -rf "$work"
mkdir -p "$work/bindings" "$work/headers" "$work/lib"

# One library for the XCFramework: the single build, or both architectures joined.
if [ "$release" = 1 ]; then
  lipo -create \
    "target/aarch64-apple-darwin/$profile/$lib" \
    "target/x86_64-apple-darwin/$profile/$lib" \
    -output "$work/lib/$lib"
else
  cp "target/aarch64-apple-darwin/$profile/$lib" "$work/lib/$lib"
fi

# The bindings come from the arm64 library (the metadata is the same in both) and from the bindgen of this workspace, so the
# generator and the library always have the same uniffi version.
echo "== uniffi-bindgen generate (Swift, library mode)"
cargo run -q -p planets-ffi --bin uniffi-bindgen -- generate \
  --library "target/aarch64-apple-darwin/$profile/$lib" \
  --language swift --out-dir "$work/bindings"

cp "$work/bindings/SidereaPlanetsFFI.h" "$work/headers/SidereaPlanetsFFI.h"
cp "$work/bindings/SidereaPlanetsFFI.modulemap" "$work/headers/module.modulemap"

echo "== xcodebuild -create-xcframework"
rm -rf "$package/SidereaPlanetsFFI.xcframework"
xcodebuild -create-xcframework \
  -library "$work/lib/$lib" -headers "$work/headers" \
  -output "$package/SidereaPlanetsFFI.xcframework"

mkdir -p "$package/Sources/SidereaPlanetsBindings"
cp "$work/bindings/SidereaPlanetsBindings.swift" "$package/Sources/SidereaPlanetsBindings/SidereaPlanetsBindings.swift"

echo "build-ffi.sh ok ($profile): $package/SidereaPlanetsFFI.xcframework"
