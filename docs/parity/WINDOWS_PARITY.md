# Windows nachziehen — Änderungen der Mac-App

Sammelstelle für alles, was die Mac-App (`mac/`, SwiftUI) bekommen hat und die Windows-App (`app/`, Tauri und Vue) noch
übernehmen muss; beide liegen auf `main`, die Gegenrichtung steht in `docs/parity/MAC_PARITY.md`. **Kein eigenes Ticket pro
Anpassung:** diese Datei ist die Liste, im Windows-Ticket (`tickets/RK-NNN-…`) genügt der Verweis darauf. Hier stehen nur
offene Punkte: Der Commit, der einen Punkt in Windows umsetzt, löscht ihn (ein Abschnitt ohne Punkte fällt mit weg); was erledigt
wurde, zeigt die Git-Historie dieser Datei.

Quelle der Wahrheit für das Aussehen ist `docs/design.md`, für das Verhalten die Mac-Implementierung (Dateien im Punkt). Die
Mac-Seite ist SwiftUI und nicht übertragbar; übernommen wird das **Verhalten und die Werte**. Gemeinsame Dateien (Rechenkern in
`crates/`, Doku-Seiten unter `app/src/docs/pages/`, Beispieldaten-Beschreibungen) liegen schon auf `main`.

Ein Punkt je Verhalten, mit den Werten (Größen, Deckkräfte, Token-Namen), den Mac-Dateien, den Texten und dem Hinweis, was davon
schon gemeinsam auf `main` liegt. Abschnitte heißen wie das Feature, gruppiert nach der Mac-Version, die es zuerst hatte.

`.githooks/commit-msg` verlangt diese Datei in jedem Commit, der die Mac-Oberfläche (`mac/`) ändert — außer der Commit löscht einen
erledigten Punkt aus `docs/parity/MAC_PARITY.md` (Mac zieht nach) oder trägt `[mac-only]` in der Nachricht.
`scripts/parity-status.sh` listet die offenen Punkte je Mac-Version.

Bis `MAC-001` fertig ist, ist die Windows-App die Vorlage: hier stehen nur Verhalten, die der Mac vorab hat, weil sie nützlich sind
und keine reine Mac-Konvention (Menüleiste, ⌘-Kürzel, Gesten und native Dialoge stehen als Abweichung in `docs/design.md`,
„Mac-App“). Danach führt der Mac, und diese Liste füllt sich mit jeder Änderung.

## Drehen und Größe (Mac: MAC-017)

- [ ] Die beiden Adjustments-Schaltflächen **Flip and Rotate** und **Resize Image** öffnen je eine Karte (Aufbau, Werte und Texte in `docs/design.md`,
      Abschnitt „Drehen und Größe (MAC-017)“; die Schaltflächen sind bisher gesperrt, „Not available yet“, und `isToneCard` kennt sie nicht); die Zahl der
      gesperrten Knöpfe sinkt auf **vier** (View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph).
      **Flip and Rotate:** Schalter **Horizontal · Vertical**, Knöpfe **Left 90° · Right 90° · 180°**, **Angle (°)** −180 … 180 in Zehnteln (positiv im
      Uhrzeigersinn) mit Feld und Regler, **Interpolation** Bilinear · Bicubic (Standard Bicubic), Häkchen **Crop to fit**, Zeile *Picture* mit der Größe, **Reset**.
      Die Schalter wirken auf das Bild auf dem Bildschirm: der Kern spiegelt zuerst und dreht dann, darum kehrt jedes Spiegeln das Vorzeichen des Winkels um
      (`OrientationValues.flipHorizontally`). **Resize Image:** Segmente **Percent · Pixels**, **Size (%)** 100 · 10 … 400, **Width (px)** / **Height (px)** 1 … 16384
      mit **Lock aspect ratio** (an: die zuletzt getippte Seite bestimmt die andere), **Method** Lanczos3 · Bicubic · Bilinear · Nearest Neighbor (Standard Lanczos3),
      Zeile *Result* mit der Größe der Datei, **Reset**.
- [ ] **Reihenfolge und Wirkung:** Refine → Farb- und Tonwerte → **Flip and Rotate** → **Crop** → **Resize Image**. Das Bild im Bildbereich ist das gedrehte
      (Schritte Wavelets und Save, auch Before und die linke Hälfte von View Compare); das Zuschneiden arbeitet in den Pixeln dieses Bildes. Resize Image zeigt
      sich nur als Größe (Karte, Gruppe **Size** im Schritt Save), nicht im Bild. Ändert man Spiegelung, Winkel oder Crop to fit, wird ein angewendeter Zuschnitt
      entfernt („Crop removed: the picture was flipped or turned“); bei offenem Zuschneiden sind die Bedienelemente von Flip and Rotate gesperrt. Die Marken des
      Stacks (Alignpoints, Preview-Bereich) werden auf einem gedrehten Bild nicht gezeichnet. Gilt nur für das geschärfte Bild; das Stack-Bild des Schritts Stack
      wird nie gedreht oder skaliert. Die Werte bleiben bei einem neuen Stack und einer anderen Aufnahme (der Zuschnitt nicht).
- [ ] **Projektdatei:** eigener optionaler Abschnitt `geometry` (`flipHorizontal`, `flipVertical`, `angle`, `cropToFit`, `rotationMethod`, `resize`
      {`mode`, `percent`, `width`, `height`, `lockAspect`, `driver`, `method`}); fehlt er oder ein Schlüssel, bleiben die Werte, Zahlen werden auf die Grenzen gekürzt.
      Mac: `mac/App/Model/GeometryLogic.swift` (`geometrySection`, `applyGeometrySection`), `AppModel.projectSections`.
- [ ] **Rechenkern schon gemeinsam auf `main`:** `planets_core::geometry` (`Orientation`, `Resize`, `ResizeTarget`, `Resampling`, `Geometry`, `Image::flipped`,
      `turned`, `oriented`, `resized`), `Session::save_image(…, crop, geometry: Option<&Geometry>)` und `Session::orient_preview` (8-Bit-RGBA, Vorschau).
      Die Tauri-Befehle `save_image` und `save_image_as` reichen vorerst `None` durch (`app/src-tauri/src/export_commands.rs`); Windows ergänzt dort das Argument
      `geometry` (serde-Typen fehlen noch: `Orientation`, `Resize`, `Geometry` brauchen dann `Deserialize` mit camelCase) und den Befehl für die Vorschau.
      Anwenderseiten (`color-and-tones.md`, `save-and-export.md`, `step-wavelets-save.md`, `image-area.md`, `glossary.md`) liegen schon gemeinsam auf `main`.
      Mac: `mac/App/Views/Wavelets/GeometryCards.swift`, `mac/App/Model/AppModel+Geometry.swift`.

## De-rotate images (Mac: MAC-020)

- [ ] Das Häkchen **De-rotate images** (Advanced im Schritt Stack) ist frei und zeigt eingeschaltet die Felder **Planet** (Segmente Jupiter · Saturn · Mars, beim Öffnen
      aus dem Dateinamen vorgeschlagen), **Start time** (nur wenn die Datei keine Zeit je Frame hat; Datum und Uhrzeit in der Ortszeit, darunter dieselbe Zeit in UTC und bei einem
      Vorschlag aus dem Dateidatum „taken from the file date, check it“), **Camera angle (°)** (−360 … 360, eine Nachkommastelle, Standard 0, positiv gegen den Uhrzeigersinn),
      das Häkchen **Mirrored image** und die Zeile **„Span 3 min 12 s · rotation 5.8°“** mit dem Hinweis **„Too short to matter“** unter 1°. Aufbau, Werte und Texte in `docs/design.md`,
      Abschnitt „Derotation (MAC-020)“. In `StackStep.vue` wird aus dem gesperrten Häkchen (`stack-pages.test.ts` zählt bisher sieben gesperrte Optionen, danach sechs) die
      Gruppe; `settings.stack.derotate` ist der Schalter, die übrigen Werte kommen neu dazu (`planet`, `cameraAngle`, `mirrored`, `startTime`).
- [ ] **Overlay** im Schritt Stack, solange das Häkchen an ist: gemessene Scheibe als Ellipse (`--accent`, 1,5 px), für Saturn die Außenkante des Rings gestrichelt, kurzer Strich mit **N**
      über dem Nordpol des Planeten; der Schritt zeigt dazu das Bezugsframe (dort wurde gemessen). Mac: `mac/App/Views/ImageOverlays.swift` (`drawDisc`),
      `mac/App/Model/AppModel+Derotation.swift` (`refreshDerotation`, 120 ms verzögert, nach jeder Änderung der Felder und der Frame-Auswahl).
- [ ] **Projektdatei:** optionaler Abschnitt `derotation` (`enabled`, `planet`, `cameraAngle`, `mirrored`, `startTimeUtc` in Sekunden seit 1970 UTC); fehlt er oder ein Schlüssel,
      bleiben die Werte. Mac: `mac/App/Model/StackLogic.swift` (`derotationSection`, `applyDerotationSection`).
- [ ] **Rechenkern schon gemeinsam auf `main`:** `StackSettings.derotation: Option<DerotationSettings>` (`#[serde(default)]`, die Tauri-Befehle reichen den Wert durch, die Windows-App
      schickt bisher keinen: `{ "planet": "jupiter" | "saturn" | "mars", "cameraAngleDeg": 0, "mirrored": false, "startTimeUtc": <Sekunden seit 1970 oder null>, "useFileTimestamps": true }`),
      `Session::derotation_info(settings, referenceFrame, frames)` (Span, Drehung, gemessene Scheibe; ein Befehl dafür fehlt in `app/src-tauri` noch), `RecordingInfo` mit
      `hasTimestamps`, `startTime` (Unix-Sekunden) und `startTimeFromFileDate`; SER-Zeitstempel (Nachspann der Datei) in `planets_io::ser`. Die Hilfeseiten `step-stack.md` und
      `glossary.md` beschreiben die Funktion schon (Zeile in `step-stack.md`: „Works in the Mac version“; die Windows-Version nennt sie bis dahin gesperrt, dann ist die Zeile
      und der Test `describes De-rotate images as working in the Mac version` anzupassen). *Save Aligned* und *Save Integrated* derotieren nicht.

## Align by center of gravity (Mac: MAC-021)

- [ ] Im Schritt Align, Advanced, sind **Align by center of gravity** (Häkchen, Standard aus) und **Brightness threshold** (0–255, Standard **20**,
      eingerückt 20 px, nur mit dem Häkchen aktiv, sonst 45 % gedimmt) frei; *Estimate rotation* bleibt gesperrt. Tooltips und Aufbau in `docs/design.md`,
      Abschnitt „3 Align“. Windows: `app/src/components/AlignStep.vue` (heute gesperrt, Feld heißt dort noch „Lum. Threshold“ mit 0), neue Werte
      `settings.align.centreOfGravity` und `settings.align.brightnessThreshold`. Mac: `mac/App/Views/AlignPanels.swift`, `mac/App/Model/AlignLogic.swift`
      (`centreOfGravityThreshold`: aus = `null`, an = Schwelle auf 0–255 begrenzt).
- [ ] Align **und** Stack schicken den Wert mit: `AlignSettings.centreOfGravity` und `StackSettings.centreOfGravity` (Zahl 0–255 oder fehlend/`null` = aus).
      Windows: `app/src/logic/align.ts` (`alignArgs`), `app/src/logic/stack.ts`. Mac: `mac/App/Model/StackLogic.swift` (`stackSettings`).
- [ ] **Projektdatei:** Abschnitt `align` bekommt `centreOfGravity` (bool) und `brightnessThreshold` (Zahl); fehlen sie, bleiben die Werte.
      Mac: `mac/App/Model/SaveLogic.swift` (`alignSection`, `applyAlignSection`).
- [ ] **Schon gemeinsam auf `main`:** Rechenkern (`AlignParams.centre_of_gravity`, `crates/core/src/align.rs`), Session (`#[serde(default)]`, die Tauri-Befehle
      reichen den Wert durch, die Windows-App schickt bisher keinen) und die Hilfe (`align-your-frames.md`, Abschnitt „Align by center of gravity“, mit der Zeile
      „Works in the Mac version. In the Windows version the check box is locked for now.“; `steps-alignpoints-align-limit.md`; `glossary.md`). Beim Nachziehen
      die Windows-Hinweise in diesen Seiten streichen.
