# SP-021 View Compare und Zuschneiden vor dem Speichern (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-016, MAC-016, MAC-019
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

- [x] View Compare teilt das Bild an einer ziehbaren Linie (0,02 … 0,98), Beschriftungen Before/After mit 84-px-Regel; Before/After der
      Zoom-Leiste und View Compare schließen sich aus; nur im Schritt Wavelets
- [x] Knopf **Crop** im Panel Adjustments und Gruppe **Crop** mit **Crop…** und **Remove Crop** im Schritt Save öffnen dasselbe Werkzeug
- [x] Rechteck mit Abdunklung, Drittel-Raster, Fadenkreuz, acht Griffen und Größenplatte; Formate Free · 1:1 · 3:2 · 4:3, Reset, Cancel,
      Done; Geometrie-Tests aus den Mac-Fällen
- [x] Zuschnitt gilt für jedes Format von Save Image im Schritt Wavelets und Save, nach den Farb- und Tonwerten, bis zum nächsten Stack
      oder einer anderen Aufnahme; nicht in der Projektdatei
- [x] Gesperrte Adjustments-Knöpfe nach diesem Ticket: View Zoomed, View Stack Size, Flip and Rotate, Resize Image, Wavelet Filter, Show Line Graph
- [x] Punkte aus `docs/parity/WINDOWS_PARITY.md` gelöscht (View Compare, Abschnitt MAC-016; mit SP-020 auch der Punkt „Die Zahl der
      gesperrten Adjustments-Knöpfe“ und damit der ganze Abschnitt MAC-019)
- [x] Probe von Hand als `## SP-021 View Compare und Zuschneiden (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen

## Umsetzung

- Geometrie: `app/src/logic/crop.ts`, Port von `mac/App/Model/CropRectangle.swift` (Rundung halb weg von null wie Swift); alle Fälle aus
  `CropRectangleTests.swift` stehen in `crop.test.ts`.
- Zustand und Regeln: `app/src/cropTool.ts` (View Compare an/aus, Before/After schließt den Vergleich, Werkzeug öffnen auf 80 % der Mitte
  oder dem angewendeten Rechteck, Done/Cancel/Reset/Remove Crop, Format umformen, ein Rechteck über das ganze Bild ist keins). Before/After
  liegt dafür jetzt in `session.before`, die Lage der Trennlinie in `session.compareDivider` (nicht in der Projektdatei), das Werkzeug in
  `session.crop`. Ein neuer Stack, Set Alignpoints/Align oder eine neue Aufnahme löschen den Zuschnitt; ein Wechsel aus Wavelets/Save
  schließt das Werkzeug.
- Bildbereich (`ImageViewer.vue`): das Summenbild als zweites Canvas, an der Trennlinie per `clip-path` abgeschnitten;
  `CompareOverlay.vue` (Linie, Griff, Before/After mit der 84-px-Regel, Pfeiltasten am Griff); `CropOverlay.vue` (Abdunklung, Rand,
  Drittel, Fadenkreuz, acht Griffe, Größenplatte) in Bildschirmpixeln; Ziehen, Griffe und Neuzeichnen mit passendem Mauszeiger.
- Karte `CropCard.vue` (Format Free · 1:1 · 3:2 · 4:3, Size, Reset, Cancel, Done), Gruppe **Crop** mit **Crop…** und **Remove Crop** im
  Schritt Save (`SaveStep.vue`). Der Adjustments-Knopf heißt **Crop**; gesperrt bleiben sechs Knöpfe.
- Speichern: `save_image` und `save_image_as` nehmen `crop` (`CropArea` in `app/src-tauri/src/export_commands.rs`, wird zu `CropRect` des
  Kerns); `saveImage` schickt den Zuschnitt nur für das geschärfte Bild.
- Hilfeseite `color-and-tones.md`: der Hinweis, dass Windows View Compare nicht hat, ist gestrichen.
