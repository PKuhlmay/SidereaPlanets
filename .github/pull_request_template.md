## What and why

<!-- What does this change, and why? Link the issue if there is one. -->

## Checklist

- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass
- [ ] `cd app && pnpm test` passes
- [ ] `cargo deny check` and `cd app && pnpm check:licenses` pass; new dependencies are GPL-compatible
- [ ] New or changed behaviour has a test that fails without the change
- [ ] The user documentation in `app/src/docs/pages/` is updated where the change affects it
- [ ] Commit messages follow `[TYPE] PLANETS-N - Message` (external contributors may omit the ticket)
