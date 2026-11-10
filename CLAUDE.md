# SidereaPlanets

Prototyp: Nachbau von **the reference program** (Planeten-/Mond-Stacking und Wavelet-Schärfung für die Astrofotografie)
in **Rust**. Das Original (Delphi, Windows) ist kostenlos, wird aber seit 2011 nicht mehr gepflegt.
Ziel ist ein möglichst genauer Klon in **Bedienung und Ergebnis** — gleicher Workflow, gleiche Begriffe,
vergleichbare Resultate. Es ist zunächst nur ein Prototyp, um zu prüfen, ob das Vorhaben überhaupt aufgeht.

## Status

Gerüst steht (RK-001, RK-002): Workspace, SER-Reader, Tauri-Shell mit Platzhalter-UI. the reference program ist unter
`C:\Program Files (x86)\the reference program` installiert; daraus entsteht die Spezifikation (RK-003).
Das Design der Oberfläche wird getrennt von anderen Agenten gebaut (RK-010) — **modern**, nicht der Delphi-Look.

## Tech Stack

- Rust stable (Edition 2024), Cargo-Workspace, `unsafe_code` verboten (Workspace-Lint)
- App-Shell: **Tauri 2** (Rust-Kern + Web-UI), Frontend **Vue 3 + TypeScript + Vite** (`pnpm`)
- Ziel-Plattformen: Windows, macOS, Linux — nichts Windows-Spezifisches im Code; Mehrplattform-Build später per CI
- Bildverarbeitung: eigene Implementierung der Algorithmen, keine Delphi-Quellen
- Entscheidung und Begründung: `docs/gui-toolkit.md`

## Dev-Commands

```
cargo test --workspace                       # Rechenkern und Reader
cargo clippy --workspace --all-targets       # soll ohne Warnungen laufen
cargo fmt --all
cd app && pnpm install && pnpm tauri dev     # App mit Hot-Reload starten
cd app && pnpm build                         # Frontend bauen (vue-tsc + vite)
cd app && pnpm tauri build                   # Release-Installer bauen
```

**Nicht** `target/debug/planets-app.exe` direkt starten: im Debug-Build erwartet die App den Vite-Server
(`localhost:1420`) und zeigt sonst „Seite nicht gefunden“. Immer `pnpm tauri dev` benutzen.

Rust liegt unter `~/.cargo/bin` (in Git Bash ggf. `export PATH="$HOME/.cargo/bin:$PATH"`).

## Deploy nach jedem Commit

Nach **jedem** Commit stellt Claude die App bereit, damit Philipp immer den letzten Stand sieht: erst
`cargo test --workspace` und `pnpm build` (in `app/`), dann läuft `pnpm tauri dev` im Hintergrund mit dem committeten Stand
(läuft schon eine Sitzung, nimmt sie Änderungen selbst auf; sonst neu starten). „Deploy“ heißt hier nur lokal
starten, kein Release-Build und kein Veröffentlichen.

## Verzeichnisstruktur

```
crates/core/ Bildtyp und (später) Algorithmen: Ausrichtung, Stacking, Wavelets — GUI-frei
crates/io/   Leser für Aufnahmen (SER und unkomprimiertes AVI fertig; Bildserien, FITS, komprimierte AVI-Codecs folgen)
app/         Tauri-App: `app/src` = Vue-Frontend (hier entsteht das Design), `app/src-tauri` = Rust-Shell mit Commands
tickets/   Aufgaben als Markdown-Dateien (RK-NNN-kurzer-titel.md), kein Jira
docs/      Spezifikation, die beim Reverse Engineering entsteht (Workflow, UI, Algorithmen, Formate)
re-local/  Lokal, nicht in Git: Original-Binaries, Beispieldaten, Screenshots, Mitschnitte
.claude/   Hooks, Skills und Einstellungen aus `npx github:PKuhlmay/claude-kit install` (nicht von Hand ändern)
```

## Tickets

- Nur im Repo unter `tickets/`, **kein Jira**. Dateiname `RK-NNN-kurzer-titel.md`, fortlaufend nummeriert.
- Kopf: Status (`offen` · `in Arbeit` · `erledigt`), Abhängigkeiten, dann Ziel und Akzeptanzkriterien.
- Akzeptanzkriterien sind **Checkboxen** (`- [ ]` offen, `- [x]` erledigt). Sobald ein Kriterium wirklich erfüllt ist
  (Test grün, Verhalten geprüft), wird es im selben Commit abgehakt. Status `erledigt` erst, wenn alle abgehakt sind;
  bewusst gestrichene Kriterien bleiben offen und bekommen eine Begründung im Text.
- Status-Änderung direkt in der Datei und im selben Commit wie die Umsetzung.
- Commits: `[TYPE] RK-NNN - Message in English` (Types wie in den globalen Regeln).

## Reverse Engineering

- Quelle ist das **beobachtbare Verhalten** des installierten Originals: Oberfläche, Ein-/Ausgaben,
  Dateiformate, Vergleich von Ergebnisbildern. Erkenntnisse landen als Beschreibung in `docs/`.
- Original-Binaries, Ressourcen, Icons und Texte werden **nicht** ins Repo übernommen oder kopiert
  (`re-local/` ist gitignored). Der Klon bekommt eigene Implementierung und eigene Assets.
- Algorithmen, die nur geraten sind, in `docs/` als **Vermutung** markieren, bis ein Vergleich mit dem
  Original sie bestätigt.

## Konventionen

- Bildverarbeitung prüfen wir **gegen das Original**: gleiche Eingabe, Ergebnis vergleichen
  (Differenzbild, PSNR). Referenzfälle liegen in `docs/` beschrieben, Daten in `re-local/`.
- Rechenkern (Stacking, Wavelets) ist GUI-frei in eigenen Crates und per Unit-Test prüfbar.
- Performance zählt: Videos mit tausenden Frames, deshalb Streaming statt alles im RAM, Parallelität via `rayon`.
- `clippy` ohne Warnungen, `cargo fmt`; kein `unsafe` ohne Begründung im Kommentar.
