# RK-022 Flip and Rotate und Resize Image (Windows zieht nach)

- Status: offen
- Abhängig von: RK-021, MAC-017
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die Karten Flip and Rotate und Resize Image im Panel Adjustments. Braucht RK-021, weil Drehen einen Zuschnitt entfernt, das Zuschneiden
in den Pixeln des gedrehten Bildes arbeitet und View Compare das gedrehte Bild zeigt.

## Umfang

Werte, Reihenfolge (Refine → Farbe → Flip and Rotate → Crop → Resize Image), Projektdatei und Mac-Dateien in
`docs/parity/WINDOWS_PARITY.md`, Abschnitt „Drehen und Größe (Mac: MAC-017)“, Aufbau in `docs/design.md`, „Drehen und Größe (MAC-017)“.

Tauri: `save_image` und `save_image_as` bekommen das Argument `geometry`, dazu ein Befehl für `Session::orient_preview`. Die serde-Typen
fehlen im Kern noch (`Orientation`, `Resize`, `Geometry` brauchen `Deserialize` mit camelCase); diese Kernänderung bekommt nach den
Projektregeln einen eigenen Commit mit Hinweis, weil die Mac-App dieselben Typen benutzt.

## Akzeptanzkriterien

- [ ] Karte Flip and Rotate (Horizontal · Vertical, Left 90° · Right 90° · 180°, Angle, Interpolation, Crop to fit, Reset); Spiegeln kehrt
      das Vorzeichen des Winkels um (Test)
- [ ] Karte Resize Image (Percent · Pixels, Size, Width/Height mit Lock aspect ratio, Method, Result, Reset); Gruppe **Size** im Schritt Save
- [ ] Bildbereich zeigt das gedrehte Bild (auch Before und linke Hälfte von View Compare); Marken des Stacks werden darauf nicht gezeichnet
- [ ] Änderung an Spiegelung, Winkel oder Crop to fit entfernt einen Zuschnitt („Crop removed: the picture was flipped or turned“); bei
      offenem Zuschneiden sind die Bedienelemente von Flip and Rotate gesperrt
- [ ] Projektdatei-Abschnitt `geometry` schreiben und lesen, fehlende Schlüssel bleiben, Zahlen auf Grenzen gekürzt (Test)
- [ ] Gesperrt bleiben nur noch View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph
- [ ] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-022 Flip and Rotate und Resize Image (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen
