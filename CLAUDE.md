# Siderea Planets

Planeten- und Mond-Stacking mit Wavelet-Schärfung für die Astrofotografie in **Rust**: Aufnahme laden, Alignpoints setzen,
ausrichten, die besten Frames wählen, stacken, schärfen und speichern. Ein eigenes Programm mit eigenem Rechenkern, eigener
Oberfläche und eigenem Design; es ist zunächst ein Prototyp, der prüft, ob das Vorhaben trägt.

**Name:** Die App heißt **Siderea Planets** (macOS, Windows, Linux), passend zur Suite mit Siderea. Einen abweichenden internen Namen
gibt es nicht: Repo und Ordner `SidereaPlanets`, Rust-Crates `planets-*`, Projektdateien `.sidereaplanets`,
Tickets `PLANETS-N` (früher `SP-NNN`, `MAC-NNN`).

**Ein bestimmter Programmname steht nirgends im Repo** (Philipp, 2026-11-27): weder in sichtbaren Texten noch in Code,
Kommentaren, Bezeichnern, Tests, `docs/`, Changelogs oder Commit-Messages, und er wird auch nicht umschrieben. Die Sperrliste
(ein Begriff oder regulärer Ausdruck je Zeile) liegt lokal in `re-local/forbidden-terms.txt`. Vor jedem Commit, der Texte
berührt, prüfen: `git grep -n -i -E -f re-local/forbidden-terms.txt -- . ':!tickets'` — Treffer sind nur erlaubt, wenn das englische Wort
im Bild-Sinn „unbearbeitetes Quellbild“ gemeint ist (z. B. in der Beschriftung einer Größenoption).

## Status

Es gibt **eine App**: Tauri 2 und Vue unter `app/`. Sie läuft den ganzen Ablauf im echten Fenster: Aufnahme laden, Alignpoints, Align,
Best Frames (Limit), Stack mit Derotation, Wavelets mit Do All und den Adjustments, Save image mit Crop, Flip and Rotate und Resize,
dazu Projektdatei, Save Aligned/Integrated, Schemes, CPU-Auswahl. Auf dem Mac hat sie Menüleiste, Cmd-Kürzel, Dateizuordnung und
folgt dem Erscheinungsbild des Systems (PLANETS-3); für Windows gibt es einen Installer (`scripts/publish-windows.sh --installer`,
Inno Setup wie bei Siderea). Die frühere native SwiftUI-App (Mac-Release 0.1.0) und die Rust-Schnittstelle `crates/ffi` sind mit
PLANETS-4 entfallen und stehen nur noch in der Git-Historie. Die Beschreibung von Ablauf, Oberfläche, Algorithmen und Formaten
steht in `docs/`; eigene Referenzbilder und Aufnahmen für die Regressionstests liegen lokal in `re-local/` (nicht in Git).
Das Design der Oberfläche ist in `docs/design.md` festgelegt (Tokens in
`docs/design/tokens.css`, App-Icon in `docs/design/icon/`) und für jede Arbeit an der Oberfläche verbindlich — **modern** und eigenständig.

## Tech Stack

- Rust stable (Edition 2024), Cargo-Workspace, `unsafe_code` verboten (Workspace-Lint)
- App-Shell: **Tauri 2** (Rust-Kern + Web-UI), Frontend **Vue 3 + TypeScript + Vite** (`pnpm`)
- Plattformen: **macOS, Windows und Linux** mit derselben Tauri-App `app/`. Der Rechenkern (`crates/`) bleibt plattformunabhängig:
  nichts Plattformspezifisches im Code, er muss auf allen dreien bauen und testen
- Bildverarbeitung: eigene Implementierung der Algorithmen, keine fremden Quellen
- Entscheidung und Begründung: `docs/gui-toolkit.md`

## Dev-Commands

```
cargo test --workspace                       # Rechenkern und Reader
cargo clippy --workspace --all-targets       # soll ohne Warnungen laufen
cargo fmt --all
cd app && pnpm install && pnpm tauri dev     # App mit Hot-Reload starten
cd app && pnpm build                         # Frontend bauen (vue-tsc + vite)
cd app && pnpm test                          # Frontend-Tests (vitest), z. B. Doku-Logik
scripts/publish-windows.sh --installer       # Release-Bau mit Inno-Setup-Installer, ZIP und SHA256SUMS (build/windows/dist/)
```

**Nicht** `target/debug/planets-app.exe` direkt starten: im Debug-Build erwartet die App den Vite-Server
(`localhost:1420`) und zeigt sonst „Seite nicht gefunden“. Immer `pnpm tauri dev` benutzen.

Rust liegt unter `~/.cargo/bin` (in Git Bash ggf. `export PATH="$HOME/.cargo/bin:$PATH"`).

## Deploy nach jedem Commit

Nach **jedem** Commit auf `main` läuft die Debug-App mit dem letzten Stand, damit Philipp ihn immer sieht. Das erledigt
der Git-Hook `.githooks/post-commit` (Skript `scripts/deploy-debug.sh`) von selbst, auch bei Commits anderer Agenten:
`pnpm install` und `pnpm build` in `app/`, dann `pnpm tauri dev` im Hintergrund (läuft schon eine Sitzung, nimmt sie die Änderung selbst
auf). Schlägt der Bau fehl, wird nicht deployt. Protokolle: `deploy-debug.log` und `tauri-dev.log` im Repo-Stamm, beide
nicht in Git. „Deploy“ heißt nur lokal starten, kein Release-Build und kein Veröffentlichen.

- Aktivieren, einmal je Checkout: `git config core.hooksPath .githooks`
- Nur der Haupt-Checkout auf `main` löst aus; Worktrees und andere Branches nicht. Der Squash-Merge eines Worktrees
  deployt also genau einmal, mit dem Ticket-Commit.
- Testläufe waren vorübergehend aus (Philipp, 2026-11-10) und laufen seit dem Abschluss der Tickets am 2026-11-12 wieder, von Hand
  und in den Agenten. Mit `DEPLOY_TESTS=1` läuft vor dem Bau auch `cargo test --workspace`; als Standard im Skript ist das noch nicht
  gesetzt.
- `DEPLOY_DEBUG=0` schaltet den Deploy aus.
- Von Hand deployen: `scripts/deploy-debug.sh --run`
- **Am Mac** läuft derselbe Hook und dasselbe Skript (bash 3.2, PATH mit Homebrew-rustup, das `rust-toolchain.toml` beachtet): die Tauri-Debug-App
  ist die Entwicklungsumgebung.
- Lokales Bündel am Mac: `cd app && pnpm tauri build` legt `.app` und `.dmg` unter `target/release/bundle/` ab (unsigniert);
  `tauri.conf.json` baut `all`, `tauri.windows.conf.json` beschränkt Windows auf `nsis`.

## Doku in der App

Zu jeder Funktion gehört Anwenderdoku, die **in der App über die UI erreichbar** ist (eigener Bereich/Hilfe-Ansicht, im
Design aus `docs/design.md`, gleiche Tokens und Komponenten). Sie wird mit der Funktion zusammen geschrieben und **in jedem
Commit mitgepflegt, wo die Änderung sie betrifft** (neue Funktion, geändertes Verhalten, neue Begriffe/Formate).
Die Doku beschreibt Bedienung und Ergebnis, nicht die technische Beschreibung der Algorithmen — die bleibt in `docs/`.
Die Seiten liegen als Markdown in `app/src/docs/pages/` (Kopf mit `title`, `group`, `order`), die Ansicht ist `app/src/docs/DocsView.vue`; Help in der Werkzeugleiste öffnet sie. Eine neue Seite ist nur eine neue Datei.

## Eine App für drei Plattformen

Die Tauri-App in `app/` ist die einzige Oberfläche und läuft auf macOS, Windows und Linux. Entwickelt wird am Mac; die Builds für
Windows und Linux kommen aus der CI (PLANETS-11 Bau und Tests, PLANETS-12 Releases). Es gibt keine zweite Oberfläche mehr, die
nachgezogen werden müsste, und keine Paritätslisten.

- Plattformunterschiede (Menüleiste, Cmd statt Strg, Dateizuordnung) liegen in `app/src-tauri/tauri.<plattform>.conf.json`, in
  `app/src-tauri/src/menu.rs` und in `app/src/logic/platform.ts`; was der Rechenkern kann, gilt überall gleich.
- Bündel-Kennung (`identifier`): `de.kuhlmay.siderea.planets`, dieselbe wie beim Mac-Release 0.1.0 (Signatur und Notarisierung hängen
  daran). Signieren und Notarisieren für macOS: `docs/release.md`.
- Tags: `v<version>` für den früheren Mac-Release, `windows-v<version>` für Windows; `/siderea-release` wird mit PLANETS-12
  neu geschrieben. Pushen nur nach Philipps OK.
- Tickets liegen jetzt in **Jira**, Projekt **PLANETS** auf treupo.atlassian.net (Cloud-ID und Zugang siehe globale Anweisungen);
  Commits `[TYPE] PLANETS-N - Message in English`.
- Worktrees: **nicht unter Windows** (Speicher, langsames Löschen), am Mac erlaubt.

## Verzeichnisstruktur

```
crates/core/ Bildtyp und Algorithmen: Ausrichtung, Stacking, Wavelets, Farbe — GUI-frei, auf allen Plattformen gleich
crates/io/   Leser und Schreiber für Aufnahmen und Ergebnisse (SER, AVI, Bildserien, FITS, Export, Projektdatei)
crates/session/ Sitzungslogik (`Session`): offene Aufnahme, Ergebnisse je Schritt, Verwerfen, Thread-Pool der CPU-Auswahl, Abbrechen und
             Pausieren, Fortschritt über den Trait `ProgressSink` — GUI-frei; die Tauri-Shell ist eine dünne Hülle
app/         Die App (Tauri): `app/src` = Vue-Frontend (hier entsteht das Design), `app/src-tauri` = Rust-Shell mit Commands
scripts/   Skripte, z. B. `deploy-debug.sh` für den Deploy-Hook (`.githooks/post-commit`), `publish-windows.sh`, `make-icons.sh`
tickets/   Alte Aufgaben als Markdown-Dateien (SP-NNN-kurzer-titel.md); neue Tickets in Jira (PLANETS)
docs/      Technische Beschreibung (Ablauf, Oberfläche, Algorithmen, Formate, Design), dazu `docs/release.md`
           (Signieren und Notarisieren für macOS)
re-local/  Lokal, nicht in Git: Beispielaufnahmen, eigene Referenzbilder, Screenshots, interne Notizen
.claude/   Hooks, Skills und Einstellungen aus `npx github:PKuhlmay/claude-kit install` (nicht von Hand ändern). Projekteigen und nicht aus
           dem Kit: der Skill `siderea-release` unter `.claude/skills/`
```

## Tickets

- Neue Tickets in Jira (PLANETS). Die älteren liegen im Repo unter `tickets/`, **ohne Jira**. Dateiname `SP-NNN-kurzer-titel.md`, fortlaufend nummeriert.
- Kopf: Status (`offen` · `in Arbeit` · `erledigt`), Abhängigkeiten, dann Ziel und Akzeptanzkriterien.
- Akzeptanzkriterien sind **Checkboxen** (`- [ ]` offen, `- [x]` erledigt). Sobald ein Kriterium wirklich erfüllt ist
  (Test grün, Verhalten geprüft), wird es im selben Commit abgehakt. Status `erledigt` erst, wenn alle abgehakt sind;
  bewusst gestrichene Kriterien bleiben offen und bekommen eine Begründung im Text.
- Status-Änderung direkt in der Datei und im selben Commit wie die Umsetzung.
- Commits: `[TYPE] SP-NNN - Message in English` (Types wie in den globalen Regeln).

## Eigene Implementierung

- Algorithmen, Oberfläche, Texte, Icons und Assets sind eigene Arbeit. Fremde Binaries, Ressourcen, Icons oder Texte kommen nicht ins
  Repo; Aufnahmen und Referenzdaten für Tests liegen in `re-local/` (gitignored).
- Was in `docs/` steht, beschreibt, was Siderea Planets tut und warum. Offene Annahmen stehen dort als Entwurfsentscheidung, nicht
  als Vergleich mit anderer Software.

## Konventionen

- Bildverarbeitung sichern Regressionstests gegen **eigene Referenzbilder**: gleiche Eingabe, Ergebnis vergleichen
  (Differenzbild, PSNR mit festem Schwellenwert). Die Daten liegen in `re-local/` (nicht in Git, die Tests überspringen sich ohne sie);
  die Schwellen stehen im Test und in `docs/`.
- Rechenkern (Stacking, Wavelets) ist GUI-frei in eigenen Crates und per Unit-Test prüfbar.
- Performance zählt: Videos mit tausenden Frames, deshalb Streaming statt alles im RAM, Parallelität via `rayon`.
- `clippy` ohne Warnungen, `cargo fmt`; kein `unsafe` ohne Begründung im Kommentar.
