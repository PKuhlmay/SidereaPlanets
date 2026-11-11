# RK-006 Stacken

- Status: in Arbeit
- Abhängig von: RK-005

## Ziel
Aus den ausgewählten Frames ein Summenbild berechnen wie das Original. the reference program hat keinen eigenen
Optimierungsschritt mehr (Ablauf Set Alignpoints → Align → Limit → Stack, siehe `docs/workflow.md`).
Die lokale Ausrichtung über die Alignpoints passiert beim Stacken selbst.

## Akzeptanzkriterien
- [ ] Stacking mit lokaler Korrektur über die nächstgelegenen Alignpoints (*Use Nearestby Alignpoints*, *Correct geometry*)
      mit den Standardwerten aus `docs/ui.md`
- [ ] Stacking mit Fließkomma oder 16-Bit-Genauigkeit ohne Überlauf
- [ ] Ergebnis gegen das Original verglichen (Differenzbild, PSNR; Referenz und Schwelle in `docs/testdaten.md`) und in `docs/` festgehalten

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Stack-Reiter komplett, Karte Alignmentpoints, Zusatzfelder für Resampling und Drizzling (`docs/design.md`, Abschnitte Stack und Zustände)
- Soll-Bilder: `docs/design/screenshots/05-stack-limited.png`, `06-stack-stacked.png`

## Stand
Rechenkern (`crates/core/src/stack.rs`), Tauri-Befehle (`app/src-tauri/src/stack_commands.rs`), Vergleichstest
(`crates/io/tests/stack_reference.rs`) und Doku (`docs/stacking.md`, Seite *Stack* in der App) sind geschrieben und
kompilieren. Die Kriterien bleiben offen, weil keine Tests und kein Vergleich gelaufen sind (Testpause seit 2026-11-10):
Kriterium 1 und 2 brauchen grüne Tests, Kriterium 3 die Messwerte. Der Stack-Reiter ist angebunden (`app/src/stack.ts`: Stack-Button, Fortschritt, Cancel/Pause,
Ergebnis in `session.stackImage`); im Fenster ausprobiert wurde es nicht. Save Aligned und Save Integrated (Create AVI) gehören
zu RK-009, Resampling und Drizzling sind nicht umgesetzt.
