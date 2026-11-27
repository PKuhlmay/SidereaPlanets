# SP-001 Cargo-Workspace und Gerüst

- Status: erledigt
- Abhängig von: —

## Ziel
Lauffähiges Rust-Gerüst, auf dem alle weiteren Tickets aufbauen.

## Akzeptanzkriterien
- [x] Cargo-Workspace mit Crates: `core` (Bildverarbeitung, GUI-frei), `io` (Video-/Bildformate), `app/src-tauri` (Tauri-Shell)
- [x] Rust-Toolchain ist installiert und per `rust-toolchain.toml` auf eine Version fixiert (1.99.1)
- [x] Befehle laufen: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [x] Dev-Commands in `CLAUDE.md` eingetragen
