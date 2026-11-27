# SP-018 Go to Best Frame (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-016, MAC-011
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Nach Align lässt sich der beste Frame mit einem Klick als Referenz für ein neues Set Alignpoints nehmen, wie in der Mac-App.

## Umfang

Verhalten, Maße, Sperren und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „Go to best frame (MAC-011)“. Namen und Texte
gelten in der Fassung von MAC-014 (Tabelle 1 im Abschnitt „Verständliche Texte für Einsteiger“): Knopf **Go to Best Frame**, Karte
**Frame List** mit Check All · Check None · Clear List, Hinweis „Best frame: N. Run Set Alignpoints and Align again to line all frames
up on this one.“

## Akzeptanzkriterien

- [x] Knopf im Panel des Schritts Best Frames unter dem Qualitätsdiagramm und in der Karte Frame List unter der Knopfzeile; gesperrt
      vor Align („Run Align first“) und während einer Berechnung
- [x] Klick zeigt den ersten Frame der Rangliste (höchste Qualität, bei Gleichstand kleinere Nummer) und öffnet Alignpoints; nichts
      wird verworfen (Test)
- [x] Hinweis über dem Threshold-Regler, bis Set Alignpoints, Align oder ein neues Öffnen ihn löscht (Test)
- [x] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## SP-018 Go to Best Frame (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen

## Umsetzung

- Rangliste und bester Frame in `app/src/logic/align.ts` (`rankedFrames`, `bestFrame`, `bestFrameHint`, `goToBestFrameHelp`); die Frame List
  sortiert jetzt mit derselben Rangliste (bei gleicher Qualität die kleinere Nummer zuerst).
- `goToBestFrame` und `canGoToBestFrame` in `app/src/align.ts`: zeigt den Frame, öffnet Alignpoints und merkt die Nummer in
  `session.bestFrameHint`; `discardResults` (Set Alignpoints, Align) und `startSession` (neue Aufnahme) löschen sie. Nichts sonst wird
  verworfen.
- Knopf im Panel Best Frames unter dem Diagramm (volle Breite, 8 px Abstand) und in der Karte Frame List als zweite Zeile (24 px, 12 px
  Text); Hinweis im Panel Alignpoints über dem Threshold-Regler (`AlignpointsStep.vue`, Hinweis-Komponente aus `docs/design.md`).
- Die Hilfeseiten beschrieben die Funktion schon und brauchten keine Änderung.
