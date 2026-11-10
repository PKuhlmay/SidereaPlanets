# RK-005 Ausrichten und Qualitätsbewertung der Frames

- Status: in Arbeit (Rechenkern fertig, Bedienung folgt mit RK-010)
- Abhängig von: RK-004

## Ziel
Wie im Original: Referenzbereich wählen, alle Frames darauf ausrichten, Qualität je Frame schätzen,
Frames nach Qualität sortieren und begrenzen.

## Akzeptanzkriterien
- [x] Ausrichtung per Korrelation (subpixelgenau) mit mehreren Ausrichtungsbereichen (Alignpoints), grobe Suche für Drift bis 64 px davor. Ein frei wählbarer einzelner Bereich (*R.o.Interest*, *Scan Frames*) fehlt noch: im Original unbeobachtet, siehe `docs/offene-fragen.md`
- [x] Qualitätsmaß je Frame, sortierte Liste, Auswahl über Prozentsatz/Anzahl (*Lowest Quality %*, *Best Frames %*, *Best Frames*; *Frames/Apoint* ist unbeobachtet und fehlt)
- [x] Verhalten mit dem Original an Testdaten verglichen (Alignpoint-Zahl, Reihenfolge, Limit); Ergebnis und Abweichungen in `docs/testdaten.md`, Test `crates/io/tests/align_reference.rs`. Versatzwerte des Originals sind nicht auslesbar, nur gegen künstliche Verschiebungen geprüft
- [x] Parallel über Frames (`rayon`), Frames werden nacheinander gelesen und in Blöcken parallel ausgerichtet

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Align-Reiter komplett, Karten Framelist und Registrationgraph, Fortschritt mit Cancel und Pause (`docs/design.md`, Abschnitte Align, Zustände, Schwebende Karten)
- Soll-Bilder: `docs/design/screenshots/01-align-loaded.png` bis `04-align-aligned.png`, hell `11-align-light.png`
