# Siderea Planets

Prototyp: Planeten-/Mond-Stacking und Wavelet-Schärfung für die Astrofotografie in **Rust**, nach dem Vorbild eines älteren
Windows-Programms (Delphi, kostenlos, seit 2011 nicht mehr gepflegt), hier und in `docs/` das **Referenzprogramm** genannt.
Ziel ist ein möglichst genauer Nachbau in **Bedienung und Ergebnis** — gleicher Workflow, gleiche Begriffe,
vergleichbare Resultate. Es ist zunächst nur ein Prototyp, um zu prüfen, ob das Vorhaben überhaupt aufgeht.

**Name:** Die App heißt **Siderea Planets** (Mac und Windows), passend zur Suite mit Siderea. Einen abweichenden internen Namen
gibt es nicht: Repo und Ordner `SidereaPlanets`, Rust-Crates `planets-*`, Swift-Module (`SidereaPlanetsKit`,
`SidereaPlanetsBindings`, `SidereaPlanetsFFI`), Xcode-Ziel und -Schema `SidereaPlanets`, Projektdateien `.sidereaplanets`,
Tickets `SP-NNN`.

**Der Name des Referenzprogramms steht nirgends im Repo** (Philipp, 2026-11-27): weder in sichtbaren Texten noch in Code,
Kommentaren, Bezeichnern, Tests, `docs/`, Tickets oder Commit-Messages. Wo es gemeint ist, heißt es „das Referenzprogramm“
(englisch „the reference program“). In Texten, die Nutzer sehen (Oberfläche und Tooltips beider Apps, Dateidialoge, `.rwv` heißt
„Wavelet scheme“, Statusmeldungen, Datei- und Projektköpfe, About, Anwenderseiten unter `app/src/docs/pages/`, Changelogs), wird
es auch nicht umschrieben erwähnt, auch nicht als „das Original“: Siderea Planets ist ein eigenes Programm. Vergleiche gehören in
die interne Doku (`docs/`, z. B. `docs/offene-fragen.md`).

## Status

Die **Windows-App** (Tauri und Vue unter `app/`) läuft den ganzen Ablauf im echten Fenster: Aufnahme laden, Alignpoints, Align,
Limit, Stack, Wavelets mit Do All, Save image, dazu Projektdatei, Save Aligned/Integrated, Schemes, CPU-Auswahl und ein Installer
(`scripts/publish-windows.sh --installer`, Inno Setup wie bei Siderea). Stand und offene Kriterien stehen in `tickets/`. Das Referenzprogramm ist
auf dem Windows-Rechner installiert; die Spezifikation daraus steht in `docs/` (SP-003), Referenzergebnisse
in `re-local/reference/` (beschrieben in `docs/testdaten.md`).
Das Design der Oberfläche ist in `docs/design.md` festgelegt (Tokens, klickbarer Entwurf und Soll-Bilder in
`docs/design/`) und für jede Arbeit an der Oberfläche verbindlich — **modern**, nicht der Delphi-Look.
Die **Mac-App** (native SwiftUI auf dem Rust-Kern, `mac/`) entsteht als Nächstes am Mac und führt danach; die Windows-App zieht nach
(siehe „Mac und Windows“).

## Tech Stack

- Rust stable (Edition 2024), Cargo-Workspace, `unsafe_code` verboten (Workspace-Lint)
- App-Shell: **Tauri 2** (Rust-Kern + Web-UI), Frontend **Vue 3 + TypeScript + Vite** (`pnpm`)
- Plattformen: **Windows** (Tauri-App `app/`, jetzt) und **macOS** (native SwiftUI-App `mac/` auf dem Rust-Kern über eine
  Schnittstelle `crates/ffi`, geplant mit `mac/tickets/MAC-001-…`). Der Rechenkern (`crates/`) bleibt plattformunabhängig:
  nichts Windows-Spezifisches im Code, er muss am Mac bauen und testen
- Bildverarbeitung: eigene Implementierung der Algorithmen, keine Delphi-Quellen
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

- Aktivieren, einmal je Klon: `git config core.hooksPath .githooks`
- Nur der Haupt-Checkout auf `main` löst aus; Worktrees und andere Branches nicht. Der Squash-Merge eines Worktrees
  deployt also genau einmal, mit dem Ticket-Commit.
- Testläufe waren vorübergehend aus (Philipp, 2026-11-10) und laufen seit dem Abschluss der Tickets am 2026-11-12 wieder, von Hand
  und in den Agenten. Mit `DEPLOY_TESTS=1` läuft vor dem Bau auch `cargo test --workspace`; als Standard im Skript ist das noch nicht
  gesetzt.
- `DEPLOY_DEBUG=0` schaltet den Deploy aus.
- Von Hand deployen: `scripts/deploy-debug.sh --run`
- **Am Mac** läuft derselbe Hook und dasselbe Skript (bash 3.2, PATH mit Homebrew-rustup, das `rust-toolchain.toml` beachtet): die Tauri-Debug-App
  ist die Entwicklungsumgebung. `scripts/deploy-mac.sh` (SwiftUI-App) ruft der Hook nicht mehr auf; von Hand geht es noch (`--run`).
- Lokales Bündel am Mac: `cd app && pnpm tauri build` legt `.app` und `.dmg` unter `target/release/bundle/` ab (unsigniert);
  `tauri.conf.json` baut `all`, `tauri.windows.conf.json` beschränkt Windows auf `nsis`.

## Doku in der App

Zu jeder Funktion gehört Anwenderdoku, die **in der App über die UI erreichbar** ist (eigener Bereich/Hilfe-Ansicht, im
Design aus `docs/design.md`, gleiche Tokens und Komponenten). Sie wird mit der Funktion zusammen geschrieben und **in jedem
Commit mitgepflegt, wo die Änderung sie betrifft** (neue Funktion, geändertes Verhalten, neue Begriffe/Formate).
Die Doku beschreibt Bedienung und Ergebnis, nicht die Reverse-Engineering-Notizen — die bleiben in `docs/`.
Die Seiten liegen als Markdown in `app/src/docs/pages/` (Kopf mit `title`, `group`, `order`), die Ansicht ist `app/src/docs/DocsView.vue`; Help in der Werkzeugleiste öffnet sie. Eine neue Seite ist nur eine neue Datei.

## Mac und Windows

Zwei Oberflächen auf einem gemeinsamen Rechenkern, wie bei Siderea: Die **Mac-App** (`mac/`, SwiftUI) führt, die **Windows-App**
(`app/`, Tauri und Vue) zieht nach. Beide liegen auf `main`; was nur die eine kann, steht als Checkbox in der Liste der anderen:

- `docs/parity/WINDOWS_PARITY.md`: was der Mac bekommen hat und Windows noch übernehmen muss.
- `docs/parity/MAC_PARITY.md`: was Windows bekommen hat und der Mac noch übernehmen muss. Bis `MAC-001` fertig ist, steht dort der
  Verweis auf den ganzen Funktionsumfang der Windows-App (Spezifikationen und Tickets).
- `docs/parity/Handprobe.md`: was nur ein Mensch von Hand prüfen kann; `[ ]` offen, `[x]` klappt, `[!]` klappt nicht.

Jede Änderung, die verändert, was Nutzer sehen oder tun (Schaltflächen, Texte, Tastenkürzel, Schritte, Werte), hinterlässt **im selben
Commit** eine Checkbox für die andere App: eine Mac-Änderung in `WINDOWS_PARITY.md`, eine Windows-Änderung in `MAC_PARITY.md`. Ein
Punkt je Verhalten, mit den Werten, den Dateien und den Texten; der Commit, der nachzieht, löscht ihn (die Listen enthalten nur offene
Punkte, die Git-Historie zeigt das Erledigte). Gemeinsames braucht keinen Eintrag: Rechenkern (`crates/`), Anwenderseiten unter
`app/src/docs/pages/` (beide Apps zeigen dieselben Markdown-Dateien), Tests und Tickets.

`.githooks/commit-msg` erzwingt das: Ein Commit, der `mac/` oder die Windows-Oberfläche (`app/src`, `app/src-tauri/src`) ändert, ohne
eine der beiden Listen anzufassen, wird abgelehnt, außer die Nachricht trägt `[mac-only]` bzw. `[windows-only]` (Glue-Code, Installer,
Umbau ohne sichtbare Wirkung). Keine Liste darf gelöscht oder umbenannt werden. Der Hook ist über `core.hooksPath = .githooks`
aktiv; einmal je Klon `git config core.hooksPath .githooks`.

- `/nachzug [mac]` (`scripts/parity-status.sh [liste]`) zeigt die offenen Punkte je Version und schlägt die nächste Stufe vor.
- `/siderea-release mac|windows <version>` setzt die Version, prüft, baut (Windows: NSIS-Installer), committet und taggt. Tags: `v<version>`
  für den Mac, `windows-v<version>` für Windows. Haupt- und Nebennummer heißen Funktionsumfang, Patch-Nummern zählt jede Plattform
  für sich. Pushen nur nach Philipps OK.
- Der Rechenkern ist die einzige Kopplung. Was eine Oberfläche sonst nachbauen müsste (Sitzungszustand, Verwerfen der Ergebnisse,
  Thread-Pool, Abbrechen und Pausieren), gehört in den Kern (`crates/`: `crates/session`, ab MAC-004 `crates/ffi`), nicht in die
  Oberfläche. Eine Änderung an einer Kern-Schnittstelle, die die andere App benutzt, bekommt einen eigenen Commit mit Hinweis.
- Mac-Tickets: `mac/tickets/MAC-NNN-kurzer-titel.md`, Commits `[TYPE] MAC-NNN - Message`. Das Startticket ist
  `mac/tickets/MAC-001-mac-app-aufbauen.md`; Aufbau, Voraussetzungen und Arbeitsweise am Mac stehen in `mac/README.md`.
- Worktrees: **nicht unter Windows** (Speicher, langsames Löschen), am Mac erlaubt.

## Verzeichnisstruktur

```
crates/core/ Bildtyp und Algorithmen: Ausrichtung, Stacking, Wavelets, Farbe — GUI-frei, auf allen Plattformen gleich
crates/io/   Leser und Schreiber für Aufnahmen und Ergebnisse (SER, AVI, Bildserien, FITS, Export, Projektdatei)
crates/session/ Sitzungslogik (`Session`): offene Aufnahme, Ergebnisse je Schritt, Verwerfen, Thread-Pool der CPU-Auswahl, Abbrechen und
             Pausieren, Fortschritt über den Trait `ProgressSink` — GUI-frei; die Tauri-Shell und (MAC-004) `crates/ffi` sind dünne Hüllen
app/         Windows-App (Tauri): `app/src` = Vue-Frontend (hier entsteht das Design), `app/src-tauri` = Rust-Shell mit Commands
mac/         Mac-App (SwiftUI) und ihre Tickets (`mac/tickets/MAC-NNN-…`); noch ohne Code, siehe `mac/README.md`
scripts/   Skripte, z. B. `deploy-debug.sh` für den Deploy-Hook (`.githooks/post-commit`), `parity-status.sh` für `/nachzug`
tickets/   Aufgaben der Windows-App und des Kerns als Markdown-Dateien (SP-NNN-kurzer-titel.md), kein Jira
docs/      Spezifikation, die beim Reverse Engineering entsteht (Workflow, UI, Algorithmen, Formate), `docs/parity/` mit den
           Nachzug-Listen und der Handprobe
re-local/  Lokal, nicht in Git: Original-Binaries, Beispieldaten, Screenshots, Mitschnitte
.claude/   Hooks, Skills und Einstellungen aus `npx github:PKuhlmay/claude-kit install` (nicht von Hand ändern). Projekteigen und nicht aus
           dem Kit: die Skills `nachzug` und `siderea-release` unter `.claude/skills/`
```

## Tickets

- Nur im Repo unter `tickets/`, **kein Jira**. Dateiname `SP-NNN-kurzer-titel.md`, fortlaufend nummeriert.
- Kopf: Status (`offen` · `in Arbeit` · `erledigt`), Abhängigkeiten, dann Ziel und Akzeptanzkriterien.
- Akzeptanzkriterien sind **Checkboxen** (`- [ ]` offen, `- [x]` erledigt). Sobald ein Kriterium wirklich erfüllt ist
  (Test grün, Verhalten geprüft), wird es im selben Commit abgehakt. Status `erledigt` erst, wenn alle abgehakt sind;
  bewusst gestrichene Kriterien bleiben offen und bekommen eine Begründung im Text.
- Status-Änderung direkt in der Datei und im selben Commit wie die Umsetzung.
- Commits: `[TYPE] SP-NNN - Message in English` (Types wie in den globalen Regeln); Mac-Tickets `[TYPE] MAC-NNN - …`.
- Mac-Tickets liegen getrennt unter `mac/tickets/` (gleiche Regeln für Kopf, Kriterien und Status).

## Reverse Engineering

- Quelle ist das **beobachtbare Verhalten** des installierten Originals: Oberfläche, Ein-/Ausgaben,
  Dateiformate, Vergleich von Ergebnisbildern. Erkenntnisse landen als Beschreibung in `docs/`.
- Binaries, Ressourcen, Icons und Texte des Originals werden **nicht** ins Repo übernommen oder kopiert
  (`re-local/` ist gitignored). Der Nachbau bekommt eigene Implementierung und eigene Assets.
- Algorithmen, die nur geraten sind, in `docs/` als **Vermutung** markieren, bis ein Vergleich mit dem
  Original sie bestätigt.

## Konventionen

- Bildverarbeitung prüfen wir **gegen das Original**: gleiche Eingabe, Ergebnis vergleichen
  (Differenzbild, PSNR). Referenzfälle liegen in `docs/` beschrieben, Daten in `re-local/`.
- Rechenkern (Stacking, Wavelets) ist GUI-frei in eigenen Crates und per Unit-Test prüfbar.
- Performance zählt: Videos mit tausenden Frames, deshalb Streaming statt alles im RAM, Parallelität via `rayon`.
- `clippy` ohne Warnungen, `cargo fmt`; kein `unsafe` ohne Begründung im Kommentar.
