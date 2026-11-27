#!/usr/bin/env bash
# Builds a shippable version of the Windows app (SP-026), like Siderea's windows/scripts/publish.sh: a plain folder plus a ZIP of it,
# optionally an installer.
#   scripts/publish-windows.sh               build/windows/dist/Siderea Planets/ and the ZIP
#   scripts/publish-windows.sh --installer   also build/windows/dist/Siderea-Planets-<version>-setup.exe (Inno Setup, app/installer/SidereaPlanets.iss)
#
#   build/windows/dist/Siderea Planets/                   Siderea Planets.exe (frontend built in) and CHANGELOG.md
#   build/windows/dist/Siderea-Planets-<version>-win-x64.zip      the same folder, zipped; <version> is "version" in app/package.json
#   build/windows/dist/Siderea-Planets-<version>-setup.exe        the installer (--installer only)
#   build/windows/dist/Siderea-Planets-<version>-SHA256SUMS.txt   SHA-256 of the ZIP (and the installer): "sha256sum -c" next to the files
#   release/Siderea-Planets-<version>-setup.exe (and the SHA256SUMS)  a copy of the installer in the repo root (Philipp, 2026-11-13),
#                                                                   git-ignored like build/
#
# app/CHANGELOG.md needs a section "## <version> — <date>" with "### English" and "### Deutsch": the installer shows it before it installs.
# The siderea-release skill (.claude/skills/siderea-release) sets the version and writes that section first.
set -euo pipefail

installer=0
for arg in "$@"; do
  case "$arg" in
    --installer) installer=1 ;;
    *) echo "usage: publish-windows.sh [--installer]" >&2; exit 2 ;;
  esac
done

case "$(uname -s)" in
  MINGW*|MSYS*) ;;
  *) echo "error: the Windows app is built on Windows" >&2; exit 1 ;;
esac

export PATH="$HOME/.cargo/bin:$PATH"
root=$(git rev-parse --show-toplevel)
app_dir="$root/app"
version=$(node -p "require('$app_dir/package.json').version")
conf_version=$(node -p "require('$app_dir/src-tauri/tauri.conf.json').version")
[ "$version" = "$conf_version" ] || { echo "error: app/package.json says $version, tauri.conf.json says $conf_version" >&2; exit 1; }

dist_dir="$root/build/windows/dist"
out_dir="$dist_dir/Siderea Planets"
zip_file="$dist_dir/Siderea-Planets-$version-win-x64.zip"
setup_file="$dist_dir/Siderea-Planets-$version-setup.exe"
sums_file="$dist_dir/Siderea-Planets-$version-SHA256SUMS.txt"
mkdir -p "$dist_dir"
rm -rf "$out_dir" "$zip_file" "$setup_file" "$sums_file"
mkdir -p "$out_dir"

# The release build of the Tauri app; the frontend is built into the exe. No bundle: the installer is Inno Setup's, not Tauri's NSIS.
(cd "$app_dir" && CARGO_INCREMENTAL=0 pnpm tauri build --no-bundle)
exe="$root/target/release/planets-app.exe"
[ -f "$exe" ] || { echo "error: $exe was not built" >&2; exit 1; }
cp "$exe" "$out_dir/Siderea Planets.exe"

# The changelog travels with the app, and its section of this version is what the installer shows before it installs (one text file per
# language, UTF-8 with a byte order mark and CRLF line endings, which is what Inno Setup reads as Unicode text).
changelog="$app_dir/CHANGELOG.md"
[ -f "$changelog" ] || { echo "error: app/CHANGELOG.md is missing" >&2; exit 1; }
cp "$changelog" "$out_dir/CHANGELOG.md"
section="$(awk -v v="$version" '/^## /{ keep = ($2 == v) } keep' "$changelog")"
[ -n "$section" ] || { echo "error: app/CHANGELOG.md has no section '## $version'" >&2; exit 1; }
notes_dir="$root/build/windows/notes"
mkdir -p "$notes_dir"
write_notes() { # <heading in the changelog> <title> <file>
  { printf '\xEF\xBB\xBF%s\n\n' "$2"
    echo "$section" | awk -v heading="### $1" '/^### /{ keep = ($0 == heading); next } keep && /^- /{ print; print "" }'
  } | sed 's/$/\r/' > "$3"
}
write_notes English "Siderea Planets for Windows $version - what is new" "$notes_dir/notes-en.txt"
write_notes Deutsch "Siderea Planets für Windows $version - was ist neu" "$notes_dir/notes-de.txt"
for notes in en de; do grep -q '^- ' "$notes_dir/notes-$notes.txt" || { echo "error: app/CHANGELOG.md has no $notes text for $version" >&2; exit 1; }; done

# bsdtar ships with Windows; use it by absolute path because Git Bash's own tar cannot write ZIPs.
(cd "$dist_dir" && /c/Windows/System32/tar.exe -a -c -f "$(basename "$zip_file")" "Siderea Planets")

echo "folder: $out_dir ($(du -sh "$out_dir" | cut -f1), $(find "$out_dir" -type f | wc -l) files)"
echo "zip:    $zip_file ($(du -h "$zip_file" | cut -f1))"
(cd "$dist_dir" && sha256sum "$(basename "$zip_file")" > "$sums_file")

if [ "$installer" = 1 ]; then
  # Microsoft's WebView2 bootstrapper (about 2 MB), which Setup runs only where the runtime is missing. Fetched once and kept.
  webview2="$root/build/windows/cache/MicrosoftEdgeWebview2Setup.exe"
  if [ ! -s "$webview2" ]; then
    mkdir -p "$(dirname "$webview2")"
    curl -fsSL -o "$webview2" "https://go.microsoft.com/fwlink/p/?LinkId=2124703" || { echo "error: could not fetch the WebView2 bootstrapper" >&2; exit 1; }
  fi
  # winget install JRSoftware.InnoSetup puts it in the user's programs folder (or Program Files when installed for all users).
  iscc=""
  for candidate in "${LOCALAPPDATA:-}/Programs/Inno Setup 6/ISCC.exe" "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" "/c/Program Files/Inno Setup 6/ISCC.exe"; do
    [ -x "$candidate" ] && { iscc="$candidate"; break; }
  done
  [ -n "$iscc" ] || { echo "error: ISCC.exe not found; run: winget install JRSoftware.InnoSetup" >&2; exit 1; }
  # MSYS_NO_PATHCONV keeps Git Bash from rewriting the /D switches into paths.
  MSYS_NO_PATHCONV=1 "$iscc" /Qp "/DAppVersion=$version" "/DSourceDir=$(cygpath -w "$out_dir")" "/DOutputDir=$(cygpath -w "$dist_dir")" \
    "/DNotesEn=$(cygpath -w "$notes_dir/notes-en.txt")" "/DNotesDe=$(cygpath -w "$notes_dir/notes-de.txt")" \
    "/DWebView2Setup=$(cygpath -w "$webview2")" "$(cygpath -w "$app_dir/installer/SidereaPlanets.iss")"
  [ -f "$setup_file" ] || { echo "error: $setup_file was not written" >&2; exit 1; }
  echo "setup:  $setup_file ($(du -h "$setup_file" | cut -f1))"
  (cd "$dist_dir" && sha256sum "$(basename "$setup_file")" >> "$sums_file")
fi
echo "sums:   $sums_file"

if [ "$installer" = 1 ]; then
  # The installer, the one file that is handed out, also lies in release/ of the repo root.
  mkdir -p "$root/release"
  cp "$setup_file" "$sums_file" "$root/release/"
  echo "copy:   $root/release/$(basename "$setup_file")"
fi
