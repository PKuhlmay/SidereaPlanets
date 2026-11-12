# MAC-006 Schritte Alignpoints, Align und Limit

- Status: erledigt
- Abhängig von: MAC-005
- Teil von: MAC-001, Kriterium „Schritte Alignpoints, Align, Limit“

## Akzeptanzkriterien

- [x] Alignpoints: Set Alignpoints mit allen Feldern der Windows-App (`app/src/components/AlignpointsStep.vue`), Threshold-Regler,
      Punkte als Overlay im Bildbereich, R.o.Interest per Ziehen
- [x] Align: Fortschritt (Balken, Frame-Zähler), Pause und Cancel (⌘.), Registrationgraph, Frames/Apoint, erneutes Ausrichten
      verwirft die Ergebnisse (Session)
- [x] Limit: Qualitätsbalken (`QualityChart.vue`), Framelist nach Qualität, Limit mit Prozent/Anzahl wie Windows
- [x] Gesperrte Optionen der Windows-App auch hier gesperrt mit „Not available yet“
- [x] Mit JUPITER.AVI: gleiche Anzahl Alignpoints und gleiche Frame-Reihenfolge wie die Windows-App (gleicher Kern)

## Umsetzung

- Zustände und Aufrufe: `mac/App/Model/AppModel+Align.swift` (Set Alignpoints, Threshold, R.o.Interest, Align mit Fortschritt, Cancel, Pause,
  Limit, Framelist), reine Rechnung (Werte und Voreinstellungen, Diagramm, Meldungen, Zeitmessung) in `AlignLogic.swift`, nach
  `app/src/align.ts` und `app/src/logic/{align,qualityChart,chart}.ts`. Panels `Views/AlignPanels.swift`, Überlagerungen
  `ImageOverlays.swift`, Karten `FloatingCards.swift`, Zahlenfeld und Auswahl `Controls.swift`.
- Wie in Windows: gleiche Felder, Voreinstellungen und Meldungen (`78 alignpoints set`, `Alignment finished.`, `Processing limited to … frames`,
  `Stacksize: … Low quality: …%`); gesperrt mit „Not available yet“: No Align, Scan Frames, Keep Alignpoints inside ScanFrame, Align by Centre
  of gravity, Lum. Threshold, Estimate Rotation; Empty List meldet „Empty List: not available yet“. Die Pille Stack ist nach Limit
  offen, ihr Panel kommt mit MAC-007.
- Mac-Art: Cancel (⌘.) und Pause/Resume auch im Menü „Process“; Cancel ist in der Werkzeugleiste gefüllt (`--accent`), solange
  Align läuft. Cancel und Pause wirken nur auf Align (Set Alignpoints und Limit sind in einem Augenblick fertig; Windows schaltet sie
  dort kurz frei). Ein Align, das trotz Cancel fertig wird, meldet sein Ergebnis statt „Cancelled“. Felder tragen Mindestwerte
  (Alignmentbox size und Anzahl Frames ab 1, Rest ab 0), Windows nimmt sie beim Senden zurecht.
- Der Rechenkern blieb unverändert; `OperationProgress` in `SidereaPlanetsKit` bekam einen öffentlichen Initialisierer (für die Attrappe in den Tests).
- Geprüft am laufenden Fenster mit JUPITER.AVI (dunkel und hell): Set Alignpoints (78 Punkte), Threshold-Regler, R.o.Interest per
  Ziehen (31 Punkte in der Region), Align mit Fortschritt, Pause und Resume, Limit mit Qualitätsdiagramm, Framelist und
  Registrationgraph über das Menü Show; Soll-Bilder `02`-`05` und `15`. Die Reihenfolge der Framelist ist die des Windows-Klons
  (1, 3, 10, 2, 8, 9, 4, 5, 6, 7, `docs/testdaten.md`).
- Tests: `AlignLogicTests`, `AlignModelTests` (Attrappe), `JupiterModelTests` (echter Kern, echte Aufnahme, auch ein echtes Cancel).
