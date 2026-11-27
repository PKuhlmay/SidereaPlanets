# SP-027 Überall Siderea Planets

- Status: erledigt
- Abhängig von: MAC-013, MAC-014

## Ziel

Die App hat auch intern nur noch ihren eigenen Namen, und der Name des Referenzprogramms steht nirgends mehr im Repo.

## Getroffene Entscheidungen (Philipp, 2026-11-27)

- Es wird alles umbenannt, auch intern: Xcode-Ziel und Swift-Module `SidereaPlanets…`, Rust-Crates `planets-*`, Typen `Planets…`,
  Testvariablen `PLANETS_*`, Kürzel `SP` statt des früheren in Design-Bausteinen und gespeicherten Einstellungen.
- Projektdateien enden auf `.sidereaplanets`. Die frühere Endung wird nicht mehr gelesen; der Inhalt ist unverändert, Umbenennen genügt.
- Der Name des Referenzprogramms wird auch aus Kommentaren, Bezeichnern, `docs/` und Tickets entfernt.
- Alle Tickets heißen `SP-NNN`; Mac-Tickets bleiben `MAC-NNN`.
- Die Entwurfsdatei heißt `docs/design/siderea-planets.figcraft`, auch ihr Inhalt ist angepasst.
- Das GitHub-Repo heißt `SidereaPlanets`.
- Alte Commit-Messages werden nicht umgeschrieben.

## Akzeptanzkriterien

- [x] Weder Dateiinhalte noch Dateinamen enthalten den Namen des Referenzprogramms oder den früheren Arbeitsnamen.
- [x] `cargo test`, `cargo clippy`, die Frontend-Tests und der Frontend-Bau laufen durch.
- [x] Swift-Paket und Mac-App bauen, ihre Tests laufen durch.
- [x] `core.hooksPath` zeigt wieder auf `.githooks` (er zeigte auf den früheren, nicht mehr vorhandenen Ordner).

## Offene Fragen / Risiken

- Der Windows-Installer ist am Mac nicht prüfbar; beim nächsten Windows-Bau mitprüfen.
- Die Windows-App beginnt mit leerer Recent-Liste und Standardeinstellungen, weil sich die Speicherschlüssel geändert haben.
- Die Entwurfsdatei wurde außerhalb von FigCraft geändert und dort noch nicht geöffnet. Die Soll-Bilder 13 und 14 sind direkt im Bild
  angepasst und weichen in der Schrift der geänderten Zeilen leicht ab, bis sie neu exportiert werden.
- Der Remote-Branch `spike/rk-002-tauri` behält seinen Namen.
