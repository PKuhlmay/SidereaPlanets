# MAC-011 Go to best frame

- Status: erledigt
- Abhängig von: MAC-006

## Ziel

Nach Align soll man den besten Frame mit einem Klick als Referenz für einen neuen Set Alignpoints wählen können. Bisher führt der
Weg über Show Framelist, das Ablesen der Nummer und das Feld Goto Frame (Philipp, 2026-11-12). Die Funktion ist neu und gibt es im
Original nicht; der Mac führt, Windows zieht über `docs/parity/WINDOWS_PARITY.md` nach.

## Akzeptanzkriterien

- [x] Schaltfläche **Go to best frame** im Schritt Limit (bei den Ergebnissen bzw. unter dem Qualitätsdiagramm) und in der Karte
      Framelist; gesperrt, solange Align nicht gelaufen ist. Sie zeigt den besten Frame (erster Platz der Rangliste) im Bildbereich
      und in der Frame-Leiste und öffnet den Schritt Alignpoints, damit Set Alignpoints darauf aufsetzt
- [x] Ein Hinweis im Panel nennt die Nummer („Best frame: 295“) und dass Set Alignpoints und Align erneut laufen müssen
- [x] Die Ergebnisse werden dabei nicht verworfen; erst Set Alignpoints verwirft sie wie bisher
- [x] Modell-Tests (Attrappe), einmal rot an einer Assertion
- [x] Anwenderdoku in `app/src/docs/pages/` (gemeinsam) beschreibt den Weg zum besten Referenzframe
- [x] Punkt in `docs/parity/WINDOWS_PARITY.md` mit Texten, Ort und Verhalten

## Umsetzung

- Modell: `AppModel.goToBestFrame()` und `canGoToBestFrame` (`mac/App/Model/AppModel+Align.swift`): erster Platz der Rangliste (`AlignData.bestFrame`,
  bei gleicher Qualität die kleinere Nummer) wird mit `showFrame` in Bild und Frame-Leiste gezeigt, danach `show(.alignpoints)`. Gesperrt ohne
  Bewertung (vor Align) und während einer Berechnung. `AlignData.bestFrameHint` merkt die Frame-Nummer; `discardRating` (Set Alignpoints
  und Align) und ein neues Öffnen löschen sie. Ergebnisse, Auswahl der Frames, Limit und Stack bleiben unberührt.
- Hinweis: `bestFrameHint(frame:)` in `AlignLogic.swift` (Nummer ab 1), gezeigt als Hinweis-Block (`--accent-soft`, Streifen `--accent`) über dem
  Threshold-Regler im Schritt Alignpoints (`AlignPanels.swift`).
- Schaltfläche in Standardstil unter dem Qualitätsdiagramm (Schritt Limit) und als zweite Zeile der Karte Framelist (`FloatingCards.swift`).
- Tests: `AlignModelTests` (Abschnitt Go to best frame, Attrappe) und `AlignLogicTests`; einmal rot an den Assertions, als `bestFrame` das
  letzte statt das erste der Rangliste lieferte (acht Assertions fielen durch), danach wiederhergestellt.
- Doku: Abschnitt „Use the best frame as reference“ in `app/src/docs/pages/frame-quality-and-limit.md`, Verweise in `align-your-frames.md` und
  `steps-alignpoints-align-limit.md`. Windows zieht über `docs/parity/WINDOWS_PARITY.md` nach.
