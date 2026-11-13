# RK-021 View Compare und Zuschneiden vor dem Speichern (Windows zieht nach)

- Status: offen
- Abhängig von: RK-016, MAC-016, MAC-019
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Zwei Werkzeuge über dem Bild im Schritt Wavelets: die geteilte Ansicht Before/After an einer ziehbaren Linie und das Zuschneiden
vor dem Speichern. Sie gehören zusammen, weil beide Overlays im Bildbereich sind und sich gegenseitig sperren (View Compare ist aus,
solange das Zuschneiden offen ist).

## Umfang

Maße, Texte und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md` und `docs/design.md` („Bildbereich“, „Zuschneiden (Mac-016)“):

- Abschnitt „Adjustments: Color Mixing und View Compare (Mac: MAC-019)“: Punkt **View Compare**
- Abschnitt „Zuschneiden vor dem Speichern (Mac: MAC-016)“: alle drei Punkte

Die Rechteck-Geometrie (Ziehen, feste Formate, Klemmen, Mindestgröße) wird als reine Logik in `app/src/logic/` gebaut und mit den Fällen
aus `mac/Tests/SidereaPlanetsTests/CropRectangleTests.swift` getestet. Tauri: `save_image` und `save_image_as` bekommen das Argument
`crop` (`app/src-tauri/src/export_commands.rs`); der Kern (`CropRect`, `Image::cropped`) liegt auf `main`.

## Akzeptanzkriterien

- [ ] View Compare teilt das Bild an einer ziehbaren Linie (0,02 … 0,98), Beschriftungen Before/After mit 84-px-Regel; Before/After der
      Zoom-Leiste und View Compare schließen sich aus; nur im Schritt Wavelets
- [ ] Knopf **Crop** im Panel Adjustments und Gruppe **Crop** mit **Crop…** und **Remove Crop** im Schritt Save öffnen dasselbe Werkzeug
- [ ] Rechteck mit Abdunklung, Drittel-Raster, Fadenkreuz, acht Griffen und Größenplatte; Formate Free · 1:1 · 3:2 · 4:3, Reset, Cancel,
      Done; Geometrie-Tests aus den Mac-Fällen
- [ ] Zuschnitt gilt für jedes Format von Save Image im Schritt Wavelets und Save, nach den Farb- und Tonwerten, bis zum nächsten Stack
      oder einer anderen Aufnahme; nicht in der Projektdatei
- [ ] Gesperrte Adjustments-Knöpfe nach diesem Ticket: View Zoomed, View Stack Size, Flip and Rotate, Resize Image, Wavelet Filter, Show Line Graph
- [ ] Punkte aus `docs/parity/WINDOWS_PARITY.md` gelöscht (View Compare, Abschnitt MAC-016; mit RK-020 auch der Punkt „Die Zahl der
      gesperrten Adjustments-Knöpfe“ und damit der ganze Abschnitt MAC-019)
- [ ] Probe von Hand als `## RK-021 View Compare und Zuschneiden (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen
