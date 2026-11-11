//! Where the tests find `re-local/` (not in Git): `$PLANETS_RE_LOCAL`, the repository itself, or - in a
//! worktree below `.claude/worktrees/<name>/` - the main checkout, five levels above `crates/io`.

use std::path::PathBuf;

/// The first existing `re-local/` folder; the repository's own path if there is none (callers then skip).
pub fn re_local() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        std::env::var_os("PLANETS_RE_LOCAL").map(PathBuf::from),
        Some(manifest.join("../../re-local")),
        Some(manifest.join("../../../../../re-local")),
    ];
    let found = candidates.into_iter().flatten().find(|p| p.is_dir());
    found.unwrap_or_else(|| manifest.join("../../re-local"))
}
