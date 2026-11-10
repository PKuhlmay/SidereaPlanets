# RK-001 Cargo-Workspace und Gerüst

- Status: offen
- Abhängig von: —

## Ziel
Lauffähiges Rust-Gerüst, auf dem alle weiteren Tickets aufbauen.

## Akzeptanzkriterien
- Cargo-Workspace mit Crates: `core` (Bildverarbeitung, GUI-frei), `io` (Video-/Bildformate), `app` (GUI)
- Rust-Toolchain ist installiert (aktuell fehlt `cargo` im PATH) und per `rust-toolchain.toml` fixiert
- Befehle laufen: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- Dev-Commands in `CLAUDE.md` eingetragen
