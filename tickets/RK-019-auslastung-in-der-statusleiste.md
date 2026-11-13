# RK-019 Auslastung in der Statusleiste (Windows zieht nach)

- Status: erledigt
- Abhängig von: MAC-015
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die Statusleiste zeigt links CPU-Last und Arbeitsspeicher der App, abschaltbar in Settings, wie in der Mac-App.

## Umfang

Formate, Schwelle, Tooltips, Messintervall und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „Auslastung in der
Statusleiste (Mac: MAC-015)“; die Rechenregeln stehen in den Mac-Tests `mac/Tests/SidereaPlanetsTests/ProcessUsageTests.swift` und
werden als Vitest-Fälle übernommen.

Gemessen wird im Tauri-Prozess (`app/src-tauri/src`): Prozess-CPU-Zeit und Working Set. Die Messung ist Windows-eigen und bleibt in der
Shell, nicht im Rechenkern (`crates/` bleibt plattformunabhängig). Die Formatierung („  7 %“, „1.5 GB“) gehört ins Frontend
(`app/src/logic/`), damit sie testbar ist.

## Akzeptanzkriterien

- [x] Feld „CPU (n)“ und „RAM“ ganz links in der Statusleiste; Formate und Deckelung wie im Parity-Punkt (Tests aus den Mac-Regeln)
- [x] Ab 90 % der Kerne ist der CPU-Wert `--accent` und fett; bei Platzmangel fällt RAM zuerst weg, kein Umbruch
- [x] Messung einmal pro Sekunde, nur solange das Fenster sichtbar ist (nicht minimiert)
- [x] Settings-Schalter **Show CPU and memory usage** (Standard an); aus, verschwindet das Feld samt Trennlinie und die Messung stoppt
- [x] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## RK-019 Auslastung in der Statusleiste (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen

## Umsetzung

- Messung: Befehl `process_usage` in `app/src-tauri/src/usage_commands.rs` über `sysinfo` 0.40 (nur das Feature `system`, kein eigenes
  `unsafe`): CPU-Zeit (Benutzer plus System) und Working Set des eigenen Prozesses; die Prozesstabelle bleibt zwischen zwei Messungen
  erhalten, gemessen wird nur dieser Prozess.
- Rechenregeln und Texte in `app/src/logic/processUsage.ts`, Tests aus `ProcessUsageTests.swift` übernommen (ohne die deutsche
  Dezimalschreibweise: die Windows-App schreibt Zahlen überall mit Punkt). Monitor in `app/src/resource.ts`: einmal pro Sekunde, nur
  solange der Schalter an ist und das Fenster sichtbar (`visibilitychange`, ein minimiertes Fenster ist für das Webview verborgen); eine
  fehlgeschlagene Messung oder ein Stopp löscht beide Werte, die Anzeige zeigt dann „–“.
- Statusleiste: Feld ganz links vor dem Fortschritt, „CPU (n)“ mit n = logische Kerne (wie das CPUs-Feld), ab 90 % fett in
  `--accent-ink`; RAM fällt per Container-Abfrage unter 1000 px Breite der Leiste weg.
- Settings: Eintrag **Show CPU and Memory Usage** (Title Case wie die übrigen Menüeinträge, am Mac heißt der Schalter im Settings-Fenster
  „Show CPU and memory usage“), Standard an, gemerkt im `localStorage`. Hilfeseite `menus-and-settings.md` ergänzt.
