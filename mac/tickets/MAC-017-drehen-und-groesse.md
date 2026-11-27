# MAC-017 Flip and Rotate und Resize Image

- Status: erledigt
- Abhängig von: MAC-014 (Wortregeln), MAC-008 (Adjustments-Panel und Karten), MAC-016 (Zuschneiden)

## Ziel

Zwei weitere Knöpfe des Panels Adjustments im Schritt Wavelets (Philipp, 2026-11-12) werden freigeschaltet; die übrigen vier bleiben gesperrt
(View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph).

1. **Flip and Rotate:** Karte zum Spiegeln (horizontal, vertikal), Drehen um 90° links und rechts und um 180°, und Drehen um einen freien Winkel
   (−180 … 180° in Schritten von 0,1°, Bilinear oder Bicubic, Hintergrund schwarz), dazu **Crop to fit**, damit keine schwarzen Ecken bleiben.
   Wer Planeten aufnimmt, richtet damit Norden nach oben aus oder gleicht die Lage an eine Karte oder ein anderes Bild an.
2. **Resize Image:** Karte zum Skalieren um 10 … 400% oder auf Breite und Höhe in Pixeln, mit **Lock aspect ratio**, mit der Methode Lanczos3
   (Standard), Bicubic, Bilinear oder Nearest Neighbor und der Anzeige der Größe, die herauskommt.

Beides sind Geometrie-Schritte im Rechenkern am Ende von Save Image; das Bild im Bildbereich zeigt das gedrehte Bild. Do All und das Speichern in
allen Formaten (BMP, JPEG, FITS, TIFF, PNG) wenden sie an.

## Reihenfolge (entschieden)

**Refine → Farb- und Tonwerte → Flip and Rotate → Crop → Resize Image.**

Das Zuschneiden (MAC-016) liegt in den Pixeln des Bildes, das man sieht. Würde zuerst zugeschnitten und danach gedreht, läge das Rechteck auf einem
Bild, das der Nutzer so nie sieht (das Werkzeug müsste das ungedrehte Bild zeigen, während alles andere gedreht wäre). Darum ist das **gedrehte Bild
das Bild des Bildbereichs** und der Zuschnitt arbeitet in dessen Pixeln. Resize Image kommt zuletzt und skaliert, was nach dem Zuschnitt übrig ist
(50% eines 600 × 800-Zuschnitts sind 300 × 400); der Bildbereich zeigt es nicht, die Karte und die Gruppe **Size** im Schritt Save nennen die Größe der Datei.

- Spiegeln und Drehen ändern, wo die Pixel liegen. **Ein angewendeter Zuschnitt wird deshalb entfernt**, wenn man Spiegelung, Winkel oder Crop to fit
  ändert (Statuszeile „Crop removed: the picture was flipped or turned“). Die Interpolation ändert nichts an der Lage und lässt ihn stehen.
  Solange das Zuschneiden offen ist, sind die Bedienelemente von Flip and Rotate gesperrt („Finish or cancel the crop first“).
- Die Werte beider Karten sind **Einstellungen** wie die Farbwerte und bleiben bei einem neuen Stack und einer anderen Aufnahme (Norden oben gilt für
  die nächste Aufnahme ebenso); der **Zuschnitt** bleibt nicht (er liegt auf einem bestimmten Bild). Beide Werte stehen in der Projektdatei.
- Die Marken des Stacks (Alignpoints, Preview-Bereich) liegen in den Pixeln des Stacks und werden auf einem gedrehten Bild nicht gezeichnet.
- Das Stack-Bild aus dem Schritt Stack wird nie gedreht oder skaliert (wie beim Zuschnitt).

## Akzeptanzkriterien

- [x] Rechenkern: `planets_core::geometry` mit `Orientation` (Spiegeln, Winkel, Crop to fit, Methode), `Resize` (Prozent oder Pixel, eine Seite darf der anderen folgen),
      `Resampling` (Nearest, Bilinear, Bicubic als Catmull-Rom, Lanczos3), `Geometry`; `Image::flipped`, `turned`, `oriented`, `resized`. Spiegeln, Vierteldrehungen und
      Winkel 0 lassen Pixel nur wandern oder das Bild bitgenau stehen; ein freier Winkel zerlegt sich in Vierteldrehungen und einen Rest bis 45°
- [x] Rust-Tests (alle erst rot, siehe unten): zweimal Spiegeln = Identität bitgenau; vier Vierteldrehungen = Identität; Winkel 0 = Identität; 90° setzt bekannte Pixel
      (3 × 2, mit Kanälen); freier Winkel behält das Pixel in der Mitte; Drehrichtung im Uhrzeigersinn; Größe des Raums (Bilinear, Bicubic, Crop to fit ohne schwarze Ecke);
      Resize 100% = Identität; Größen für Prozent, Pixel, Seitenverhältnis und Grenzen; Lanczos3 (und die anderen) lassen ein konstantes Bild konstant; Nearest verdoppelt Pixel;
      Bilinear halbiert als Mittel; kein Überschwingen aus 0 … 1; **Reihenfolge Crop nach Drehen und Resize zuletzt**, Crop außerhalb des gedrehten Bildes
- [x] Sitzung und Schnittstelle: `Session::save_image(…, crop, geometry: Option<&Geometry>)` (Reihenfolge Refine, Farben, Drehen, Crop, Resize; ein Rechteck ohne Pixel im gedrehten Bild ist ein Fehler und
      schreibt nichts), `Session::orient_preview` (8-Bit-RGBA, volle Größe); `crates/ffi` spiegelt `OrientationSettings`, `ResizeSettings`, `ResizeTarget`, `Resampling`,
      `GeometrySettings`, `ImageSize` und die Funktionen `orientation_size` und `resize_size`; die Tauri-Befehle reichen `None` durch und bauen weiter
- [x] Karte **Flip and Rotate**: Spiegeln, Drehen um 90°/180°, Winkel (Feld und Regler, 0,1°), Interpolation, Crop to fit, Größe des Bildes, Reset; Live-Vorschau im ganzen Bild
      (auch Before und die linke Hälfte von View Compare, Schritte Wavelets und Save)
- [x] Karte **Resize Image**: Percent (10 … 400) und Pixels (1 … 16384, mit Lock), Methode (Lanczos3 Standard), Größe, die herauskommt, Reset
- [x] Do All (Ergebnis im Bild) und Save Image in allen Formaten wenden Drehen, Zuschnitt und Resize an; TIFF, PNG, BMP und JPEG gedreht, zugeschnitten und skaliert auf 300 × 400 geprüft
- [x] Projektdatei: optionaler Abschnitt `geometry`, tolerant gelesen (Windows ignoriert ihn); Zurücksetzen je Karte; Werte bleiben bei neuem Stack, der Zuschnitt nicht
- [x] Gesperrte Knöpfe von sechs auf vier (View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph); Tests, Hilfeseiten und `docs/design.md` angepasst
- [x] Wortlaut nach den Regeln von MAC-014 (US-Englisch, Schaltflächen in Title Case, Beschriftungen und Tooltips in Sentence case, Einheiten, kein Name des Referenzprogramms)
- [x] Swift-Tests (alle erst rot): Kartenzustand, Winkel und Spiegeln auf dem Bildschirm, Grenzen, Größen, Projekt-Rundlauf, Zurücksetzen, Größe als Text, Vorschau (Größe, Viewport,
      View Compare), Zuschnitt auf dem gedrehten Bild, Zuschnitt entfällt beim Drehen, Sperre bei offenem Zuschneiden, Aufruf von Save Image mit Zuschnitt und Geometrie, echter Kern mit `JUPITER.AVI`
- [x] Anwenderdoku in der App (`color-and-tones.md`, `save-and-export.md`, `step-wavelets-save.md`, `image-area.md`, `glossary.md`), `docs/design.md` (Abschnitt „Drehen und Größe“) und
      `docs/parity/WINDOWS_PARITY.md`
- [x] Ein gedrehtes und skaliertes TIFF aus der echten Kette gespeichert und die Größe mit `sips` geprüft

## Umsetzung

**Kern.** `crates/core/src/geometry.rs`. `Orientation { flip_horizontal, flip_vertical, angle, crop_to_fit, method }`: zuerst spiegeln (im Rahmen des Bildes), dann im Uhrzeigersinn drehen.
Der Winkel zerfällt in ganze Vierteldrehungen (`turned`, ohne Rechnen) und einen Rest von höchstens 45° (`rotated_free`, rückwärts abgebildet, Taps am Rand wiederholt,
außerhalb des Quellbildes schwarz). Der Raum wächst auf das umschließende Rechteck; seine Seiten haben die Parität der Quelle, damit die Mitte des Bildes auf der Mitte eines Pixels bleibt
(das Pixel in der Mitte bleibt). **Crop to fit** nimmt das größte Rechteck in der Form des viertelgedrehten Bildes, das ganz im gedrehten liegt. `Resize { target, method }` mit
`ResizeTarget::Percent` (10 … 400) oder `Pixels { width, height }` (eine Seite `None` folgt der anderen, 1 … 16384); `output_size` ist die einzige Quelle der Größen, auch für die Oberfläche
(FFI `resize_size`). Skaliert wird getrennt waagerecht und senkrecht; beim Verkleinern wird der Kern um den Faktor gestreckt (kein Aliasing); eine Achse, die ihre Größe behält, wird übersprungen
(bitgenau). Ergebnis auf 0 … 1 geklemmt. `Geometry::finish(image, crop)` ist die Kette Drehen, Crop, Resize.

**Sitzung.** `crates/session/src/export.rs`: `SaveOptions { adjustments, refine, crop, geometry }` statt immer längerer Parameterlisten; `save` stellt Refine und Farben her und ruft
`Geometry::finish`. `crates/session/src/geometry.rs`: `Session::orient_preview`. **Schnittstelle:** `crates/ffi/src/types.rs` (Spiegel der Typen), `CoreSession.save_image(…, geometry)`,
`orient_preview`, freie Funktionen `orientation_size` und `resize_size`. Tauri: `export_commands.rs` reicht `None` durch (Windows ergänzt, siehe `WINDOWS_PARITY.md`).

**Mac.** `GeometryLogic.swift` (Werte, Grenzen, Größen, Projektabschnitt), `AppModel+Geometry.swift` (Größen, Bedienung, Zuschnitt entfernen, gedrehtes Bild für den Bildbereich in einem Slot je Bild:
`shown` und `before`, eine Anfrage zugleich, das vorige Bild bleibt bis das neue da ist), `Views/Wavelets/GeometryCards.swift` (`FlipRotateCard`, `ResizeCard`, `SizeSection` im Schritt Save).
`AppModel.shownImage` und `compareBeforeImage` geben das gedrehte Bild, `cropImageSize` ist die Größe nach dem Drehen. Debug-Argument `-geometry angle=15,fit=1,percent=50`
(auch `fliph`, `flipv`, `method`, `width`, `height`, `lock`) mit `-functions "Flip and Rotate,Resize Image"`.

**Die Knöpfe wirken auf das Bild auf dem Bildschirm.** Der Kern spiegelt zuerst und dreht danach; ein Spiegeln nach einer Drehung würde an der falschen Achse spiegeln. Darum kehrt
`flipHorizontally` und `flipVertically` das Vorzeichen des Winkels um (ein Spiegel, der eine Drehung durchquert, dreht sie um).

## Prüfung

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 Warnungen), `cargo fmt --all`, `scripts/build-ffi.sh`, `swift test` im Paket, `xcodebuild test` (App), `pnpm test` (Doku-Seiten).
- Rot zuerst, Rust: `geometry.rs` zuerst mit Rümpfen, die das Bild unverändert liefern: 23 Prüfungen schlugen an Assertions fehl (Spiegeln, Vierteldrehungen, Größen, Skalieren,
  Reihenfolge); die Tests, die ein Nichts-passiert-Verhalten prüfen (Identität, zweimal Spiegeln), sind von Natur aus auch gegen den Rumpf grün. Zwei Tests (Mitte, konstantes Bild) wurden
  dafür mit einer Größenprüfung verschärft, damit sie gegen den Rumpf rot sind. Sitzung: `save` ohne Geometrie: 5 Prüfungen in `export.rs` und die Kette mit `JUPITER.AVI`
  (600 × 580 statt 300 × 400) schlugen an; `orient_preview` als Rumpf: 2 Prüfungen an Assertions (die Prüfung der Bytezahl stürzte am `debug_assert` des Rumpfes ab); FFI-Umwandlung mit vertauschtem
  Spiegeln: `the_geometry_keeps_every_field` rot.
- Rot zuerst, Swift: Paket (`GeometryTests`): `orientPreview` mit der Neutralen statt der gewünschten Orientierung: 3 Prüfungen rot. App: nach dem Schreiben der Tests wurden
  vorübergehend 13 Stellen der Umsetzung zerstört (Spiegeln ohne Vorzeichenwechsel, Winkel nicht gewickelt, Zuschnitt bleibt beim Drehen, `saveGeometry` immer `nil`, Zuschneiden auf dem Stack-Maß,
  Bild nicht gedreht, Abschnitt ohne Winkel, Lock ignoriert, Zuschnitt für Resize ignoriert, Sperre bei offenem Zuschneiden fehlt, Werte beim neuen Stack gelöscht, Knöpfe aus der Kartenliste,
  linke Hälfte von View Compare ungedreht, Quelle beim Speichern nicht beachtet) und neun weitere (u. a. Resize ohne Neutralprüfung, Reset löscht alles, Spiegeln der Felder vertauscht,
  Größen immer gleich): zusammen 73 Fehlschläge an Assertions in `GeometryLogicTests`, `GeometryModelTests` und `WaveletLogicTests` (50 und 23 in zwei Läufen), dazu 6 in `GeometryModelTests` und
  `JupiterSaveTests` mit `saveGeometry` gleich `nil`; danach grün. Nur Tests, die ein Nichts-passiert-Verhalten prüfen (Standardwerte, Spiegeln und wieder zurück), waren gegen die
  Zerstörungen nicht alle einzeln rot; ihr Gegenstück sind die Tests mit Werten.
- Bilder: gedreht um 90°, frei um 15° (mit und ohne Crop to fit), die Karte Resize Image; gedrehtes, zugeschnittenes und skaliertes TIFF: `sips` meldet die erwartete Größe.
