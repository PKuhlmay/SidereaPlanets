#!/usr/bin/env bash
# Regenerates the stored reference results of the regression tests in re-local/reference-own/ (not in Git).
#
# Run it only after an intended, reviewed change of the algorithms (stacking, alignment, wavelets); see the section
# "Regressionspruefung" in docs/stacking.md and docs/wavelets.md. The input is re-local/testdata/astropix/JUPITER.AVI;
# set PLANETS_RE_LOCAL to use another re-local/ (a worktree finds the main checkout's by itself).
set -euo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"
cd "$(dirname "$0")/.."
cargo test -p planets-io --release --test reference_gen -- --ignored --nocapture
