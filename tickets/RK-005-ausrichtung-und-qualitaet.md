# RK-005 Ausrichten und Qualitätsbewertung der Frames

- Status: offen
- Abhängig von: RK-004

## Ziel
Wie im Original: Referenzbereich wählen, alle Frames darauf ausrichten, Qualität je Frame schätzen,
Frames nach Qualität sortieren und begrenzen.

## Akzeptanzkriterien
- [ ] Ausrichtung per Korrelation auf einem wählbaren Bereich (subpixelgenau), mehrere Ausrichtungsbereiche
- [ ] Qualitätsmaß je Frame, sortierte Liste, Auswahl über Prozentsatz/Anzahl
- [ ] Verhalten mit dem Original an Testdaten verglichen (Reihenfolge der besten Frames, Versatzwerte)
- [ ] Parallel über Frames (`rayon`)

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Align-Reiter komplett, Karten Framelist und Registrationgraph, Fortschritt mit Cancel und Pause (`docs/design.md`, Abschnitte Align, Zustände, Schwebende Karten)
- Soll-Bilder: `docs/design/screenshots/01-align-loaded.png` bis `04-align-aligned.png`, hell `11-align-light.png`
