#!/usr/bin/env bash
# Deploys the Mac app after a commit on main: builds mac/ (xcodegen + xcodebuild, Debug, arm64) and, only if that worked,
# restarts the app so Philipp always sees the latest state (CLAUDE.md, "Deploy nach jedem Commit"). Started by .githooks/post-commit
# on macOS; the Windows counterpart is scripts/deploy-debug.sh. Runs with the bash 3.2 of macOS.
#
#   scripts/deploy-mac.sh          called by the hook: checks the commit, then re-runs itself detached
#   scripts/deploy-mac.sh --run    does the deploy in the foreground (also usable by hand)
#
# Tests are off by default, as in deploy-debug.sh; DEPLOY_TESTS=1 runs `cargo test --workspace` first.
# DEPLOY_DEBUG=0 switches the deploy off. Output goes to deploy-mac.log in the repo root (git-ignored).
# A failed build leaves the running app untouched.
set -u
[ "${DEPLOY_DEBUG:-1}" = "0" ] && exit 0
# rustup from Homebrew is keg-only, so its bin folder is not on the PATH of a git hook.
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"

root=$(git rev-parse --show-toplevel)
lock="$root/.git/deploy-mac.lock"
log="$root/deploy-mac.log"
app_name="SidereaPlanets"
derived="$root/build/mac"
app="$derived/Build/Products/Debug/$app_name.app"
# Matches the running process even when the repo path is reached through a symlink (e.g. /tmp).
match="build/mac/Build/Products/Debug/$app_name.app/Contents/MacOS/"

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

# Quits the running app (asks politely first, then by its bundle path) and starts the freshly built one.
restart_app() {
  osascript -e "quit app \"$app_name\"" >/dev/null 2>&1 || true
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -f "$match" >/dev/null 2>&1 || break
    sleep 0.5
  done
  pkill -f "$match" >/dev/null 2>&1 || true
  # LaunchServices needs a moment after a quit; `open` fails with error -600 if it is asked too early.
  for _ in 1 2 3 4 5; do
    sleep 1
    open "$app" && return 0
  done
  echo "could not start $app"
}

cd "$root" || exit 1
while :; do
  rm -f "$lock.again"
  echo "=== $(date '+%Y-%m-%d %H:%M:%S') deploy $(git log -1 --format='%h %s')"

  if [ ! -f mac/project.yml ]; then
    echo "mac/project.yml missing, nothing to deploy"
  elif [ "${DEPLOY_TESTS:-0}" = 1 ] && ! cargo test --workspace; then
    echo "tests failed, app not deployed"
  elif [ -f scripts/build-ffi.sh ] && ! bash scripts/build-ffi.sh; then
    echo "build-ffi.sh failed, app not deployed"
  elif ! command -v xcodegen >/dev/null 2>&1; then
    echo "xcodegen not found (brew install xcodegen), app not deployed"
  elif ! (cd mac && xcodegen generate); then
    echo "xcodegen failed, app not deployed"
  elif ! xcodebuild -quiet -project mac/SidereaPlanets.xcodeproj -scheme SidereaPlanets -configuration Debug \
      -arch arm64 -derivedDataPath "$derived" build; then
    echo "xcodebuild failed, running app left untouched"
  elif [ ! -d "$app" ]; then
    echo "build succeeded but $app is missing, running app left untouched"
  else
    echo "build ok, restarting $app_name"
    restart_app
  fi

  [ -f "$lock.again" ] || break
done
