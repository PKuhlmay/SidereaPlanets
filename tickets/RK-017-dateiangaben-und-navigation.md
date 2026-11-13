# RK-017 Dateiangaben im Schritt Open und Navigation per Tastatur (Windows zieht nach)

- Status: erledigt
- Abhängig von: RK-016, MAC-005, MAC-010
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Drei kleine Bedienhilfen der Mac-App kommen nach Windows: die vollständigen Dateiangaben im Panel des Schritts Open, vorheriger und
nächster Frame in der Frame-Leiste und die sieben Schritte per Tastenkürzel.

## Umfang

Werte, Texte und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „v0.1.0 (Mac, in Arbeit)“:

- „Schritt Open: Dateiangaben (MAC-005)“ (Zeilen mit den Namen aus RK-016, z. B. *Color*, Tooltips an Codec und Bit depth)
- „Frame-Leiste: vorheriger und nächster Frame (MAC-005)“
- „Schritte per Tastatur (MAC-010)“

Die Windows-Belegung legt dieses Ticket fest (Philipp, 2026-11-13): ← und → für den vorherigen und nächsten Frame, solange kein
Eingabefeld den Fokus hat; Alt+1 bis Alt+7 für die Schritte. Die Belegung kommt in die Tooltips und in die Anwenderseiten (`app/src/docs/pages/`), wo
sie bisher nur die Mac-Kürzel nennen.

## Akzeptanzkriterien

- [x] Panel Open zeigt Format, Codec, Size, Bit depth, Frames, Color, Frame rate und, wo vorhanden, Observer, Instrument, Telescope
- [x] Schaltflächen vorheriger/nächster Frame zwischen Go to frame und Regler, am ersten bzw. letzten Frame gesperrt, Tooltip mit Kürzel
- [x] Kürzel für die Schritte 1 bis 7 wirken wie ein Klick auf die Schrittleiste; gesperrter Schritt und laufende Berechnung sperren sie (Test)
- [x] Anwenderseiten nennen die Windows-Kürzel
- [x] Die drei Abschnitte sind aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## RK-017 Dateiangaben und Navigation (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen

## Umsetzung

- Dateiangaben: `recordingRows`, `formatRate`, `rowExplanation` in `app/src/logic/recordingDetails.ts` (Regeln wie
  `mac/App/Model/RecordingDetails.swift`, Tests aus `RecordingDetailsTests.swift` übernommen), im Panel als Liste aus Beschriftung und Wert
  (`OpenStep.vue`, 12 px, Beschriftung `--muted`).
- Frame-Leiste: zwei randlose Pfeil-Schaltflächen zwischen Go to frame und Regler, am ersten bzw. letzten Frame gesperrt, Tooltip
  „Previous frame (←)“ / „Next frame (→)“; `showNextFrame` in `recording.ts`.
- Tastatur: `keyCommand` (`app/src/logic/keys.ts`) entscheidet, was eine Taste bedeutet; `shortcuts.ts` setzt es um (App.vue hört auf
  `keydown`). ← und → wirken nur, solange die Frame-Leiste da ist und kein Feld, Regler, Radio oder eine Liste den Fokus hat. Alt+1 bis
  Alt+7 öffnen einen Schritt wie ein Klick auf die Pille (gesperrter Schritt und laufende Berechnung sperren); die Ziffer wird über
  `KeyboardEvent.code` erkannt (auch Ziffernblock) und nie an das Fenstermenü von Windows weitergereicht.
- Der Tooltip jeder Pille nennt in der letzten Zeile den Kürzel („Shortcut: Alt+2“); Windows hat keine Menüleiste, die ihn zeigt.
- Hilfeseiten: `steps-alignpoints-align-limit.md` (Frame-Leiste), `window-at-a-glance.md` (Schrittleiste), `select-a-recording.md`
  (Dateiangaben).
