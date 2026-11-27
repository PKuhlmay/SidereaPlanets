#!/usr/bin/env bash
# Rebuilds the app icon files of the app from the Icon Composer layers in docs/design/icon (MAC-012).
#
#   scripts/make-icons.sh            everything below
#   scripts/make-icons.sh --check    only render the master PNG into the build folder (no changes in the repo)
#
# What it does:
#   1. renders the blurred planet layer (docs/design/icon/src/planet-blur.svg) to AppIcon.icon/Assets/planet-blur.png, because
#      Icon Composer ignores SVG filters, and copies the shared layers into AppIconDev.icon (its own copy plus the DEV badge);
#   2. stacks the layers of AppIcon.icon on the squircle background and writes docs/design/icon/AppIcon-1024.png, the master;
#   3. runs `pnpm tauri icon` on the master and writes the Windows set (icon.ico, icon.icns, PNGs, Square*Logo) to
#      app/src-tauri/icons/; the 16, 24 and 32 px sizes come from the simplified src/AppIcon-small.svg (readable when tiny).
#
# Needs rsvg-convert (brew install librsvg), python3 and pnpm; step 3 runs `pnpm install --frozen-lockfile` in app/ when needed.
# Edit the SVG layers (or the .icon in Icon Composer), run this script, commit the changed files. Runs with the bash 3.2 of macOS.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
icon="$root/docs/design/icon"
assets="$icon/AppIcon.icon/Assets"
dev_assets="$icon/AppIconDev.icon/Assets"
master="$icon/AppIcon-1024.png"
work="$root/build/icon"

check=0
case "${1:-}" in
  "") ;;
  --check) check=1; master="$work/AppIcon-1024.png" ;;
  *) echo "usage: scripts/make-icons.sh [--check]" >&2; exit 2 ;;
esac

command -v rsvg-convert >/dev/null || { echo "rsvg-convert is missing: brew install librsvg" >&2; exit 1; }
mkdir -p "$work"

# 1. Layers ---------------------------------------------------------------------------------------------------------------
planet_blur="$work/planet-blur.png"
rsvg-convert -w 1024 -h 1024 "$icon/src/planet-blur.svg" -o "$planet_blur"
if [ "$check" = 0 ]; then
  cp "$planet_blur" "$assets/planet-blur.png"
  for layer in frame-back.svg frame-middle.svg frame-front.svg planet-glow.svg planet-sharp.svg planet-blur.png; do
    cp "$assets/$layer" "$dev_assets/$layer"
  done
fi

# 2. Master PNG -----------------------------------------------------------------------------------------------------------
# The layers are drawn on the 824-unit grid of the macOS icon (100 px margin). Windows icons fill more of the canvas, so the
# whole stack is scaled up around the centre to a 944 px squircle (40 px margin). The shadows (an explicit blur/offset/merge chain: rsvg's feDropShadow tints dark colours) stand in for the neutral
# shadows of the layer groups in icon.json (back 0.3, middle 0.4, front group 0.5).
inline() { sed -e '/<?xml/d' "$1"; }
back=$(inline "$assets/frame-back.svg")
middle=$(inline "$assets/frame-middle.svg")
front=$(inline "$assets/frame-front.svg")
glow=$(inline "$assets/planet-glow.svg")
sharp=$(inline "$assets/planet-sharp.svg")
svg="$work/AppIcon-1024.svg"
cat > "$svg" <<EOF
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 1024 1024" width="1024" height="1024">
<defs>
  <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="rgb(52,48,44)"/><stop offset="1" stop-color="rgb(20,19,18)"/>
  </linearGradient>
  <clipPath id="squircle"><rect x="100" y="100" width="824" height="824" rx="185"/></clipPath>
  <filter id="shadow-back" color-interpolation-filters="sRGB" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur in="SourceAlpha" stdDeviation="14"/><feOffset dy="8" result="o"/><feFlood flood-color="#000" flood-opacity=".3"/><feComposite in2="o" operator="in" result="s"/><feMerge><feMergeNode in="s"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
  <filter id="shadow-middle" color-interpolation-filters="sRGB" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur in="SourceAlpha" stdDeviation="14"/><feOffset dy="8" result="o"/><feFlood flood-color="#000" flood-opacity=".4"/><feComposite in2="o" operator="in" result="s"/><feMerge><feMergeNode in="s"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
  <filter id="shadow-front" color-interpolation-filters="sRGB" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur in="SourceAlpha" stdDeviation="14"/><feOffset dy="8" result="o"/><feFlood flood-color="#000" flood-opacity=".5"/><feComposite in2="o" operator="in" result="s"/><feMerge><feMergeNode in="s"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
</defs>
<g transform="translate(512 512) scale(1.145631) translate(-512 -512)">
  <rect x="100" y="100" width="824" height="824" rx="185" fill="url(#bg)"/>
  <g clip-path="url(#squircle)">
    <g filter="url(#shadow-back)">$back</g>
    <g filter="url(#shadow-middle)">$middle</g>
    <g filter="url(#shadow-front)">$front$glow<image x="0" y="0" width="1024" height="1024" xlink:href="planet-blur.png"/>$sharp</g>
  </g>
</g>
</svg>
EOF
rsvg-convert -w 1024 -h 1024 "$svg" -o "$master"
echo "wrote ${master#"$root"/}"

[ "$check" = 1 ] && exit 0

# 3. Windows icon set -----------------------------------------------------------------------------------------------------
cd "$root/app"
[ -d node_modules ] || pnpm install --frozen-lockfile
pnpm tauri icon "$master" --output src-tauri/icons
# Tauri also writes the mobile sets; the app targets Windows (and desktop), so they stay out of the repo.
rm -rf src-tauri/icons/android src-tauri/icons/ios

# Up to 32 px the stack of three frames melts into one blob and the divider vanishes: those sizes (and 32x32.png, the window icon)
# come from the simplified docs/design/icon/src/AppIcon-small.svg, the larger ones from the master. icon.ico is assembled here.
for size in 16 24 32; do
  rsvg-convert -w "$size" -h "$size" "$icon/src/AppIcon-small.svg" -o "$work/ico-$size.png"
done
for size in 48 64 256; do
  rsvg-convert -w "$size" -h "$size" "$svg" -o "$work/ico-$size.png"
done
# An .ico is a small header plus the PNG files; written by hand so every size stays a compressed PNG (ImageMagick stores 256 px as BMP).
python3 - "$work" src-tauri/icons/icon.ico <<'PY'
import struct, sys
work, out = sys.argv[1], sys.argv[2]
sizes = [16, 24, 32, 48, 64, 256]
images = [open(f"{work}/ico-{size}.png", "rb").read() for size in sizes]
header = struct.pack("<HHH", 0, 1, len(sizes))
offset = 6 + 16 * len(sizes)
entries = b""
for size, data in zip(sizes, images):
    entries += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
open(out, "wb").write(header + entries + b"".join(images))
PY
cp "$work/ico-32.png" src-tauri/icons/32x32.png
echo "wrote app/src-tauri/icons/"
