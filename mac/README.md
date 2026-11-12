# SidereaPlanets für den Mac

Native Mac-App in **SwiftUI** auf dem gemeinsamen **Rust-Kern** (`crates/`), analog zu Siderea: Die Mac-App führt, die Windows-App
(`app/`, Tauri und Vue) zieht nach. Die Mac-App gibt es noch nicht; sie entsteht mit
[`tickets/MAC-001-mac-app-aufbauen.md`](tickets/MAC-001-mac-app-aufbauen.md) am Mac.

## Aufbau (geplant, MAC-001 legt ihn an)

```
mac/
  project.yml            xcodegen-Spezifikation (Ziel SidereaPlanets, MARKETING_VERSION)
  App/                   SwiftUI-Oberfläche: Schrittleiste, Panels, Bildbereich, Fenster, Menüs
  Packages/SidereaPlanetsKit/  Swift-Paket um die Rust-Schnittstelle (XCFramework und erzeugte Bindings)
  tickets/               MAC-NNN-kurzer-titel.md
  CHANGELOG.md           ab dem ersten Mac-Release
  README.md              diese Datei
crates/ffi/              Rust-Schnittstelle (UniFFI) auf den Rechenkern, GUI-frei (von MAC-001 angelegt)
```

Das einzige, was Mac und Windows teilen, ist der **Rechenkern**: `crates/core`, `crates/io` und die GUI-freie Sitzungslogik, die
MAC-001 aus den Tauri-Befehlen (`app/src-tauri/src/*_commands.rs`) in einen eigenen Crate zieht. Alles, was eine Oberfläche sonst
nachbauen müsste (Zustand, Verwerfen der Ergebnisse nach neuem Align, Thread-Pool, Abbrechen und Pausieren), gehört in den Kern, nicht
in die Oberfläche.

## Voraussetzungen am Mac

```bash
xcode-select --install                      # Command Line Tools; für die App Xcode aus dem App Store
brew install rustup xcodegen node pnpm      # Rust über rustup, xcodegen für das Projekt, node und pnpm für die Windows-App
rustup default stable
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

Prüfen, dass der Kern am Mac baut und die Tests laufen:

```bash
cargo test --workspace                      # ohne re-local/ überspringen sich die Referenztests
cargo clippy --workspace --all-targets
```

Die Referenzdaten (`re-local/`, nicht in Git) liegen nur auf dem Windows-Rechner; für Referenztests am Mac den Ordner kopieren und
`PLANETS_RE_LOCAL=<pfad>` setzen (`crates/io/tests/common/mod.rs`).

## Arbeitsweise

- **Commit-Regeln:** `[TYPE] MAC-NNN - Message` für Mac-Tickets, `[TYPE] RK-NNN - Message` für die Windows-App und den Kern. Der
  Commit-Hook `.githooks/commit-msg` verlangt bei jeder Änderung an `mac/` einen Eintrag in `docs/parity/WINDOWS_PARITY.md` (Windows
  muss es nachziehen) oder löscht einen Punkt aus `docs/parity/MAC_PARITY.md`; ohne sichtbare Wirkung trägt der Commit `[mac-only]`.
- **Hook aktivieren:** einmal je Klon `git config core.hooksPath .githooks`. Der Hook `post-commit` startet die Debug-App der
  **Windows**-App (`scripts/deploy-debug.sh`); am Mac steht er im Weg, wenn dort nicht `pnpm tauri dev` laufen soll — dann
  `DEPLOY_DEBUG=0` setzen (siehe Kopf des Skripts) oder den Hook lokal nicht verwenden.
- **Worktrees** sind am Mac erlaubt (auf Windows nicht): Branch und Ordner mit Ticketnummer, Squash-Merge, ein Commit pro Ticket.
- **Nachzug und Veröffentlichung:** `/nachzug` zeigt, was die jeweils andere App noch übernehmen muss; `/release mac|windows <version>`
  veröffentlicht. Tags `v<version>` für den Mac, `windows-v<version>` für Windows.
- **Aussehen:** `docs/design.md` und `docs/design/tokens.css` gelten auch für den Mac (modern, nicht der Delphi-Look); Mac-übliche
  Dinge (Menüleiste, ⌘-Kürzel, native Dateidialoge, Vollbild) folgen den Mac-Konventionen und stehen als bewusste Abweichung in
  `docs/design.md`.
- **Doku in der App:** Die Anwenderseiten liegen als Markdown unter `app/src/docs/pages/` und sind gemeinsam; die Mac-App zeigt
  dieselben Dateien in einem eigenen Hilfe-Fenster. Eine neue Seite ist eine neue Datei, für beide Apps.
- **Tests zuerst rot** (Rust: Kern und Schnittstelle, Swift: `swift test` am Paket), wie in den globalen Regeln.

## Bauen und veröffentlichen

Wird mit `MAC-001` ausgefüllt (xcodegen, `xcodebuild`, Bündel, Signierung und Notarisierung). Bis dahin gilt das Gerüst im Skill
`release`.

## Status

| Ticket | Thema | Stand |
|---|---|---|
| MAC-001 | Mac-App aufbauen (Klammer) | in Arbeit |
| MAC-002 | Kern am Mac und Deploy der Mac-App | offen |
| MAC-003 | Sitzungslogik im Kern (`crates/session`) | offen |
| MAC-004 | Rust-Schnittstelle (UniFFI), XCFramework, Swift-Paket | offen |
| MAC-005 | App-Gerüst, Fenster, Bildbereich, Open | offen |
| MAC-006 | Alignpoints, Align, Limit | offen |
| MAC-007 | Stack | offen |
| MAC-008 | Wavelets, Farbe, Functions | offen |
| MAC-009 | Save, Export, Projektdatei | offen |
| MAC-010 | Hilfe, Menüs, CPUs, Release | offen |
