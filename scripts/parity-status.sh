#!/bin/bash
# Lists the open items of a follow-up list grouped by the first release whose commit added them, next to the latest Mac and
# Windows tags — the input of the /nachzug skill. Read-only; runs with the bash 3.2 of macOS.
# docs/parity/WINDOWS_PARITY.md (Windows catches up the Mac) groups by Mac tags (v*), docs/parity/MAC_PARITY.md (the Mac catches up
# Windows) by Windows tags (windows-v*).
# Usage: scripts/parity-status.sh [list]   (default: docs/parity/WINDOWS_PARITY.md of this checkout)
set -euo pipefail

cd "$(dirname "$0")/.."
parity="${1:-docs/parity/WINDOWS_PARITY.md}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# True when version $1 <= version $2 (both "v1.2.3" or "1.2.3").
not_newer() {
  awk -v a="${1#v}" -v b="${2#v}" 'BEGIN {
    split(a, x, "."); split(b, y, ".")
    for (i = 1; i <= 3; i++) { if (x[i] + 0 < y[i] + 0) exit 0; if (x[i] + 0 > y[i] + 0) exit 1 }
    exit 0 }'
}

latest_mac="$(git tag -l 'v[0-9]*' --sort=-v:refname | sed -n 1p)"
latest_windows="$(git tag -l 'windows-v[0-9]*' --sort=-v:refname | sed -n 1p)"
echo "Mac: ${latest_mac:-no tag} · Windows: ${latest_windows:-no tag}"

# The tags of the platform that added the items, and the latest release of the platform that catches up.
case "$parity" in
  *MAC_PARITY.md) source_tags='windows-v[0-9]*'; behind_tag="$latest_mac"; behind_name=Mac ;;
  *) source_tags='v[0-9]*'; behind_tag="$latest_windows"; behind_name=Windows ;;
esac
behind_version="${behind_tag#windows-}"

# line<TAB>section<TAB>text of every open checkbox
awk '/^## / { section = substr($0, 4) } /^- \[ \] / { print NR "\t" section "\t" substr($0, 7) }' "$parity" > "$tmp/open"

# line<TAB>commit of every line; every line's group starts with "<40 hex> <orig line> <final line> [count]"
# (a list that is not committed yet counts as uncommitted as a whole)
if git ls-files --error-unmatch -- "$parity" >/dev/null 2>&1; then
  git blame --porcelain -- "$parity" \
    | awk 'length($1) == 40 && $1 ~ /^[0-9a-f]+$/ { print $3 "\t" $1 }' > "$tmp/blame"
else
  awk '{ print NR "\t0000000000000000000000000000000000000000" }' "$parity" > "$tmp/blame"
fi

# commit<TAB>first Mac tag containing it
cut -f2 "$tmp/blame" | sort -u | while read -r commit; do
  if [ "$commit" = 0000000000000000000000000000000000000000 ]; then
    version=uncommitted
  else
    version="$(git tag --contains "$commit" -l "$source_tags" --sort=v:refname | sed -n 1p)"
  fi
  printf '%s\t%s\n' "$commit" "${version:-unreleased}"
done > "$tmp/versions"

awk -F '\t' 'FILENAME == ARGV[1] { version[$1] = $2; next }
             FILENAME == ARGV[2] { commit[$1] = $2; next }
             { print version[commit[$1]] "\t" $2 "\t" $3 }' "$tmp/versions" "$tmp/blame" "$tmp/open" > "$tmp/items"

echo "Open: $(wc -l < "$tmp/items" | tr -d ' ') of $(grep -c '^- \[[ x]\] ' "$parity" || true) items"
for version in $(git tag -l "$source_tags" --sort=v:refname) unreleased uncommitted; do
  count="$(awk -F '\t' -v v="$version" '$1 == v' "$tmp/items" | wc -l | tr -d ' ')"
  [ "$count" -eq 0 ] && continue
  note=""
  plain="${version#windows-}"
  if [ -n "$behind_version" ] && [ "${plain#v}" != "$plain" ] && not_newer "$plain" "$behind_version"; then
    note=" — $behind_name already released $behind_tag"
  fi
  echo
  echo "## $version ($count open)$note"
  awk -F '\t' -v v="$version" '$1 == v { print "- " $2 ": " $3 }' "$tmp/items"
done
