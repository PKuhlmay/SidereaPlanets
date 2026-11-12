# MAC-006 Schritte Alignpoints, Align und Limit

- Status: offen
- Abhängig von: MAC-005
- Teil von: MAC-001, Kriterium „Schritte Alignpoints, Align, Limit“

## Akzeptanzkriterien

- [ ] Alignpoints: Set Alignpoints mit allen Feldern der Windows-App (`app/src/components/AlignpointsStep.vue`), Threshold-Regler,
      Punkte als Overlay im Bildbereich, R.o.Interest per Ziehen
- [ ] Align: Fortschritt (Balken, Frame-Zähler), Pause und Cancel (⌘.), Registrationgraph, Frames/Apoint, erneutes Ausrichten
      verwirft die Ergebnisse (Session)
- [ ] Limit: Qualitätsbalken (`QualityChart.vue`), Framelist nach Qualität, Limit mit Prozent/Anzahl wie Windows
- [ ] Gesperrte Optionen der Windows-App auch hier gesperrt mit „Not available yet“
- [ ] Mit JUPITER.AVI: gleiche Anzahl Alignpoints und gleiche Frame-Reihenfolge wie die Windows-App (gleicher Kern)
