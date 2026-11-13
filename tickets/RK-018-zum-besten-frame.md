# RK-018 Go to Best Frame (Windows zieht nach)

- Status: offen
- Abhängig von: RK-016, MAC-011
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Nach Align lässt sich der beste Frame mit einem Klick als Referenz für ein neues Set Alignpoints nehmen, wie in der Mac-App.

## Umfang

Verhalten, Maße, Sperren und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „Go to best frame (MAC-011)“. Namen und Texte
gelten in der Fassung von MAC-014 (Tabelle 1 im Abschnitt „Verständliche Texte für Einsteiger“): Knopf **Go to Best Frame**, Karte
**Frame List** mit Check All · Check None · Clear List, Hinweis „Best frame: N. Run Set Alignpoints and Align again to line all frames
up on this one.“

## Akzeptanzkriterien

- [ ] Knopf im Panel des Schritts Best Frames unter dem Qualitätsdiagramm und in der Karte Frame List unter der Knopfzeile; gesperrt
      vor Align („Run Align first“) und während einer Berechnung
- [ ] Klick zeigt den ersten Frame der Rangliste (höchste Qualität, bei Gleichstand kleinere Nummer) und öffnet Alignpoints; nichts
      wird verworfen (Test)
- [ ] Hinweis über dem Threshold-Regler, bis Set Alignpoints, Align oder ein neues Öffnen ihn löscht (Test)
- [ ] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-018 Go to Best Frame (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen
