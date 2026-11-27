# SP-022 Flip and Rotate und Resize Image (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-021, MAC-017
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die Karten Flip and Rotate und Resize Image im Panel Adjustments. Braucht SP-021, weil Drehen einen Zuschnitt entfernt, das Zuschneiden
in den Pixeln des gedrehten Bildes arbeitet und View Compare das gedrehte Bild zeigt.

## Umfang

Werte, Reihenfolge (Refine → Farbe → Flip and Rotate → Crop → Resize Image), Projektdatei und Mac-Dateien in
`docs/parity/WINDOWS_PARITY.md`, Abschnitt „Drehen und Größe (Mac: MAC-017)“, Aufbau in `docs/design.md`, „Drehen und Größe (MAC-017)“.

Tauri: `save_image` und `save_image_as` bekommen das Argument `geometry`, dazu ein Befehl für `Session::orient_preview`. Die serde-Typen
fehlen im Kern noch (`Orientation`, `Resize`, `Geometry` brauchen `Deserialize` mit camelCase); diese Kernänderung bekommt nach den
Projektregeln einen eigenen Commit mit Hinweis, weil die Mac-App dieselben Typen benutzt.

## Akzeptanzkriterien

- [x] Karte Flip and Rotate (Horizontal · Vertical, Left 90° · Right 90° · 180°, Angle, Interpolation, Crop to fit, Reset); Spiegeln kehrt
      das Vorzeichen des Winkels um (Test)
- [x] Karte Resize Image (Percent · Pixels, Size, Width/Height mit Lock aspect ratio, Method, Result, Reset); Gruppe **Size** im Schritt Save
- [x] Bildbereich zeigt das gedrehte Bild (auch Before und linke Hälfte von View Compare); Marken des Stacks werden darauf nicht gezeichnet
- [x] Änderung an Spiegelung, Winkel oder Crop to fit entfernt einen Zuschnitt („Crop removed: the picture was flipped or turned“); bei
      offenem Zuschneiden sind die Bedienelemente von Flip and Rotate gesperrt
- [x] Projektdatei-Abschnitt `geometry` schreiben und lesen, fehlende Schlüssel bleiben, Zahlen auf Grenzen gekürzt (Test)
- [x] Gesperrt bleiben nur noch View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph
- [x] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## SP-022 Flip and Rotate und Resize Image (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen

## Umsetzung

- **Kein Kern-Commit nötig:** Statt `Deserialize` an `Orientation`, `Resize` und `Geometry` im Kern hat die Tauri-Shell eigene
  Übergabetypen (`app/src-tauri/src/geometry_commands.rs`: `GeometryArea`, `OrientationArea`, `ResizeArea`, camelCase), die in die Typen
  des Kerns umgewandelt werden; der gemeinsame Kern und damit die Mac-App bleiben unberührt (wie `CropArea` aus SP-021). Dort auch der Befehl
  `orient_preview` (rohes RGBA im Body, `width`, `height` und die Ausrichtung als JSON im Header, ruft `Session::orient_preview`);
  `save_image` und `save_image_as` nehmen `geometry`.
- **Werte und Größen** (`app/src/logic/geometry.ts`): Port der Regeln aus `GeometryLogic.swift` (Winkel in Zehnteln auf −180 … 180,
  Spiegeln kehrt das Vorzeichen um, Lock aspect ratio mit zuletzt getippter Seite, Abschnitt `geometry` der Projektdatei). Die Größen
  (`orientedSize`, `resizeSize`) folgen `Orientation::output_size` und `Resize::output_size` des Kerns, damit die Karten die Größe beim
  Ziehen sofort zeigen; die Testwerte stammen aus den Tests in `crates/core/src/geometry.rs`.
- **Ablauf** (`app/src/geometryView.ts`): Bildgröße nach dem Drehen (darin liegt das Zuschnitt-Rechteck, `cropTool.cropImageSize`),
  Größe der Datei nach Zuschnitt und Resize, was Save Image schickt, `editOrientation` (gesperrt bei offenem Zuschneiden, „Crop removed:
  the picture was flipped or turned“ bei einer Änderung von Spiegelung, Winkel oder Crop to fit) und `orientImage` für die gedrehte
  Anzeige des gezeigten Bildes und der linken Hälfte von View Compare. Die Marken des Stacks fallen auf einem gedrehten Bild weg.
- **Oberfläche:** Karten `FlipRotateCard.vue` und `ResizeCard.vue`, Gruppe **Size** im Schritt Save; gesperrt bleiben vier Knöpfe.
- Hilfeseite `color-and-tones.md`: die Sätze „the Windows version … ignores them“ sind gestrichen. Damit ist `docs/parity/WINDOWS_PARITY.md`
  für `windows-v0.1.0` leer.
