# MAC-013 Name „Siderea Planets“ in beiden Apps

- Status: erledigt
- Abhängig von: MAC-012 (App-Icon)

## Ziel

Die App heißt für Nutzer **Siderea Planets** und gehört damit zur Suite mit Siderea (Muster „Siderea“ + Tool). Entschieden von
Philipp am 2026-11-12: alles, was Nutzer sehen, wird umbenannt; Repo, Ordner, Crates, Swift-Module, Tickets und die interne
Doku (`docs/`, Reverse Engineering) behielten in diesem Ticket den früheren Arbeitsnamen. Er ist inzwischen auch dort ersetzt; die
Bezeichner in diesem Ticket sind die heutigen. Hinweise auf das Referenzprogramm (als Vorbild, Begriffe, Vergleiche) sollten
bleiben, wo sie fachlich gemeint sind.

## Akzeptanzkriterien

- [x] Mac: Anzeigename, `PRODUCT_NAME` (Siderea Planets.app), Bundle-ID `de.kuhlmay.siderea.planets`, App-Menü, About, Fenstertitel,
      Hilfe-Fenster, Copyright; Deploy- und Release-Skripte finden die umbenannte App (Beenden und Starten beim Deploy)
- [x] Windows: `productName`, Fenstertitel, Installer-Name (`Siderea Planets_<version>_x64-setup.exe`), Identifier in
      `tauri.conf.json`, Hilfe-Fenster, sichtbare Texte im Vue-Frontend
- [x] Anwenderseiten `app/src/docs/pages/` nennen die App „Siderea Planets“
- [x] `CLAUDE.md`, `mac/README.md`, Release-Skill und Changelogs nennen den neuen Anzeigenamen dort, wo es um die ausgelieferte App geht
- [x] Tests grün (cargo, swift test, xcodebuild test, pnpm test), keine Treffer für den früheren Arbeitsnamen mehr in sichtbaren Texten
      (Suche dokumentiert im Ticket)

## Umsetzung und Hinweise

- **Mac:** `PRODUCT_NAME` „Siderea Planets“ (Bündel `Siderea Planets.app`, Ausführungsdatei gleichnamig), `PRODUCT_MODULE_NAME` bleibt
  `SidereaPlanets` (`@testable import SidereaPlanets`), Xcode-Ziel, -Schema und Testziel heißen weiter `SidereaPlanets`. Bundle-ID
  `de.kuhlmay.siderea.planets` (Tests: `….tests`), `CFBundleDisplayName`, `NSHumanReadableCopyright`, Fenstertitel
  („Siderea Planets — ‹Datei›“, „Siderea Planets · Align 40%“), Menüs „About Siderea Planets“ und „Siderea Planets Help“, Titel des
  Hilfe-Fensters. Der Name im App-Menü und im Dock kommt aus `CFBundleName`/`PRODUCT_NAME`. Das Testziel bekam `TEST_HOST` und
  `BUNDLE_LOADER` ausdrücklich, weil xcodegen den Host sonst aus dem Zielnamen ableitet.
- **Windows:** `productName` „Siderea Planets“ (Installer `Siderea Planets_<version>_x64-setup.exe`), Identifier
  `com.pkuhlmay.siderea.planets` (Stil des alten Identifiers beibehalten), Fenstertitel, `index.html`, Titel des
  Hilfe-Fensters, `windowTitle()` mit Test, Dateifilter „Siderea Planets project“ in den Projekt-Dialogen, Beschreibung im
  `Cargo.toml` der Shell. Nicht geprüft: ein echter `pnpm tauri build` und der Installer (am Mac nicht möglich).
- **Gemeinsam:** FITS-Kopfzeile `CREATOR  = 'Siderea Planets'` (sichtbar für Nutzer in jeder gespeicherten FITS-Datei), alle Seiten
  unter `app/src/docs/pages/` (für beide Apps), `mac/CHANGELOG.md`, `mac/README.md`, `CLAUDE.md`, Skill `siderea-release` (Installer- und
  Bündelnamen, Tag-Texte), `docs/design.md` (die Zeilen mit Fenstertitel und Menünamen; es beschreibt die Oberfläche der App),
  `docs/parity/Handprobe.md` (Dateiname des Bündels).
- **Skripte:** `scripts/deploy-mac.sh` startet und beendet `Siderea Planets.app`; beim Neustart beendet es außerdem einmal eine noch
  laufende App aus dem alten Bündel mit dem früheren Arbeitsnamen, entregistriert es (`lsregister -u`) und löscht es. `scripts/build-mac-release.sh`
  baut mit dem Schema `SidereaPlanets`, nennt das Bündel `Siderea Planets.app` und Zip und DMG `Siderea-Planets-<version>-mac.zip/.dmg`
  (ohne Leerzeichen im Dateinamen). In README und Skill stehen die neuen Namen; das notarytool-Profil heißt dort jetzt `siderea-planets`
  (nur Beispielname, noch nicht angelegt).
- **Bundle-ID und Einstellungen:** Die UserDefaults hängen an der Bundle-ID. Mit `de.kuhlmay.siderea.planets` beginnt die MRU-Liste
  („Open Recent“) der Mac-App leer, ebenso die gespeicherte CPUs-Einstellung; das ist gewollt. Unter Windows liegen die Daten des Webviews
  unter dem Identifier; auch sie beginnen mit dem neuen Identifier leer (die App speichert dort keine Projekte).
- **Parität:** Beide Apps ändern sich im selben Commit, deshalb gibt es keinen Eintrag in `WINDOWS_PARITY.md` oder `MAC_PARITY.md`.
  Der Hook verlangt trotzdem eine Markierung je Seite; die Commit-Nachricht trägt `[mac-only]` und `[windows-only]` im Text.

## Suche nach dem früheren Arbeitsnamen (Ergebnis)

Gesucht wurde ohne Beachtung der Groß- und Kleinschreibung im ganzen Repo außer `tickets/` und `mac/tickets/`, Stand nach der
Umbenennung. Treffer in sichtbaren Texten: keine. Die übrigen waren intern und blieben in diesem Ticket:

- `mac/project.yml`, `mac/Tests/SidereaPlanetsTests/*` (`@testable import SidereaPlanets`, Ordnername), `mac/App/SidereaPlanetsApp.swift`
  (Typname), Xcode-Projekt und -Schema, Aufrufe `xcodebuild -project mac/SidereaPlanets.xcodeproj -scheme SidereaPlanets` in Skripten,
  README und Skill: interne Namen (Ziel, Schema, Modul).
- `scripts/deploy-mac.sh`: die Variable `old_app_name` mit dem Namen des alten Bündels, um dieses aufzuräumen
  (inzwischen entfernt).
- `crates/ffi/Cargo.toml`, `crates/session/Cargo.toml`, `crates/session/src/lib.rs`: Beschreibungen und Kommentare der Crates, nicht
  sichtbar. `app/package.json` (`name: siderea-planets`) und die Namen der Crates: interne Paketnamen.
- `CLAUDE.md` (Überschrift und der Absatz „Name“), `mac/README.md` (Erklärung des internen Namens), `.claude/skills/nachzug`:
  Repo-Name und Erklärung der Umbenennung.
- `docs/design/` (Entwurfs-HTML, `tokens.css`, `figcraft-build.mjs`, SVG-Kommentare der Icon-Ebenen): Entwurfs- und Quellmaterial, nicht
  ausgeliefert.
- Die Hinweise auf das Referenzprogramm blieben (Seiten, Texte, Dateifilter für `.rwv`); aus den sichtbaren Texten nimmt sie erst
  MAC-014.

Geprüft: `cargo fmt --check`, `cargo clippy --workspace --all-targets` ohne Warnungen, `cargo test --workspace` 361 grün,
`swift test` im Paket, `xcodebuild test` 234 Tests grün, `pnpm test` 241 grün, `pnpm build`.
