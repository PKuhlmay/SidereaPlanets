# RK-016 Verständliche Texte für Einsteiger (Windows zieht nach)

- Status: erledigt
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

- [x] Alle Zeilen aus Tabelle 1, die die Windows-Oberfläche betreffen, sind umgesetzt (`app/src/components/*.vue`, `app/src/logic/*.ts`,
      `app/src/App.vue`, `app/src/wavelet.ts`); Projektdatei-Schlüssel und Bezeichner bleiben
- [x] Tooltips aus Tabelle 2 stehen an Schritten, Aktionsknöpfen, Feldern und Optionen; kein Tooltip trägt einen „the reference program: …“-Zusatz
- [x] Bedienelemente aus Tabelle 3 sind gesperrt (gedimmt 45 %, nicht klickbar, Tooltip beginnt mit „Not available yet“); Show-Menü mit
      Abschnittskopf „Not available yet“
- [x] Ergebniszeile des Schritts Align zeigt „‹n› frames rated“ (Test, der vorher an „best quality“ rot war)
- [x] Knopf **Select Recording…** im Fuß des Schritts Open und in der Werkzeugleiste, Menü **Recent Recordings**
- [x] `git grep -n -i "the reference program\|the original" -- app/src app/src-tauri/src` trifft nur Kommentare, Bezeichner und Tests
- [x] Die drei Abschnitte sind aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## RK-016 Texte für Einsteiger (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen

## Umsetzung

- Schrittlogik (`app/src/logic/workflow.ts`): neue Namen, Sätze, `explanation` und `actionExplanation` je Schritt (Tooltips der Pillen und
  Aktionsknöpfe), `againLabel` („‹Aktion› Again“, Open: „Select Another Recording…“), Ergebniszeilen („82 points“, „10 frames rated“,
  „8 of 10 frames kept“, „Sharpening applied“). Statusmeldungen in `align.ts`, `stack.ts` (`stackMessages`), `wavelet.ts`, `colour.ts`,
  `saving.ts`, `recording.ts`; Fehler einheitlich über `failureText` aus `jobs.ts` („‹Schritt› failed: ‹Grund›“). Die Läufe heißen jetzt
  Keep Best Frames, Do All, Update Preview, Save Aligned Video und Save Averaged Video (Fenstertitel, Cancel/Pause-Zuordnung in `jobs.ts`).
- Show-Menü aus `showItems` (`app/src/logic/menu.ts`), die wirkungslosen Einträge gedimmt unter „Not available yet“. Adjustments-Knöpfe:
  `isFunctionBuilt` und `functionHelp` (`app/src/logic/colour.ts`), elf gesperrt.
- Tabelle 3 (g): *Hold* hatte in Windows keine Wirkung (`holdContrast` wird nur gespeichert), *Keep* ist deshalb gesperrt.
- Regel 6 über die Tabellen hinaus, mit Philipp abgestimmt (2026-11-13): Die Menüs *Flat/Dark/Reference* und *Tools* sind aus der
  Werkzeugleiste entfernt (kein Eintrag hatte eine Wirkung, und mit den längeren Namen passte die Leiste bei 1120 px nicht mehr); im Menü
  *Settings* wirken nur Preview Area, Theme und Show Advanced Controls, die übrigen Einträge stehen gedimmt unter „Not available yet“.
  About ist gesperrt. Die Werte bleiben in `settings.menu`.
- Weil der Satz im Schritt Open und der Text im leeren Bildbereich das Ablegen von Dateien nennen, öffnet die Windows-App jetzt auch
  Dateien, die auf das Fenster gezogen werden (`openDroppedFiles` in `recording.ts`, Test in `flow.test.ts`); während einer Berechnung
  passiert nichts.
- Das Feld CPUs in der Werkzeugleiste behält die kurze Beschriftung (Platz), Tooltip und Hilfeseite nennen „CPU cores to use“.
- Lange Feldnamen im Schritt Alignpoints („Min. distance from image edge (px)“) brechen im 272-px-Panel auf zwei Zeilen um; im Schritt
  Best Frames steht das Feld in der Zeile seines Modus wie am Mac.
- Hilfeseiten `menus-and-settings.md` und `window-at-a-glance.md` an die Werkzeugleiste ohne die beiden Menüs angepasst.
