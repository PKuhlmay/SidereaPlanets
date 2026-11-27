# SP-008 Farbabgleich, RGB-Ausrichtung und Tonwerte

- Status: in Arbeit
- Abhängig von: SP-006

## Ziel
Nachbearbeitung wie im Original: Farbkanäle gegeneinander verschieben (Atmosphärendispersion), RGB-Abgleich,
Gamma, Kontrast, Histogramm.

## Akzeptanzkriterien
- [ ] RGB-Ausrichtung automatisch und manuell
- [ ] RGB-Balance, Gamma, Kontrast und Histogramm-Anpassung
- [ ] Vergleich mit dem Original an einem Farbdatensatz

## Stand

Rechenkern und Tauri-Befehle stehen (`crates/core/src/{color,tone}.rs`, `app/src-tauri/src/color_commands.rs`), dazu die
Karten Histogram, Gamma, RGB Align und RGB Balance im Wavelet-Reiter, Contrast/Brightness, Copy To / Load to / Difference,
Colour/LRGB-Sperren, Anwenderdoku (`colour-and-tones`) und die Karten in `docs/design.md`. `pnpm build`, `cargo clippy` und
`cargo fmt` laufen sauber; die Tests (Rust und Vitest) sind geschrieben, aber noch nicht gelaufen, die Oberfläche noch nicht
gesehen. Offen: der Vergleich mit dem Original, für den es keine Referenzergebnisse gibt (`docs/testdaten.md`), und die Wirkung
von Colour/LRGB auf Do All (SP-007). Deshalb ist noch kein Kriterium abgehakt: Es fehlen grüne Tests und eine Sichtprüfung.

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: rechtes Panel im Wavelet-Reiter (Functions-Raster, Contrast/Brightness), Häkchen Colour und LRGB (`docs/design.md`, Abschnitt Wavelet)
- Soll-Bilder: `docs/design/screenshots/07-wavelet.png`, hell `12-wavelet-light.png`
- Die Panels hinter den Functions-Schaltflächen (Histogram, Gamma, RGB Align, RGB Balance …) haben noch kein Soll-Bild. Form: schwebende Karten wie Framelist; `docs/design.md` im selben Commit ergänzen
