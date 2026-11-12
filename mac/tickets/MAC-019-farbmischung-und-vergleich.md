# MAC-019 Color Mixing und View Compare

- Status: erledigt
- Abhängig von: MAC-014 (Wortregeln), MAC-008 (Adjustments-Panel und Karten)

## Ziel

Zwei weitere Knöpfe des Panels Adjustments im Schritt Wavelets (Philipp, 2026-11-12) werden freigeschaltet; die übrigen bleiben gesperrt.

1. **Color Mixing:** Karte mit einem 3 × 3 Kanalmischer. Jeder Ausgangskanal (R, G, B) ist die gewichtete Summe der drei Eingangskanäle,
   Regler und Felder von −200 bis +200 %, dazu **Preserve brightness** und ein Preset-Menü (Neutral und drei weitere). Nützlich für
   Aufnahmen durch ein einzelnes Filter (Infrarot-Pass, Monochrom-Kamera) und zum Korrigieren von Farbstichen. Gerechnet im Rust-Kern
   neben den anderen Farbwerten, mit Live-Vorschau, angewendet von Do All und Save Image wie die anderen Karten, im Projekt gespeichert
   (optional; das Windows-Format bleibt kompatibel), neutral = Identität bitgenau.
2. **View Compare:** Teilansicht im Bildbereich: links das Summenbild (**Before**), rechts das Ergebnis mit Schärfung und Karten
   (**After**), getrennt von einer ziehbaren senkrechten Linie, beide Hälften mit demselben Zoom und derselben Lage. Reine Ansicht, kein
   Eingriff in den Kern. Das bestehende **Before/After** der Zoom-Leiste bleibt und ist mit der Teilung abgestimmt.

## Akzeptanzkriterien

- [x] Kern: `ChannelMix` und `mix_channels` in `crates/core/src/color.rs`, Teil von `Adjustments` zwischen RGB Align und RGB Balance;
      Identität lässt das Bild bitgenau stehen, Graustufenbilder mit weniger als drei Kanälen laufen unverändert durch
- [x] Rust-Tests: Identität (auch mit Preserve brightness), bekannte Matrix, Vertauschen und Abschneiden, **Preserve brightness hält die
      Luminanz**, Pixel das zu Schwarz wird, Reihenfolge vor RGB Balance (alle erst rot, siehe unten)
- [x] Sitzung und Schnittstelle: `ColorMix` (Prozent, neun Werte zeilenweise) in `ColorSettings`, `#[serde(default)]`, damit Tauri-Befehle,
      Windows-Projekte und bestehende Aufrufer ohne das Feld die Identität bekommen; `crates/ffi` spiegelt es für Swift
- [x] Karte Color Mixing: neun Regler mit Feldern (−200 … 200 %), Preset-Menü (Neutral, Swap Red and Blue, Luminance Gray, Red to All
      Channels), Häkchen Preserve brightness, Reset; Live-Vorschau; bei Graustufenbildern nicht gesperrt
- [x] Do All und Save Image wenden die Mischung wie die anderen Karten an (`isNeutral` kennt die Matrix); Projektdatei `wavelet.tone.mix`
      (`matrix`, `preserveBrightness`), optional und tolerant gelesen
- [x] View Compare: Teilung mit ziehbarer Linie (Griff, Cursor, VoiceOver), Beschriftungen Before und After in den unteren Ecken, gleicher
      Zoom und gleiche Lage; die Linie bleibt innerhalb von 2 … 98 % der Breite; Schaltfläche stellt sich auf Gedrückt
- [x] Before/After der Zoom-Leiste und View Compare schließen sich aus (Before beendet die Teilung, View Compare beendet Before)
- [x] Wortlaut nach den Regeln von MAC-014 (US-Englisch, Schaltflächen in Title Case, Beschriftungen und Tooltips in Sentence case,
      Einheiten, kein „the reference program“); nur diese zwei Knöpfe entsperrt
- [x] Swift-Tests: Kartenzustand, Presets, Kern-Einheiten, Projekt-Rundlauf, Teilung ein und aus, Grenzen der Linie (alle erst rot)
- [x] Hilfeseiten (`color-and-tones.md`, `image-area.md`, `step-wavelets-save.md`, `save-and-export.md`, `glossary.md`), `docs/design.md`
      und `docs/parity/WINDOWS_PARITY.md` (Windows baut Karte und Teilung nach)
- [x] Cargo-Tests, `clippy` ohne Warnungen, `cargo fmt`, `swift test`, `xcodebuild test` und `pnpm test` grün; Tauri-Shell baut

## Umsetzung

**Kern.** `ChannelMix { matrix: [[f32; 3]; 3], preserve_brightness }` mit `matrix[ausgabe][eingang]` als Faktoren (1,0 = 100 %), Standard
die Einheitsmatrix. `mix_channels` ersetzt jedes Pixel durch die Summen, negative Summen werden 0 (es gibt kein negatives Licht).
Mit **Preserve brightness** wird danach jedes Pixel so skaliert, dass es die Luminanz vor dem Mischen behält (`LUMA_WEIGHTS`, dieselben
Gewichte wie `Image::to_luma`); ein Pixel, das zu Schwarz gemischt wurde, wird grau in der alten Luminanz. Danach Abschneiden auf 0 … 1
(das kann den Farbton sehr heller Pixel leicht verbiegen, in der Hilfe vermerkt). Die Einheitsmatrix und Bilder mit weniger als drei
Kanälen werden übersprungen, die Identität bleibt also bitgenau.

**Schnittstelle.** Die Sitzung nimmt `ColorMix { matrix: [f32; 9], preserve_brightness }` in Prozent (`matrix[3 * ausgabe + eingang]`) in
`ColorSettings.mix` an (Grenze ±200 % auch im Kern). In `crates/ffi` ist `ColorMix` ein Record mit `Vec<f32>`; `ColorSettings` ist dort
wegen des `Vec` nicht mehr `Copy`.

**Mac.** `ColorMixLogic.swift` (Werte, Presets, Grenzen, Projektwerte), `Views/Wavelets/ColorMixingCard.swift` (Karte, bindet an
`ToneValues.mix`); `ToneValues.core`, `HistogramKey`, `isNeutral` und der Projektabschnitt kennen die Matrix. `CompareLogic.swift` (Linie als
Anteil der Breite, Grenzen, Platz der Beschriftungen), `AppModel+Compare.swift` (`isComparing`, `compareBeforeImage`, `setBefore`),
`Views/CompareOverlay.swift` (Linie, Griff, Beschriftungen; nur der Streifen um die Linie nimmt die Maus) und in `ImageCanvas.swift` eine
zweite Layer für das Summenbild, die an der Linie abgeschnitten wird (gleiche Position und Skalierung wie die Bildlayer, also kostet das
Ziehen nichts). Die Seite-an-Seite-Variante gibt es nicht (nicht billig: zwei Bildflächen bräuchten einen eigenen Zoom).
Debug-Argumente: `-mix 100,40,0,0,100,0,0,0,70`, `-preserve 1`, `-divider 0.4`, dazu `-functions "Color Mixing,View Compare"`.

**Abgestimmt mit Before/After:** Beide Schalter sind Zustand des Modells. *Before* an beendet die Teilung (der Knopf kommt hoch), der Knopf
View Compare schaltet *Before* aus. Solange geteilt wird, entfällt die Beschriftung oben links („Sharpened stack“), die die rechte Hälfte
beschriftet hätte; die Ecken tragen Before und After. Die Teilung gilt nur im Schritt Wavelets.

## Rot zuerst

Rust (`cargo test -p planets-core color::`): `mix_channels` zunächst als Leerlauf (Früh-Rückkehr). Dagegen schlugen an der Assertion fehl
`mixing_matrix_gives_the_weighted_sums`, `mixing_swaps_channels_and_clamps`, `preserve_brightness_keeps_the_luminance` und
`mixing_runs_between_align_and_balance`; `preserve_brightness_turns_a_pixel_that_mixes_to_black_gray` war zuerst grün, weil seine Eingabe grau
war, wurde mit einer farbigen Eingabe neu geschrieben und schlug dann fehl. Identität und Graustufen sind von Natur aus auch gegen den Leerlauf
grün (sie prüfen, dass nichts passiert; ihr Gegenstück ist die bekannte Matrix). Swift (`xcodebuild test`): sechs Stellen der Umsetzung
vorübergehend kaputt (Linie nicht begrenzt, Before beendet die Teilung nicht, Teilung setzt Before nicht zurück, Matrix nicht geklemmt, Mischung
fehlt in `isNeutral`, Mischung nicht im Projektabschnitt): 17 Fehlschläge an Assertions in `ColorMixAndCompareTests` und `SaveLogicTests`,
danach grün. Der Test im Swift-Paket (`ColorMixTests`) läuft durch die echte Schnittstelle; ohne das Feld in der Schnittstelle lässt er sich
gar nicht übersetzen, rot an einer Assertion war er nicht.
