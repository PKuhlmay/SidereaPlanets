# RK-019 Auslastung in der Statusleiste (Windows zieht nach)

- Status: offen
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

- [ ] Feld „CPU (n)“ und „RAM“ ganz links in der Statusleiste; Formate und Deckelung wie im Parity-Punkt (Tests aus den Mac-Regeln)
- [ ] Ab 90 % der Kerne ist der CPU-Wert `--accent` und fett; bei Platzmangel fällt RAM zuerst weg, kein Umbruch
- [ ] Messung einmal pro Sekunde, nur solange das Fenster sichtbar ist (nicht minimiert)
- [ ] Settings-Schalter **Show CPU and memory usage** (Standard an); aus, verschwindet das Feld samt Trennlinie und die Messung stoppt
- [ ] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-019 Auslastung in der Statusleiste (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen
