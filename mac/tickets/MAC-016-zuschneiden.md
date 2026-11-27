# MAC-016 Zuschneiden vor dem Speichern

- Status: erledigt
- Abhängig von: MAC-014

## Ziel

Philipp (2026-11-12): „Am Ende beim Speichern brauche ich die Möglichkeit, das Bild zuzuschneiden, in den Formaten 1:1, 3:2, 4:3 und
frei, mit einem kleinen Fadenkreuz in der Mitte wie bei Siderea. Im Schritt Wavelets gibt es die Option schon, aber sie ist gesperrt.“

Die gesperrte Schaltfläche **Cropping Area** im Panel Adjustments (von MAC-014 mit „Not available yet“ gesperrt) heißt jetzt **Crop** und
öffnet ein Werkzeug; dasselbe Werkzeug öffnet **Crop…** im Schritt Save über Save Image. Aussehen und Verhalten wie das Zuschneiden von
Siderea (`Stackr/App/Views/ZoomableImageView.swift` `CropOverlayView`, `SidereaCore/CropRectangle.swift`), übernommen und angepasst.

## Akzeptanzkriterien

- [x] Rechenkern: `planets_core::crop::CropRect` (`clamp`, `is_full`), `Image::cropped`; `Session::save_image` nimmt `crop: Option<CropRect>`,
      schneidet zuletzt aus (Reihenfolge Denoise/Deringing und Masking, Farb- und Tonwerte, Zuschnitt); ein Rechteck ohne Pixel im Bild ist ein Fehler und schreibt nichts. FFI-Record `CropRect`;
      die Tauri-Befehle reichen `None` durch und bauen weiter
- [x] Rust-Tests: Klemmen des Rechtecks (innen, über den Rand, ohne Pixel, Überlauf), Ausgabegröße, Pixelgleichheit des Ausschnitts
      (1 und 3 Kanäle), Zuschnitt nach den Farbwerten, Zuschnitt außerhalb des Bildes, ganzes Bild; echte Kette mit `JUPITER.AVI`
      (600 × 450 Ausschnitt gleich dem Ausschnitt der ganzen Datei); FFI-Umwandlung
- [x] Rechteck über dem Bild wie in Siderea: abgedunkelte Außenseite, Rand, Drittel-Raster, **kleines Fadenkreuz in der Mitte**, acht Griffe,
      Größe „W × H“; Verschieben, Größe ändern an Kante und Ecke, neues Rechteck ziehen; nie außerhalb des Bildes, mindestens 8 × 8 Pixel
- [x] Formate **Free, 1:1, 3:2, 4:3**: ein festes Format bleibt beim Ziehen erhalten (liegend oder stehend, wie das Rechteck ist), ein
      anderes Format formt das Rechteck um; Zahlenanzeige „W × H px“; Reset, Cancel, Done
- [x] **Crop** im Panel Adjustments ist nicht mehr gesperrt und ist an, solange das Werkzeug offen ist; **Crop…** im Schritt Save öffnet
      dasselbe Werkzeug; **Remove Crop** nimmt den Zuschnitt weg
- [x] Save Image schreibt in den Schritten Wavelets und Save den Zuschnitt, in jedem Format; das Stack-Bild aus dem Schritt Stack wird
      immer ganz geschrieben
- [x] Der Zuschnitt gilt bis zum nächsten Stack oder einer anderen Aufnahme (dann gelöscht), übersteht das Wechseln zwischen Wavelets und
      Save und das Speichern; verlässt man die beiden Schritte, schließt das Werkzeug
- [x] Swift-Tests rot zuerst: Formate halten das Seitenverhältnis, Klemmen, Reset, Done, Cancel, Dauer des Zuschnitts, der Aufruf von Save
      Image mit dem Rechteck, echter Kern mit `JUPITER.AVI` (TIFF und PNG 600 × 450, ohne Zuschnitt 1024 × 680)
- [x] Texte nach den Regeln von MAC-014 (US-Englisch, Title Case auf Knöpfen, „…“ bei Knöpfen mit Dialog, kein Name des Referenzprogramms)
- [x] Anwenderdoku in der App (`save-and-export.md`, `step-wavelets-save.md`, `color-and-tones.md`, Glossar), `docs/design.md` und
      `docs/parity/WINDOWS_PARITY.md`
- [x] Ein zugeschnittenes TIFF aus der echten Kette gespeichert und die Größe geprüft (600 × 450)

## Umsetzung

**Kern.** `crates/core/src/crop.rs`: `CropRect { x, y, width, height }`, `clamp(width, height)` (der Teil im Bild, `None` ohne Pixel, ohne
Überlauf bei riesigen Größen), `is_full`, `Image::cropped`. `crates/session/src/export.rs`: `save(…, adjustments, refine, crop)` stellt zuerst Refine (MAC-018) und die
Farb- und Tonwerte her und schneidet dann aus, damit die Randpixel dieselben Werte haben wie alle anderen (RGB Align schiebt Kanäle über
den Rand hinweg). Ein Rechteck außerhalb des Bildes gibt „the crop rectangle is outside the image“ und schreibt keine Datei. FFI:
`CropRect` als UniFFI-Record, `CoreSession.save_image(…, crop)`. Tauri: `export_commands.rs` reicht `None` durch (Windows ergänzt das
Argument, siehe `WINDOWS_PARITY.md`).

**Oberfläche.** Die Geometrie ist aus Siderea übernommen: `mac/App/Model/CropRectangle.swift` (`CropAspect`, `CropRectangle` mit
`dragged`, `drawn`, `fitted`, `handle(atX:…)`, Fadenkreuz-Arm, Drittel-Linien; ohne die Ränder des Crop-Moduls, dafür `full`, `centred`,
`clamped`). Zustand und Regeln: `CropLogic.swift` (`CropFormat`, `CropState`), `AppModel+Crop.swift` (`startCrop`, `finishCrop`,
`cancelCrop`, `resetCrop`, `removeCrop`, `setCropFormat`, `setCropDraft`, `saveCrop`, Löschen bei neuem Stack über `stacked.didSet`,
Schließen beim Verlassen von Wavelets und Save über `viewStep.didSet`). Gezeichnet in `Views/CropOverlay.swift` (SwiftUI-`Canvas`, Maße
und Deckkräfte von Siderea), gezogen in `ImageCanvasView` (`isCropping`, `crop`, `cropAspect`, `onCropChange`, Zeiger-Rechtecke; die
diagonalen Pfeile erst ab macOS 15, davor das Fadenkreuz), Karte `CropCard` und Gruppe `CropSection` in `Views/Save/CropViews.swift`. Die
Karte steht als oberste im Kartenstapel der Adjustments (`openCards`), im Schritt Save allein.

**Entscheidungen.**

- *Wo gilt der Zuschnitt:* nur für das geschärfte Bild (Schritte Wavelets und Save), wie die Farb- und Tonwerte. Das Stack-Bild aus dem
  Schritt Stack wird ganz geschrieben, denn dort ist das Rechteck weder sichtbar noch einstellbar.
- *Projektdatei:* der Zuschnitt steht **nicht** in der Projektdatei. Sie hält nur Einstellungen, nicht das Ergebnis; Align und Stack laufen
  nach dem Öffnen neu, und ein neuer Stack löscht den Zuschnitt ohnehin. Das Format der Windows-App bekommt also keinen neuen Abschnitt.
- *Ausrichtung:* wie in Siderea gibt es keinen Umschalter Hoch/Quer. Ein festes Format liegt oder steht, wie das Rechteck gerade ist; ein
  neues Rechteck folgt der Zugrichtung. (Siderea nennt die Formate 1:1, 2:3, 3:4; hier sind es die gewünschten 1:1, 3:2, 4:3, dieselbe
  Rechnung.)
- *Schaltfläche:* „Cropping Area“ heißt in der Mac-App „Crop“ (wie das Werkzeug); Windows folgt (Parity).
- *View Compare (MAC-019):* beim Öffnen des Werkzeugs geht die geteilte Ansicht aus, und solange es offen ist, lässt sich View Compare nicht
  einschalten (Statuszeile); `isComparing` ist bei offenem Werkzeug immer falsch. Zwei Tests in `CropModelTests`.
- *Start:* das Werkzeug öffnet auf der Mitte (80 % jeder Seite) oder auf dem angewendeten Rechteck, damit gleich etwas zu sehen ist.

## Prüfung

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 Warnungen), `cargo fmt --all`, `scripts/build-ffi.sh`,
  `swift test` im Paket, `xcodebuild test` (App), `pnpm test` (Doku-Seiten).
- Rot zuerst, Rust: die Stubs `clamp`, `is_full`, `cropped` und `save` ohne Zuschnitt lieferten das Rechteck unverändert beziehungsweise
  die ganze Datei, 5 Prüfungen in `crop.rs` und 4 in `export.rs` schlugen an Assertions fehl; die FFI-Umwandlung mit vertauschtem x und y
  schlug an `a_crop_rectangle_keeps_every_field` fehl; der echte Lauf mit `JUPITER.AVI` mit abgeschaltetem Zuschnitt: 1024 × 680 statt
  600 × 450.
- Rot zuerst, Swift: die Modellmethoden waren zuerst leere Rümpfe, 19 Prüfungen in `CropModelTests` und `WaveletLogicTests` schlugen an
  Assertions fehl. Die aus Siderea übernommene Geometrie (`CropRectangleTests`) war schon grün; mit vier absichtlichen Fehlern
  (Seitenverhältnis immer stehend, Klemmen ohne Verschieben, Save Image ohne Rechteck, Löschen nur bei offenem Werkzeug) schlugen 19
  Prüfungen in 11 Tests an Assertions fehl.
- Bilder: dunkel und hell, 4:3 und Free (Schritt Wavelets und Save), mit Fadenkreuz, Drittel-Raster, Griffen und Größe; dazu der Zustand
  nach Done. Zugeschnittenes TIFF: `sips` meldet 600 × 450.
