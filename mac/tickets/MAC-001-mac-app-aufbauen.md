# MAC-001 Mac-App aufbauen

- Status: in Arbeit
- Abhängig von: RK-005 bis RK-010, RK-012, RK-014 (Stand der Windows-App vom 2026-11-12)
- Rechner: Mac (Apple Silicon, Xcode, `mac/README.md` Abschnitt „Voraussetzungen am Mac“)

## Ziel

Die **Windows-App von SidereaPlanets** (Tauri und Vue unter `app/`, Release-Build vom 2026-11-12) wird am Mac als **native
SwiftUI-App** nachgebaut: gleicher Ablauf, gleiche Begriffe, gleiche Ergebnisse, gleiches modernes Aussehen nach `docs/design.md`.
Danach führt der Mac: neue Funktionen entstehen hier, die Windows-App zieht über `docs/parity/WINDOWS_PARITY.md` nach (wie bei
Siderea).

Gemeinsam bleibt nur der Rechenkern in Rust. Die Mac-App ruft ihn über eine Schnittstelle auf; sie rechnet nichts selbst.

## Architektur

- **Rust-Schnittstelle** `crates/ffi` (neu, GUI-frei), mit **UniFFI** (oder dem begründeten Gleichwertigen) erzeugte Swift-Bindings.
  Der Crate baut als statische Bibliothek für `aarch64-apple-darwin` und `x86_64-apple-darwin` und wird zu einem **XCFramework**
  gebündelt.
- **Sitzungslogik in den Kern ziehen:** Was heute in den Tauri-Befehlen steckt (`app/src-tauri/src/*_commands.rs`: Zustand je Schritt,
  Verwerfen der Ergebnisse nach neuem Set Alignpoints/Align (`discard_results`), Thread-Pool der CPU-Auswahl (`compute.rs`),
  Abbrechen und Pausieren, Fortschritt) wird in einen GUI-freien Crate (`crates/session`) verschoben, den die Tauri-Shell **und** die
  Schnittstelle benutzen. Die Windows-App muss danach unverändert laufen (Tests und Fensterlauf wie bei RK-014).
- **Swift-Paket** `mac/Packages/SidereaPlanetsKit`: dünne Swift-Schicht um die erzeugten Bindings (typisierte Aufrufe, `async`,
  Fortschritts-Streams, Abbruch), mit `swift test`.
- **App** `mac/App` (SwiftUI, macOS 14 oder neuer), Projekt per **xcodegen** aus `mac/project.yml`.
- **Bilder:** Bildanzeige über `CGImage`/Metal-fähige Ansicht mit Zoom, Pan, Vorher/Nachher; Übergabe der Pixel ohne unnötige Kopien.

## Akzeptanzkriterien

Gerüst
- [ ] Der Kern baut und testet am Mac: `cargo test --workspace` und `cargo clippy --workspace --all-targets` ohne Warnungen; was
      Windows-spezifisch ist (Pfade, Zeilenenden, Testdaten-Suche in `crates/io/tests/common/mod.rs`), ist portabel gemacht
- [ ] `crates/session` enthält die Sitzungslogik, die Tauri-Shell benutzt sie, alle bestehenden Tests sind grün und der
      Fensterlauf der Windows-App ist unverändert (Auswertung: Commit `[TASK] RK-… - …` getrennt vom Mac-Teil)
- [ ] `crates/ffi` mit Swift-Bindings und XCFramework; ein Test in Swift ruft den Rechenkern auf und prüft ein Ergebnis aus
      `docs/testdaten.md` (z. B. Stack gegen die Referenz mit PSNR-Schwelle)
- [ ] `mac/project.yml`, `mac/README.md` Abschnitt „Bauen und veröffentlichen“ ausgefüllt (xcodegen, `xcodebuild`, Bündel,
      Signierung und Notarisierung), `/release mac <version>` läuft durch

Oberfläche (nach `docs/design.md`, Soll-Bilder `docs/design/screenshots/`, dunkel und hell, Tokens aus `docs/design/tokens.css`)
- [ ] Fenster mit Werkzeugleiste, Schrittleiste (Open · Alignpoints · Align · Limit · Stack · Wavelets · Save), Schrittpanel,
      Bildbereich, Frame-Leiste und Statusleiste in den Zuständen aus `docs/design.md`
- [x] Schritt **Open**: Aufnahme öffnen (SER, AVI, Bildserien, FITS), MRU, Dateiangaben
- [x] Schritte **Alignpoints**, **Align**, **Limit**: Set Alignpoints, Align mit Fortschritt, Pause und Cancel, Limit mit
      Qualitätsbalken, Framelist, Registrationgraph, R.o.Interest, Frames/Apoint, erneutes Ausrichten wie in RK-005
- [x] Schritt **Stack**: alle wirksamen Optionen (Nearestby Alignpoints, Correct geometry, Normalization, Edge pixels), die
      unwirksamen gesperrt mit „Not available yet“
- [ ] Schritt **Wavelets**: sechs Ebenen, Process, Do All, Before, Processing Area, Live-Vorschau, Colour/LRGB, Functions-Karten
      (Histogram, Gamma, RGB Align, RGB Balance), Contrast/Brightness, Load Scheme und Save Scheme
- [ ] Schritt **Save**: Save image in allen Formaten, Save Aligned, Save Integrated, Save project und Open project
- [ ] Bildbereich: Zoom um den Zeiger, Pan, 100 %-Klick, Einpassen bei Schrittwechsel, Vorher/Nachher
- [ ] CPUs-Feld wirkt auf den Thread-Pool des Kerns (Standard alle Kerne)
- [ ] Help öffnet die Anwenderseiten (`app/src/docs/pages/`, dieselben Markdown-Dateien) in einem eigenen Fenster
- [ ] Mac-Konventionen als bewusste Abweichung in `docs/design.md` eingetragen: Menüleiste und ⌘-Kürzel (⌘O Open, ⌘S Save image, ⌘.
      Cancel, ⌘? Help), native Dateidialoge, Vollbild, About im App-Menü, Einstellungen im Menü statt in der Werkzeugleiste, wo es der
      Konvention folgt

Qualität
- [ ] Vergleich mit der Windows-App an JUPITER.AVI: gleiche Anzahl Alignpoints, gleiche Frame-Reihenfolge, Stack und Do All
      innerhalb der PSNR-Schwellen aus `docs/stacking.md` und `docs/wavelets.md`
- [ ] Swift-Tests (`swift test`) und die Rust-Tests laufen grün, jeder neue Test war einmal rot (an einer Assertion)
- [ ] Hand-Probe am Mac: Der ganze Ablauf mit JUPITER.AVI bis Save image, eingetragen und abgehakt in `docs/parity/Handprobe.md`
- [ ] Danach: `docs/parity/MAC_PARITY.md` enthält den Stand „Mac gleich Windows“, ab hier trägt jede Änderung ihren Gegenpunkt ein
      (der Commit-Hook erzwingt es)

## Vorgehen (Vorschlag)

1. Am Mac Repo klonen, Hook aktivieren (`git config core.hooksPath .githooks`), Kern bauen und testen (Kriterium 1).
2. Sitzungslogik in `crates/session` ziehen, Windows-App bleibt grün (Kriterium 2). Auf Windows nachprüfen lassen.
3. Schnittstelle und Swift-Paket mit einem ersten Durchstich: Aufnahme öffnen, Frame anzeigen, Alignpoints setzen.
4. Schritt für Schritt die Oberfläche, jeden Schritt mit eigenem Commit `[FEATURE] MAC-001 - …`; die Windows-Seite bekommt hier
   noch keine Punkte, denn bis zum Abschluss gilt Windows als Vorlage (`docs/parity/MAC_PARITY.md`).
5. Prüfen, Handprobe, erster Release `v0.1.0` mit `/release mac 0.1.0`.

## Hinweise

- Worktrees sind am Mac erlaubt (auf Windows nicht): Branch `MAC-001-…`, Squash-Merge, ein Commit je Teil-Ticket (die Teile in
  eigene Tickets `MAC-002 …` zerlegen, sobald klar ist, wo die Schnitte liegen; dieses Ticket bleibt die Klammer).
- Die Spezifikation des Originals steht in `docs/` (Reverse Engineering von the reference program). Original-Binaries und Beispieldaten
  (`re-local/`) sind nicht in Git; für Referenztests den Ordner kopieren und `PLANETS_RE_LOCAL` setzen.
- Wo die Windows-App ihre Grenzen hat (Resampling, Drizzling, StackSize, Stretch histogram, De-rotate, Realign with Processed,
  Scan Frames, Flat/Dark/Reference), baut die Mac-App sie ebenfalls als „Not available yet“, bis ein eigenes Ticket sie umsetzt.

## Teil-Tickets und Entscheidungen (2026-11-12)

Zerlegt in `MAC-002` bis `MAC-010` (siehe `mac/README.md`, Status). Mit Philipp entschieden:

- Deploy am Mac: der post-commit-Hook baut und startet die Mac-App (`scripts/deploy-mac.sh`), nicht die Tauri-App.
- Brücke: UniFFI in `crates/ffi`; `unsafe` nur dort erlaubt, alle anderen Crates bleiben bei `forbid`.
- Arbeit direkt auf `main`, ein Commit je Teil-Ticket.
- `crates/session` wird auch von der Tauri-Shell benutzt; der Fensterlauf der Windows-App ist Handprobe am Windows-PC.
- Design: Tokens und Layout aus `docs/design.md`, Bedienelemente nach Mac-Art.
- Signierung ad-hoc, Notarisierung offen bis eine Developer-ID da ist. macOS 14+, Debug arm64, Release universal.

## Offene Fragen / Risiken

- Testdaten: Die Eingabevideos (JUPITER, CLAVIUS, SUNSPOTS) liegen am Mac unter `re-local/testdata/astropix/` (astropix.com). Die
  Ergebnisse des Originals (`re-local/reference/`) gibt es nur auf dem Windows-PC; die PSNR-Kriterien gegen das Original bleiben am
  Mac offen, bis der Ordner kopiert ist.

