# RK-008 Farbabgleich, RGB-Ausrichtung und Tonwerte

- Status: offen
- Abhängig von: RK-006

## Ziel
Nachbearbeitung wie im Original: Farbkanäle gegeneinander verschieben (Atmosphärendispersion), RGB-Abgleich,
Gamma, Kontrast, Histogramm.

## Akzeptanzkriterien
- [ ] RGB-Ausrichtung automatisch und manuell
- [ ] RGB-Balance, Gamma, Kontrast und Histogramm-Anpassung
- [ ] Vergleich mit dem Original an einem Farbdatensatz

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: rechtes Panel im Wavelet-Reiter (Functions-Raster, Contrast/Brightness), Häkchen Colour und LRGB (`docs/design.md`, Abschnitt Wavelet)
- Soll-Bilder: `docs/design/screenshots/07-wavelet.png`, hell `12-wavelet-light.png`
- Die Panels hinter den Functions-Schaltflächen (Histogram, Gamma, RGB Align, RGB Balance …) haben noch kein Soll-Bild. Form: schwebende Karten wie Framelist; `docs/design.md` im selben Commit ergänzen
