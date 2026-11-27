# SP-006 Stacken

- Status: erledigt
- Abhängig von: SP-005

## Ziel
Aus den ausgewählten Frames ein Summenbild berechnen wie das Original. Version 6 des Referenzprogramms hat keinen eigenen
Optimierungsschritt mehr (Ablauf Set Alignpoints → Align → Limit → Stack, siehe `docs/workflow.md`).
Die lokale Ausrichtung über die Alignpoints passiert beim Stacken selbst.

## Akzeptanzkriterien
- [x] Stacking mit lokaler Korrektur über die nächstgelegenen Alignpoints (*Use Nearestby Alignpoints*, *Correct geometry*)
      mit den Standardwerten aus `docs/ui.md`
- [x] Stacking mit Fließkomma oder 16-Bit-Genauigkeit ohne Überlauf
- [x] Ergebnis gegen das Original verglichen (Differenzbild, PSNR; Referenz und Schwelle in `docs/testdaten.md`) und in `docs/` festgehalten

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Stack-Reiter komplett, Karte Alignmentpoints, Zusatzfelder für Resampling und Drizzling (`docs/design.md`, Abschnitte Stack und Zustände)
- Soll-Bilder: `docs/design/screenshots/05-stack-limited.png`, `06-stack-stacked.png`

## Stand
Rechenkern (`crates/core/src/stack.rs`), Tauri-Befehle, Anbindung des Stack-Reiters (`app/src/stack.ts`), Vergleichstest und
Doku (`docs/stacking.md`) sind fertig. Vergleich mit dem Original: 54,85 dB gegen `jupiter-stack.tif` (Schwelle 45 dB). Das gilt
mit dem Vertrauen in die Punktversätze: Bei R1 streuen die Zuordnungen der Alignpoints unzusammenhängend, deshalb entspricht das
Ergebnis dort dem Stack mit Frameversatz. Die lokale Korrektur selbst ist an einer künstlichen Verzerrung geprüft; ob das
Original bei R1 lokal korrigiert, ist offen (`docs/stacking.md`). Save Aligned und Save Integrated (Create AVI) gehören zu
SP-009, Resampling und Drizzling sind nicht umgesetzt. Die Häkchen von Resampling und Drizzling sind in der App gesperrt und mit „Not available yet“ gekennzeichnet (UI-Prüflauf: eingeschaltet wirkten sie nicht). Ebenso gesperrt sind StackSize Minimized, Maximized und R.of Interest, Stretch histogram after stacking und De-rotate images: Sie gehen nicht in die Anfrage an das Backend ein (`StackRequest`). Es wirken Use Nearestby Alignpoints, Correct geometry, Normalization of frame intensity und Do not stack n pixels at the frameedge. Test: `app/src/docs/stack-pages.test.ts` (Panel und Doku-Seite).
