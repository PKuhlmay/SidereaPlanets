# RK-006 Stacken

- Status: offen
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
