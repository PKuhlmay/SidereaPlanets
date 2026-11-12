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

Das einzige, was Mac und Windows teilen, ist der **Rechenkern**: `crates/core`, `crates/io` und die GUI-freie Sitzungslogik in
`crates/session` (aus den Tauri-Befehlen gezogen, MAC-003; die Tauri-Shell ist nur noch eine dünne Hülle darüber). Alles, was eine
Oberfläche sonst nachbauen müsste (Zustand, Verwerfen der Ergebnisse nach neuem Align, Thread-Pool, Abbrechen und Pausieren), gehört in
den Kern, nicht in die Oberfläche. Die Mac-Schnittstelle `crates/ffi` (MAC-004) legt sich über dieselbe `Session`.

## Voraussetzungen am Mac

```bash
xcode-select --install                      # Command Line Tools; für die App Xcode aus dem App Store
brew install rustup xcodegen node pnpm      # Rust über rustup, xcodegen für das Projekt, node und pnpm für die Windows-App
rustup default stable
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

`rustup` aus Homebrew ist **keg-only**: `cargo` und `rustc` liegen nicht im PATH, bis `/opt/homebrew/opt/rustup/bin` dort steht
(in `~/.zshrc`). Das Repo pinnt die Toolchain in `rust-toolchain.toml`, rustup lädt sie beim ersten Aufruf nach.

```bash
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
```

Prüfen, dass der Kern am Mac baut und die Tests laufen:

```bash
cargo test --workspace                      # ohne re-local/reference/ überspringen sich die Referenztests (mit Hinweis)
cargo clippy --workspace --all-targets
```

Am Mac liegen unter `re-local/testdata/astropix/` nur die Eingabevideos (JUPITER, CLAVIUS, SUNSPOTS von astropix.com, siehe
`docs/testdaten.md`); damit laufen die Tests, die nur die Videos brauchen (Ausrichtung, ganze Kette, Dateileser). Die Ergebnisse des
Originals (`re-local/reference/`, nicht in Git) liegen nur auf dem Windows-Rechner; die Tests dagegen (Stack, Wavelets, Farbe)
überspringen sich mit einem Hinweis. Für sie den Ordner kopieren und `PLANETS_RE_LOCAL=<pfad>` setzen
(`crates/io/tests/common/mod.rs`).

## Arbeitsweise

- **Commit-Regeln:** `[TYPE] MAC-NNN - Message` für Mac-Tickets, `[TYPE] RK-NNN - Message` für die Windows-App und den Kern. Der
  Commit-Hook `.githooks/commit-msg` verlangt bei jeder Änderung an `mac/` einen Eintrag in `docs/parity/WINDOWS_PARITY.md` (Windows
  muss es nachziehen) oder löscht einen Punkt aus `docs/parity/MAC_PARITY.md`; ohne sichtbare Wirkung trägt der Commit `[mac-only]`.
- **Hook aktivieren:** einmal je Klon `git config core.hooksPath .githooks`. Die Dateien in `.githooks/` und `scripts/*.sh` müssen
  ausführbar sein (`git update-index --chmod=+x`), sonst ignoriert Git sie stillschweigend. Der Hook `post-commit` ruft am Mac
  `scripts/deploy-mac.sh`, sonst `scripts/deploy-debug.sh` (Abschnitt „Deploy am Mac“).
- **Worktrees** sind am Mac erlaubt (auf Windows nicht): Branch und Ordner mit Ticketnummer, Squash-Merge, ein Commit pro Ticket.
- **Nachzug und Veröffentlichung:** `/nachzug` zeigt, was die jeweils andere App noch übernehmen muss; `/release mac|windows <version>`
  veröffentlicht. Tags `v<version>` für den Mac, `windows-v<version>` für Windows.
- **Aussehen:** `docs/design.md` und `docs/design/tokens.css` gelten auch für den Mac (modern, nicht der Delphi-Look); Mac-übliche
  Dinge (Menüleiste, ⌘-Kürzel, native Dateidialoge, Vollbild) folgen den Mac-Konventionen und stehen als bewusste Abweichung in
  `docs/design.md`.
- **Doku in der App:** Die Anwenderseiten liegen als Markdown unter `app/src/docs/pages/` und sind gemeinsam; die Mac-App zeigt
  dieselben Dateien in einem eigenen Hilfe-Fenster. Eine neue Seite ist eine neue Datei, für beide Apps.
- **Tests zuerst rot** (Rust: Kern und Schnittstelle, Swift: `swift test` am Paket), wie in den globalen Regeln.

## Deploy am Mac

Nach jedem Commit auf `main` baut `scripts/deploy-mac.sh` die Mac-App und startet sie neu, damit der letzte Stand immer läuft. Der
Hook startet es im Hintergrund; nur der Haupt-Checkout auf `main` löst aus (Worktrees und andere Branches nicht).

- Ablauf: `scripts/build-ffi.sh` (wenn es das Skript gibt, ab MAC-004), `xcodegen generate` in `mac/`, dann `xcodebuild` (Debug,
  arm64) mit Derived Data unter `build/mac/` (nicht in Git). **Nur wenn der Bau klappt**, wird die laufende App beendet und die
  neue gestartet; schlägt er fehl, läuft die alte weiter und der Fehler steht im Protokoll.
- Solange es `mac/project.yml` nicht gibt, meldet das Skript „mac/project.yml missing, nothing to deploy“ und endet ohne Fehler.
- Protokoll: `deploy-mac.log` im Repo-Stamm (nicht in Git).
- Von Hand: `scripts/deploy-mac.sh --run`. Ausschalten: `DEPLOY_DEBUG=0`. Mit `DEPLOY_TESTS=1` läuft vorher
  `cargo test --workspace` (sonst aus, wie bei der Windows-App).
- Läuft gerade ein Deploy, merkt sich ein weiterer Commit das und der Deploy geht danach noch einmal durch.

## Bauen und veröffentlichen

Wird mit `MAC-001` ausgefüllt (xcodegen, `xcodebuild`, Bündel, Signierung und Notarisierung). Bis dahin gilt das Gerüst im Skill
`release`.

## Status

| Ticket | Thema | Stand |
|---|---|---|
| MAC-001 | Mac-App aufbauen (Klammer) | in Arbeit |
| MAC-002 | Kern am Mac und Deploy der Mac-App | erledigt |
| MAC-003 | Sitzungslogik im Kern (`crates/session`) | erledigt |
| MAC-004 | Rust-Schnittstelle (UniFFI), XCFramework, Swift-Paket | offen |
| MAC-005 | App-Gerüst, Fenster, Bildbereich, Open | offen |
| MAC-006 | Alignpoints, Align, Limit | offen |
| MAC-007 | Stack | offen |
| MAC-008 | Wavelets, Farbe, Functions | offen |
| MAC-009 | Save, Export, Projektdatei | offen |
| MAC-010 | Hilfe, Menüs, CPUs, Release | offen |
