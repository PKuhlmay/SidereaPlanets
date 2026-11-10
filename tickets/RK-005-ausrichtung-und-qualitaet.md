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
