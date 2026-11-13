# RK-016 Verständliche Texte für Einsteiger (Windows zieht nach)

- Status: offen
- Abhängig von: MAC-014
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die Windows-App bekommt dieselben Namen, Tooltips und Sperren wie die Mac-App seit MAC-014. Das Ticket kommt vor allen anderen des
Nachzugs, weil deren Parity-Punkte schon die neuen Namen benutzen (Best Frames, Frame List, Go to Best Frame, Adjustments …).

## Umfang

Alle Werte, Texte und Dateien stehen in `docs/parity/WINDOWS_PARITY.md`, nicht hier:

- Abschnitt „Verständliche Texte für Einsteiger (MAC-014, …)“: Regeln (1) bis (6), Tabelle 1 (Umbenennungen), Tabelle 2 (Tooltips),
  Tabelle 3 (Bedienelemente ohne Wirkung sperren), Sachfehler der Align-Ergebniszeile, the reference program-Erwähnungen in Windows-Texten.
- Abschnitt „Knopf „Select Recording…““.
- Abschnitt „Menü „Recent Recordings““.

Die Mac-Tastenkürzel und Menüleisten-Zeilen der Tabelle 1 (Menü File, Run, View, About) gelten für Windows nicht; Windows übernimmt
davon nur, was in der Werkzeugleiste steht. Tabelle 3 (g): prüfen, ob *Hold* in Windows wirkt, sonst *Keep* ebenfalls sperren; das
Ergebnis kommt unter „Umsetzung“ in dieses Ticket.

## Akzeptanzkriterien

- [ ] Alle Zeilen aus Tabelle 1, die die Windows-Oberfläche betreffen, sind umgesetzt (`app/src/components/*.vue`, `app/src/logic/*.ts`,
      `app/src/App.vue`, `app/src/wavelet.ts`); Projektdatei-Schlüssel und Bezeichner bleiben
- [ ] Tooltips aus Tabelle 2 stehen an Schritten, Aktionsknöpfen, Feldern und Optionen; kein Tooltip trägt einen „the reference program: …“-Zusatz
- [ ] Bedienelemente aus Tabelle 3 sind gesperrt (gedimmt 45 %, nicht klickbar, Tooltip beginnt mit „Not available yet“); Show-Menü mit
      Abschnittskopf „Not available yet“
- [ ] Ergebniszeile des Schritts Align zeigt „‹n› frames rated“ (Test, der vorher an „best quality“ rot war)
- [ ] Knopf **Select Recording…** im Fuß des Schritts Open und in der Werkzeugleiste, Menü **Recent Recordings**
- [ ] `git grep -n -i "the reference program\|the original" -- app/src app/src-tauri/src` trifft nur Kommentare, Bezeichner und Tests
- [ ] Die drei Abschnitte sind aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-016 Texte für Einsteiger (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen
