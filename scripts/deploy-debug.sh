#!/usr/bin/env bash
# Deploys the debug app after a commit on main: installs dependencies, builds the frontend and makes sure `pnpm tauri dev` is running,
# so Philipp always sees the latest state (CLAUDE.md, "Deploy nach jedem Commit"). Started by .githooks/post-commit.
#
#   scripts/deploy-debug.sh          called by the hook: checks the commit, then re-runs itself detached
#   scripts/deploy-debug.sh --run    does the deploy in the foreground (also usable by hand)
#
# Tests are off by default (temporary request, see CLAUDE.md); DEPLOY_TESTS=1 runs `cargo test --workspace` first.
# Output goes to deploy-debug.log and tauri-dev.log in the repo root (both git-ignored).
set -u
# DEPLOY_DEBUG=0 switches the deploy off (e.g. on the Mac, where the SwiftUI app is the one that matters).
[ "${DEPLOY_DEBUG:-1}" = "0" ] && exit 0
export PATH="$HOME/.cargo/bin:$PATH"

root=$(git rev-parse --show-toplevel)
# Installs run one at a time across sessions (CLAUDE.md); the wrapper exists on Philipp's machine only.
install=""
[ -x "$HOME/.claude/bin/heavy-lock" ] && install="$HOME/.claude/bin/heavy-lock"
lock="$root/.git/deploy.lock"
log="$root/deploy-debug.log"

if [ "${1:-}" != "--run" ]; then
  # Only the main checkout on main: other worktrees and branches are work in progress, and the squash merge lands on main.
  [ "$(git rev-parse --git-dir)" = "$(git rev-parse --git-common-dir)" ] || exit 0
  [ "$(git symbolic-ref --short -q HEAD)" = "main" ] || exit 0
  nohup bash "$0" --run >>"$log" 2>&1 </dev/null &
  exit 0
fi

# One deploy at a time. A commit that arrives meanwhile sets a flag and the running deploy goes round once more.
if ! mkdir "$lock" 2>/dev/null; then
  touch "$lock.again"
  exit 0
fi
trap 'rmdir "$lock" 2>/dev/null' EXIT

cd "$root" || exit 1
while :; do
  rm -f "$lock.again"
  echo "=== $(date '+%Y-%m-%d %H:%M:%S') deploy $(git log -1 --format='%h %s')"

  if [ "${DEPLOY_TESTS:-0}" = 1 ] && ! cargo test --workspace; then
    echo "tests failed, app not deployed"
  elif ! (cd app && $install pnpm install --frozen-lockfile); then
    echo "pnpm install failed, app not deployed"
  elif ! (cd app && pnpm build); then
    echo "frontend build failed, app not deployed"
  elif curl -s -o /dev/null --max-time 2 http://localhost:1420; then
    echo "pnpm tauri dev is running and picks up the change"
  else
    echo "starting pnpm tauri dev"
    (cd app && nohup pnpm tauri dev >>"$root/tauri-dev.log" 2>&1 </dev/null &)
  fi

  [ -f "$lock.again" ] || break
done
