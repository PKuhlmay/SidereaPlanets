# RK-007 Wavelet-Schärfung

- Status: offen
- Abhängig von: RK-006

## Ziel
Das Herzstück von the reference program: mehrschichtige Wavelet-Zerlegung mit Reglern je Ebene für Schärfen und Entrauschen.

## Akzeptanzkriterien
- Zerlegung in Ebenen mit Verstärkung und Rauschunterdrückung je Ebene (Anzahl und Details laut `docs/`)
- Vorschau reagiert flüssig auf Reglerbewegung
- Ergebnis bei gleichen Reglerwerten gegen das Original verglichen (PSNR-Schwelle in `docs/` festlegen)
- Einstellungen speicher- und ladbar
