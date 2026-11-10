# RK-005 Ausrichten und Qualitätsbewertung der Frames

- Status: in Arbeit (Rechenkern und Tauri-Befehle stehen; Qualitätsmaß weicht vom Original ab, Bedienung fehlt)
- Abhängig von: RK-004

## Ziel
Wie im Original: Referenzbereich wählen, alle Frames darauf ausrichten, Qualität je Frame schätzen,
Frames nach Qualität sortieren und begrenzen.

## Akzeptanzkriterien
- [x] Ausrichtung per Korrelation, subpixelgenau, an mehreren Ausrichtungspunkten (Alignpoints); davor eine grobe Suche für Drift bis 64 px
- [ ] Ausrichtung auf einem frei wählbaren Bereich (*R.o.Interest*, *Scan Frames*): fehlt, im Original unbeobachtet (`docs/offene-fragen.md`, Frage 21)
- [x] Qualitätsmaß je Frame (Schärfe), sortierte Liste, Auswahl über Prozentsatz/Anzahl (*Lowest Quality %*, *Best Frames %*, *Best Frames*)
- [ ] Qualitätsmaß entspricht dem Original: bei Referenzfall R1 sind die drei besten Frames dieselben (Original 1, 4, 3; Klon 1, 10, 8). Nicht erreicht, welches Maß das Original nutzt, ist offen (`docs/offene-fragen.md`, Frage 17), ein längerer Referenzfall in RK-011 würde das klären
- [ ] Limit-Modus *Frames/Apoint*: fehlt, im Original unbeobachtet
- [x] Verhalten mit dem Original an Testdaten verglichen (Alignpoint-Zahl, Reihenfolge, Limit); Ergebnis und Abweichungen in `docs/testdaten.md`, Test `crates/io/tests/align_reference.rs`. Versatzwerte des Originals sind nicht auslesbar, nur gegen künstliche Verschiebungen geprüft
- [ ] Alignpoint-Zahl abgeleitet statt angepasst: der Klon trifft bei R1 80 statt 82 Punkte, weil er pauschal 43 % der 141 gefundenen verwirft (`docs/offene-fragen.md`, Frage 18)
- [x] Parallel über Frames (`rayon`), Frames werden nacheinander gelesen und in Blöcken parallel ausgerichtet
- [ ] Bedienung im Align-Reiter angebunden (Alignpoints setzen, Align mit Fortschritt, Cancel und Pause, Limit, Framelist, Registrationgraph); Tauri-Befehle stehen in `app/src-tauri/src/align_commands.rs`, die Oberfläche folgt mit RK-010
- [ ] Anwenderdoku zum Align-Reiter in der App (Regel „Doku in der App“); die Doku-Ansicht entsteht in RK-012

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Align-Reiter komplett, Karten Framelist und Registrationgraph, Fortschritt mit Cancel und Pause (`docs/design.md`, Abschnitte Align, Zustände, Schwebende Karten)
- Soll-Bilder: `docs/design/screenshots/01-align-loaded.png` bis `04-align-aligned.png`, hell `11-align-light.png`
