#!/usr/bin/env bash
# Prints the body of the "## <version>" section of app/CHANGELOG.md (the "### English" and "### Deutsch" lists, without the "## " heading),
# for the release notes of the release workflow (.github/workflows/release.yml).
#   scripts/changelog-section.sh 0.1.0
#   scripts/changelog-section.sh v1.0.0-rc.1 [path/to/CHANGELOG.md]
#
# A leading "v" (the tag) is dropped. A pre-release such as 1.0.0-rc.1 uses its own section when there is one and else the section of
# 1.0.0, so a dry run needs no extra changelog text. Exits 1 and says so when no section is found or it has no list entries.
set -euo pipefail

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
  echo "usage: changelog-section.sh <version|tag> [changelog]" >&2
  exit 2
fi

version="${1#v}"
changelog="${2:-$(dirname "$0")/../app/CHANGELOG.md}"
[ -f "$changelog" ] || { echo "error: $changelog not found" >&2; exit 1; }

# The text of one section: everything between its "## " heading and the next "## " heading, without leading and trailing blank lines.
section() { # <version>
  awk -v v="$1" '
    /^## / { keep = ($2 == v); next }
    keep { lines[++n] = $0 }
    END {
      first = 1; while (first <= n && lines[first] == "") first++
      last = n;  while (last >= first && lines[last] == "") last--
      for (i = first; i <= last; i++) print lines[i]
    }' "$changelog"
}

text="$(section "$version")"
if ! printf '%s\n' "$text" | grep -q '^- ' && [ "${version%%-*}" != "$version" ]; then
  text="$(section "${version%%-*}")"
fi
if ! printf '%s\n' "$text" | grep -q '^- '; then
  echo "error: $changelog has no section '## $version' with list entries" >&2
  exit 1
fi
printf '%s\n' "$text"
