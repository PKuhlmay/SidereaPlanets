# Contributing to Siderea Planets

Thanks for helping. Bug reports, feature ideas, documentation fixes and pull requests are all welcome. For a larger change, open an
issue first so we can agree on the direction before you spend time on it.

Siderea Planets is licensed under the GPL-3.0-or-later. By contributing you agree that your contribution is licensed the same way.

## Setup

Prerequisites (Rust through rustup, Node 24, pnpm, cargo-about, the Tauri system libraries) are listed in the
[README](README.md#build-from-source).

```bash
git clone https://github.com/PKuhlmay/SidereaPlanets.git
cd SidereaPlanets
git config core.hooksPath .githooks     # once per clone, see "Git hook" below
cd app && pnpm install
```

## Development commands

```bash
cd app && pnpm tauri dev                  # run the app with hot reload
cargo test --workspace                    # Rust: computing core, readers, sessions
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cd app && pnpm build                      # frontend build (vue-tsc and vite)
cd app && pnpm test                       # frontend tests (vitest)
cd app && pnpm check:licenses             # licences of the npm packages
cargo deny check                          # licences and advisories of the Rust crates
cd app && pnpm tauri build                # installable app for your system
```

`cargo deny` comes from [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (`cargo install cargo-deny --locked`).
Do not start `target/debug/planets-app` directly: a debug build waits for the Vite dev server, which `pnpm tauri dev` starts.

## Git hook

`git config core.hooksPath .githooks` enables the `post-commit` hook. After a commit on `main` in your main checkout it builds the
debug app and starts `pnpm tauri dev`, so you always see the latest state. It does nothing on other branches or in worktrees.
Set `DEPLOY_DEBUG=0` to switch it off, for example in your shell profile or per commit:
`DEPLOY_DEBUG=0 git commit ...`. The hook is optional; nothing else depends on it.

## Before you open a pull request

CI runs all of this on macOS, Windows and Linux; run it locally first so the pull request starts green:

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cd app && pnpm test`
- [ ] `cargo deny check` and `cd app && pnpm check:licenses`

Tests that need recordings which are not in the repository skip themselves and pass.

Rules for the code:

- The computing core (`crates/`) has no GUI code and nothing platform-specific; it has to build and test on all three systems.
- `unsafe` code is forbidden in the workspace.
- TypeScript is strict: no `any`.
- Code comments are in English.
- New or changed behaviour comes with a test that fails without the change.
- Keep a change focused. Do not mix a refactoring with a bug fix.

## Dependencies and licences

Every new dependency has to be compatible with the GPL-3.0-or-later. Rust crates are checked by `cargo deny` against the list in
`deny.toml`, npm packages by `pnpm check:licenses`. If a licence is missing from the list, check it against the
[GNU license list](https://www.gnu.org/licenses/license-list.html) and explain it in the pull request before adding it. Prefer a
small amount of own code to a large dependency for one function.

## User documentation

The documentation inside the app lives in `app/src/docs/pages/` as Markdown, one file per page. When a change affects what users see
or do (buttons, texts, shortcuts, steps, values, new terms or formats), update the page in the same pull request. Describe how to
use the feature and what comes out of it. Where something does not work yet, say **Not available yet** on the page.

## Commits

```
[TYPE] PLANETS-N - Message in English
```

- Types: `[TASK]`, `[FEATURE]`, `[BUGFIX]`. Put `!!!` in front for a breaking change.
- `PLANETS-N` is the maintainer's ticket key. **External contributors may leave the ticket out**, for example
  `[BUGFIX] Fix the crash when a SER file has no frames`; the maintainer adds the key when merging.
- Imperative mood, no full stop, details in the body.

## Reporting bugs and ideas

Use the issue forms: a bug report asks for your system, the app version (shown in the status bar), the format of the recording and
the steps to reproduce. Please do not attach large recordings to an issue; describe them and offer a short excerpt instead.
