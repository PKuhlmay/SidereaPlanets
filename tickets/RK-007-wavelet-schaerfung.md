# RK-007 Wavelet-Schärfung

- Status: offen
- Abhängig von: RK-006

## Ziel
Das Herzstück von the reference program: mehrschichtige Wavelet-Zerlegung mit Reglern je Ebene für Schärfen und Entrauschen.

## Akzeptanzkriterien
- [ ] Zerlegung in Ebenen mit Verstärkung und Rauschunterdrückung je Ebene (Anzahl und Details laut `docs/`)
- [ ] Vorschau reagiert flüssig auf Reglerbewegung (Maßnahmen und Messwerte: `docs/gui-toolkit.md`, Abschnitt Spike: Vorschau auf Ausschnitt begrenzen, RGBA-Umwandlung mit `rayon`, Übertragung neu messen)
- [ ] Ergebnis bei gleichen Reglerwerten gegen das Original verglichen (PSNR-Schwelle in `docs/` festlegen)
- [ ] Einstellungen speicher- und ladbar

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: linkes Panel im Wavelet-Reiter mit den sechs Ebenen, **Process**, **Do All**, Umschalter **Before**, Processing Area (`docs/design.md`, Abschnitte Wavelet, Regler, Bildbereich)
- Soll-Bilder: `docs/design/screenshots/07-wavelet.png`, `08-wavelet-sharpened.png`, hell `12-wavelet-light.png`
