# MAC-010 Hilfe, Menüs, CPUs und Release

- Status: in Arbeit
- Abhängig von: MAC-009
- Teil von: MAC-001, Kriterien „CPUs“, „Help“, „Mac-Konventionen“, „Bauen und veröffentlichen“, „Qualität“

## Akzeptanzkriterien

- [x] Help (⌘?) öffnet die Anwenderseiten aus `app/src/docs/pages/` (dieselben Markdown-Dateien, ins Bündel kopiert) in einem
      eigenen Fenster mit Navigation nach `group`/`order` und Suche
- [ ] Menüleiste: App-Menü mit About und Einstellungen (CPUs, Schemes), File (Open ⌘O, Open Recent, Save image ⌘S, Save project,
      Open project), Edit, View (Zoom, 100 %, Einpassen, Vollbild), Process (Schritte, Cancel ⌘.), Help
      Offen: Save image ⌘S, Save project und Open project legt MAC-009 in `AppCommands` an (die Menüs, Settings, About, Help, Edit, View und
      Process stehen); Schemes in den Einstellungen kommen mit MAC-008, die übrigen Einstellungen des Windows-Menüs mit den Schritten, die sie brauchen.
- [x] CPUs-Feld wirkt auf den Thread-Pool der Session (Standard alle Kerne)
- [x] `mac/README.md` „Bauen und veröffentlichen“: xcodegen, `xcodebuild`, Release universal, ad-hoc signiert; Notarisierung
      beschrieben, aber offen, bis eine Developer-ID hinterlegt ist
- [x] Skill `release` Abschnitt Mac ausgefüllt, `mac/CHANGELOG.md` angelegt
- [x] Handprobe-Punkte für den Mac in `docs/parity/Handprobe.md` aktuell
