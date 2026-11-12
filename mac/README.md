# Siderea Planets für den Mac

Die App heißt für Nutzer **Siderea Planets** (MAC-013); „SidereaPlanets“ bleibt der interne Name von Repo, Xcode-Ziel und -Schema und
Swift-Modul (`PRODUCT_NAME` ist „Siderea Planets“, `PRODUCT_MODULE_NAME` „SidereaPlanets“).

Native Mac-App in **SwiftUI** auf dem gemeinsamen **Rust-Kern** (`crates/`), analog zu Siderea: Die Mac-App führt, die Windows-App
(`app/`, Tauri und Vue) zieht nach. Die Mac-App entsteht mit
[`tickets/MAC-001-mac-app-aufbauen.md`](tickets/MAC-001-mac-app-aufbauen.md) am Mac; Stand siehe „Status“.

## Aufbau

```
mac/
  project.yml            xcodegen-Spezifikation (Ziele SidereaPlanets und SidereaPlanetsTests, MARKETING_VERSION, ad-hoc-Signierung)
                         App-Icon: `docs/design/icon/AppIcon.icon` (Debug: `AppIconDev.icon` mit DEV-Abzeichen), gemeinsam mit Windows, neu erzeugen mit `scripts/make-icons.sh` (MAC-012)
  App/                   SwiftUI-Oberfläche (MAC-005)
    SidereaPlanetsApp.swift  Szenen (Hauptfenster, Help, Settings), Menüleiste (⌘O, Open Recent, Zoom, Frames, Run)
    Commands/            HelpCommands (Help ⌘?, About), StepMenuItems (Process > Go to Step, ⌥⌘1 bis ⌥⌘7); weitere Menüpunkte (Save …) kommen hier oder in AppCommands dazu
    Help/                Hilfe-Fenster (MAC-010): DocPage (Kopf, Gruppen, Suche), MarkdownHTML (Renderer), DocStyle (CSS aus den Tokens), HelpModel, HelpView, DocWebView (WKWebView)
    Settings/            Einstellungen (⌘,): CpuSettings (CPUs, Klemmung, Kern), SettingsView
    Design/              Tokens.swift (Spiegel von docs/design/tokens.css, hell und dunkel), RKButtonStyle
    Model/               AppModel (@Observable, MainActor: Sitzung, Schritt, Frame, Bild, Statustexte), AppModel+Align (Set Alignpoints, Align,
                         Limit, Cancel, Pause, Framelist), AlignLogic (Werte, Diagramm, Meldungen), AppModel+Stack (Stack, Karte
                         Alignmentpoints, Ringe), StackLogic (Werte, Anfrage, Texte), StepLogic (Schrittzustände),
                         RecentList (MRU), RecordingDetails (Dateiangaben), ViewTransform und ImageViewport (Zoom und Pan),
                         SessionBackend (was die App vom Kern braucht), RecordingPicker (NSOpenPanel), DebugLaunch (nur Debug),
                         AppModel+Save (Save image, Save Aligned/Integrated, Save und Open project), SaveLogic (Create-AVI-Werte,
                         Projektabschnitte, JSON), SavePanels (NSSavePanel und NSOpenPanel),
                         AppModel+Wavelets (Vorschau, Process, Do All, Schemes, Karten, Contrast/Brightness, Copy To), WaveletLogic (Werte,
                         Verknüpfte Regler, Processing Area, Töne, Histogramm, Differenz), WaveletBackend (was der Schritt vom Kern braucht),
                         WaveletProject (Abschnitte wavelet und colour der Projektdatei), ColorMixLogic (Matrix, Presets), CompareLogic und AppModel+Compare (View Compare)
    Views/               ContentView (Fenster und Werkzeugleiste), StepBar, StepPanel, AlignPanels (Alignpoints, Align, Limit mit
                         Qualitätsdiagramm), StackPanel, Controls (Zahlenfeld, Auswahl), ImageArea (mit Show-Menü), ImageCanvas (NSView mit einer
                         CALayer für das Bild, zieht auch die R.o.Interest-Region), ImageOverlays (Punkte, Ringe, Region), FloatingCards
                         (Framelist, Registrationgraph, Alignmentpoints), FrameBar, StatusBar, Save/SaveViews (Schritt Save, Menü Save image,
                         Gruppe Create AVI im Stack-Panel), Wavelets/ (WaveletsPanel mit Ebenen und Schemes, FunctionsPanel rechts,
                         FunctionCards Histogram, Gamma, RGB Align, RGB Balance, ColorMixingCard, CompareOverlay (View Compare), RefineCards
                         Denoise/Deringing und Masking (MAC-018, Werte in Model/RefineLogic), WaveletControls: Dezimalfeld, Segmente, Textlinks)
  Tests/SidereaPlanetsTests/ Swift Testing: Schrittzustände, MRU, AppModel (mit FakeBackend), Zoom-Rechnung, Dateiangaben, Hilfeseiten (HelpDocsTests), CPUs (CpuSettingsTests)
  Packages/SidereaPlanetsKit/  Swift-Paket um die Rust-Schnittstelle (XCFramework und erzeugte Bindings)
  tickets/               MAC-NNN-kurzer-titel.md
  CHANGELOG.md           ab dem ersten Mac-Release
  README.md              diese Datei
crates/ffi/              Rust-Schnittstelle (UniFFI) auf den Rechenkern, GUI-frei (MAC-004)
```

Die App rechnet nichts selbst: `AppModel` hält nur, was die Ansichten zeigen, und ruft den Kern über das Protokoll `SessionBackend`
(`PlanetsSession` ist die echte Umsetzung, die Tests bringen eine Attrappe). Spätere Schritte (MAC-008 bis MAC-010) hängen ihre
Aufrufe dort und ihre Panels in `StepPanel` ein; `ViewStep.isBuilt` sagt, welche Schritte schon ein Panel haben. Das Bild geht als
`CGImage` in eine Layer (`ImageCanvas`): Zoomen und Verschieben ändern nur ihr Frame, die Pixel werden pro Frame einmal hochgeladen, und
Frames lesen und umrechnen auf einer Hintergrund-Queue (`@concurrent`), immer nur einer zugleich, mit dem neuesten Wunsch als nächstem.

**Sandbox aus:** Die App darf vorerst Aufnahmen an jedem Ort lesen und Ergebnisse überall schreiben (`ENABLE_APP_SANDBOX: NO` in
`project.yml`); für ein sandboxed Release brauchte es Security-scoped Bookmarks für die MRU und Entitlements für Dateizugriff.
Entscheidung mit MAC-010: **Sandbox bleibt aus**. Die App wird als Download verteilt (ad hoc signiert, später Developer ID mit
Notarisierung), nicht über den App Store, und dafür ist die Sandbox keine Pflicht. Der Mac App Store wäre ein eigenes Ticket.

**Debug-Argumente** (nur Debug-Builds, für Screenshots und Handproben): `open -a build/mac/Build/Products/Debug/"Siderea Planets.app" --args
-open <Datei[:Datei…]> -appearance light|dark -step open|alignpoints|… -frame <n>`.
Dazu für Bilder der späteren Schritte: `-run alignpoints,align,limit,stack` drückt die Schaltflächen nacheinander (`alignpaused` startet Align und
hält es nach dem ersten Frame an, `stackpaused` dasselbe für Stack, `roidraw` drückt Draw Region, `bestframe` drückt Go to Best Frame), `-show framelist,registrationgraph` (interne Namen; in der App Frame List und Quality Graph), `-roi x,y,breite,höhe`,
`-limit lowestQuality|bestPercent|framesPerPoint|bestCount`, `-advanced wavelets,stack` (oder `all`) klappt Advanced dieser Schritte auf, ohne es zu merken.

Das einzige, was Mac und Windows teilen, ist der **Rechenkern**: `crates/core`, `crates/io` und die GUI-freie Sitzungslogik in
`crates/session` (aus den Tauri-Befehlen gezogen, MAC-003; die Tauri-Shell ist nur noch eine dünne Hülle darüber). Alles, was eine
Oberfläche sonst nachbauen müsste (Zustand, Verwerfen der Ergebnisse nach neuem Align, Thread-Pool, Abbrechen und Pausieren), gehört in
den Kern, nicht in die Oberfläche. Die Mac-Schnittstelle `crates/ffi` (MAC-004) legt sich über dieselbe `Session`.

## Rust-Schnittstelle und Swift-Paket (MAC-004)

```
crates/ffi/                          UniFFI-Schnittstelle: ein Objekt `CoreSession` um `Session`, Records und Enums als Spiegel der
                                     Anfrage- und Ergebnistypen, `PlanetsError` mit dem Text des Kerns, Fortschritt über `ProgressListener`
mac/Packages/SidereaPlanetsKit/
  SidereaPlanetsFFI.xcframework            statische Bibliothek und C-Header (erzeugt, nicht in Git)
  Sources/SidereaPlanetsBindings/          das von UniFFI erzeugte Swift (erzeugt, nicht in Git)
  Sources/SidereaPlanetsKit/               dünne Schicht: `PlanetsSession` mit `async`, Fortschritt, Abbruch, `CGImage`
  Tests/SidereaPlanetsKitTests/            `swift test` gegen JUPITER.AVI
```

- `crates/ffi` erbt die Workspace-Lints **nicht**: UniFFI erzeugt `unsafe`-Code, das nur hier erlaubt ist (Begründung in
  `Cargo.toml` und `lib.rs`); alle anderen Crates bleiben bei `forbid`.
- `scripts/build-ffi.sh` baut die Bibliothek (Debug nur arm64 und optimiert mit dem Cargo-Profil `ffi-dev`, `--release` zusätzlich x86_64 und `lipo` zu einer universalen),
  erzeugt die Bindings im Bibliotheksmodus mit dem `uniffi-bindgen` dieses Workspace und legt XCFramework und Swift-Datei ins Paket.
  Vor dem ersten `swift build`/`swift test` und nach jeder Änderung an `crates/ffi` oder `crates/session` laufen lassen; der
  Deploy-Hook tut es selbst.
- Prüfen: `scripts/build-ffi.sh && cd mac/Packages/SidereaPlanetsKit && swift test` (der Test überspringt sich ohne
  `re-local/testdata/astropix/JUPITER.AVI`).
- Lange Aufrufe (Align, Stack, Do All, Save Aligned/Integrated) blockieren in Rust; `PlanetsSession` führt sie auf einer
  Hintergrund-Queue aus (`async`), meldet Fortschritt über ein Closure und bricht ab, wenn der Swift-`Task` abgebrochen wird oder
  `cancel(_:)` gerufen wird (Fehler `PlanetsError.Cancelled`, `isCancelled`). Den Fehlertext liefert `PlanetsError.message`.
- Der Rust-Typ `Operation` heißt in Swift `SessionOperation` (Foundation hat schon eine `Operation`).
- Das Release-Profil des Workspace hat `panic = "abort"`: ein Panic im Kern beendet in einem Release-Bau die App, statt als Fehler
  anzukommen.

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
- Das Bündel heißt `Siderea Planets.app`. Beim Neustart beendet das Skript außerdem eine noch laufende App aus dem alten Bündel
  `SidereaPlanets.app` (Name vor MAC-013), entregistriert es bei LaunchServices (`lsregister -u`) und löscht es, damit nie zwei Apps
  nebeneinander liegen.
- Von Hand: `scripts/deploy-mac.sh --run`. Ausschalten: `DEPLOY_DEBUG=0`. Mit `DEPLOY_TESTS=1` läuft vorher
  `cargo test --workspace` (sonst aus, wie bei der Windows-App).
- Läuft gerade ein Deploy, merkt sich ein weiterer Commit das und der Deploy geht danach noch einmal durch.

## Bauen und veröffentlichen

### Entwicklung (Debug)

```bash
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
scripts/build-ffi.sh                          # XCFramework und Swift-Bindings (Debug arm64; --release universal)
cd mac && xcodegen generate                   # schreibt SidereaPlanets.xcodeproj und App/Info.plist (beide nicht in Git)
cd .. && xcodebuild -project mac/SidereaPlanets.xcodeproj -scheme SidereaPlanets -configuration Debug -arch arm64 \
  -derivedDataPath build/mac build            # die App liegt unter build/mac/Build/Products/Debug/Siderea Planets.app
xcodebuild test -project mac/SidereaPlanets.xcodeproj -scheme SidereaPlanets -destination 'platform=macOS,arch=arm64' \
  -derivedDataPath build/mac-test             # Swift-Tests der App (der Test startet die App kurz als Host)
scripts/deploy-mac.sh --run                   # alles zusammen, dann die App neu starten
```

- Bundle-ID `de.kuhlmay.siderea.planets` (vor MAC-013 `de.kuhlmay.sidereaplanets`; die Einstellungen und die MRU-Liste liegen im neuen Bundle-ID-Bereich der UserDefaults und beginnen leer), Bündel `Siderea Planets.app`, `MARKETING_VERSION` 0.1.0, macOS 14 oder neuer, Swift 6 (strikte Nebenläufigkeit), Signierung
  ad-hoc („Sign to Run Locally“, `CODE_SIGN_IDENTITY: "-"`).
- Zu jedem Commit der App gehören die Tests grün (`xcodebuild test` oben, dazu `swift test` im Paket) und ein Bau ohne Warnungen im
  eigenen Swift.
- Ein Test, der die Oberfläche braucht, startet die App als Host; Screenshots des laufenden Fensters: `screencapture -x -l <Fenster-ID>`
  (die ID aus `CGWindowListCopyWindowInfo`).
- Die Hilfeseiten sind keine zweite Kopie: `project.yml` bindet `app/src/docs/pages/` als Ordner-Ressource ein, der Bau kopiert ihn als
  `Resources/pages` ins Bündel (`HelpModel.bundled()` liest ihn dort). Eine neue Seite ist eine neue Datei in diesem Ordner.

### Release-Bündel

```bash
scripts/build-mac-release.sh            # build/release/Siderea Planets.app und Siderea-Planets-<version>-mac.zip
scripts/build-mac-release.sh --dmg      # zusätzlich Siderea-Planets-<version>-mac.dmg
```

Das Skript tut der Reihe nach: `scripts/build-ffi.sh --release` (Kern für arm64 und x86_64, mit `lipo` zu einer Bibliothek),
`xcodegen generate`, `xcodebuild` in Release mit `ARCHS="arm64 x86_64"` und `ONLY_ACTIVE_ARCH=NO` (Derived Data unter
`build/mac-release/`), Kopie nach `build/release/`, `codesign`, Prüfung mit `codesign --verify`, `lipo -info` (muss `x86_64 arm64`
zeigen) und das Zip mit `ditto -c -k --keepParent` (erhält die Signatur und die Dateiattribute; kein `zip`). Alles liegt unter `build/`
und ist nicht in Git. Es dauert einige Minuten, weil der Kern zweimal gebaut wird. Das Release-Profil hat `panic = "abort"`.

- **Signierung heute: ad hoc** (`CODE_SIGN_IDENTITY: "-"`, „Sign to Run Locally“). Das Bündel läuft auf dem eigenen Mac. Auf einem
  anderen Mac blockiert Gatekeeper es, weil es nicht notarisiert ist: Rechtsklick auf die App, Öffnen, bestätigen (oder in den
  Systemeinstellungen unter „Datenschutz & Sicherheit“ „Trotzdem öffnen“). Das gilt nur, solange die Datei eine Quarantäne-Markierung
  trägt (Download); ein Zip vom eigenen Rechner hat keine.
- **Notarisierung: beschrieben, offen, bis eine Developer-ID hinterlegt ist** (Apple Developer Program, 99 USD im Jahr). Dann:
  1. Das Zertifikat „Developer ID Application“ in den Schlüsselbund laden, die Zugangsdaten für `notarytool` einmal ablegen:
     `xcrun notarytool store-credentials siderea-planets --apple-id <Apple-ID> --team-id <TEAMID> --password <App-spezifisches Kennwort>`.
  2. Bündel mit Hardened Runtime signieren: `SIGN_IDENTITY="Developer ID Application: Name (TEAMID)" scripts/build-mac-release.sh`
     (das Skript setzt dann `--options runtime --timestamp`; die App braucht keine Entitlements, die Sandbox ist aus).
  3. Einreichen und warten: `xcrun notarytool submit build/release/Siderea-Planets-<version>-mac.zip --keychain-profile siderea-planets --wait`.
  4. Das Ticket in die App heften und das Zip neu erzeugen: `xcrun stapler staple "build/release/Siderea Planets.app"`, dann
     `ditto -c -k --sequesterRsrc --keepParent "build/release/Siderea Planets.app" Siderea-Planets-<version>-mac.zip`. Prüfen mit
     `spctl --assess --type execute -v "build/release/Siderea Planets.app"` („accepted, source=Notarized Developer ID“).
  Bis dahin ist dieser Weg nicht erprobt; er wandert in das Skript, sobald es eine Developer-ID gibt.
- **Version:** `MARKETING_VERSION` in `project.yml` (die Build-Nummer `CURRENT_PROJECT_VERSION` zählt bei Bedarf hoch). Veröffentlicht
  wird nur mit `/release mac <version>` (Skill `release`, Abschnitt 3b): Version setzen, `mac/CHANGELOG.md` füllen, prüfen, bauen, committen,
  taggen (`v<version>`), pushen nach Philipps OK.

## Status

| Ticket | Thema | Stand |
|---|---|---|
| MAC-001 | Mac-App aufbauen (Klammer) | in Arbeit |
| MAC-002 | Kern am Mac und Deploy der Mac-App | erledigt |
| MAC-003 | Sitzungslogik im Kern (`crates/session`) | erledigt |
| MAC-004 | Rust-Schnittstelle (UniFFI), XCFramework, Swift-Paket | erledigt |
| MAC-005 | App-Gerüst, Fenster, Bildbereich, Open | erledigt |
| MAC-006 | Alignpoints, Align, Limit | erledigt |
| MAC-007 | Stack | erledigt |
| MAC-008 | Wavelets, Farbe, Functions | erledigt |
| MAC-009 | Save, Export, Projektdatei | erledigt |
| MAC-010 | Hilfe, Menüs, CPUs, Release | erledigt |
| MAC-011 | Go to best frame | erledigt |
| MAC-012 | App-Icon (beide Apps, Debug mit DEV-Abzeichen) | erledigt |
| MAC-013 | Name „Siderea Planets“ in beiden Apps | erledigt |
| MAC-014 | Texte für Einsteiger (klare Namen, Tooltips, gesperrte Elemente, kein „the reference program“ in sichtbaren Texten) | erledigt |
| MAC-015 | CPU- und RAM-Anzeige in der Statusleiste, optimierter Kern im Debug-Build | erledigt |
| MAC-018 | Adjustments „Denoise/Deringing“ und „Masking“ | erledigt |
| MAC-019 | Adjustments: Color Mixing (3 × 3 Kanalmischer im Kern) und View Compare (geteilte Ansicht Before/After) | erledigt |
