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

## v0.1.0 (Mac, in Arbeit)

### Schritt Open: Dateiangaben (MAC-005)

- [ ] Das Panel des Schritts Open zeigt unter dem Dateinamen die Zeilen Format, Codec, Size, Bit depth, Frames, Colour, Frame rate
  und, wo die Datei sie hat, Observer, Instrument und Telescope (12 px, Beschriftung `--muted`, Wert `--text`), statt nur Codec,
  Größe und Frame-Zahl in einer Zeile. `RecordingInfo` des Rechenkerns liefert alle Werte schon. Mac:
  `mac/App/Model/RecordingDetails.swift`, `mac/App/Views/StepPanel.swift` (`OpenStepView`).

### Frame-Leiste: vorheriger und nächster Frame (MAC-005)

- [ ] In der Frame-Leiste stehen zwischen Goto Frame und Regler zwei Schaltflächen „vorheriger Frame“ und „nächster Frame“
  (Pfeil links und rechts, ohne Rahmen, am ersten bzw. letzten Frame gesperrt; Tooltip nennt das Kürzel). Der Mac belegt sie mit ⌘[ und
  ⌘], eine Windows-Belegung (z. B. ← und →, solange kein Feld den Fokus hat) legt das Windows-Ticket fest. Mac:
  `mac/App/Views/FrameBar.swift`, `AppModel.showNextFrame(by:)`.

### Schritte per Tastatur (MAC-010)

- [ ] Die sieben Schritte der Schrittleiste lassen sich per Tastenkürzel öffnen: Schritt n (1 Open bis 7 Save) mit einem Kürzel, am Mac
  ⌥⌘1 bis ⌥⌘7 im Menü Process > Go to Step. Wie ein Klick auf die Schrittleiste: ein gesperrter Schritt und jede laufende Berechnung
  sperren das Kürzel. Eine Windows-Belegung (z. B. Alt+1 bis Alt+7) legt das Windows-Ticket fest. Mac:
  `mac/App/Commands/StepMenuItems.swift`, `AppModel.show(_:)`.

### Go to best frame (MAC-011)

- [ ] Neue Funktion (im Original nicht vorhanden): eine Schaltfläche **Go to best frame** setzt nach Align den besten Frame als
  Referenz für ein neues Set Alignpoints. Sie steht an zwei Stellen, Standardschaltfläche (kein gefüllter Stil, Tokens wie die übrigen
  Panel-Schaltflächen): im Panel des Schritts Limit **unter dem Qualitätsdiagramm** (volle Panelbreite, 8 px Abstand zum Diagramm wie
  zwischen den Elementen des Panels) und in der Karte **Framelist** als zweite Zeile **unter** Check all · Check none · Empty List (volle Kartenbreite,
  kompakte Größe, 24 px, 12 px Text, 8 px seitlicher Rand). Tooltip: „Show the best frame and open Alignpoints to use it as
  reference“; solange Align nicht gelaufen ist (Schritt unter *aligned*, keine Bewertung) ist sie gesperrt, Tooltip „Run Align first“,
  ebenso während einer Berechnung.
  Ein Klick (a) zeigt den **besten Frame**, den ersten der Rangliste (Framelist-Reihenfolge: höchste Qualität zuerst, bei gleicher
  Qualität die kleinere Frame-Nummer), im Bildbereich und in der Frame-Leiste (Goto Frame und Regler stehen auf ihm), (b) öffnet den
  Schritt **Alignpoints** wie ein Klick auf dessen Pille, sodass Set Alignpoints den gezeigten Frame als Referenz nimmt, und (c) merkt
  sich die Frame-Nummer für den Hinweis. **Nichts wird verworfen**: Bewertung, Auswahl der Frames, Limit und Stack bleiben, bis
  Set Alignpoints (oder Align) wieder läuft; erst das löscht sie wie bisher, und mit ihnen den Hinweis. Auch ein neues Öffnen löscht ihn.
  Hinweis oben im Panel des Schritts Alignpoints, **über** dem Threshold-Regler, solange er gemerkt ist (Hinweis-Komponente aus
  `docs/design.md`: Fläche `--accent-soft`, 3 px Streifen links in `--accent`, Radius `--r`, Text 12 px `--text`, Innenabstand
  8 px oben und unten, 10 px rechts): `Best frame: N — run Set Alignpoints and Align again to use it as reference.` Das N zählt von 1
  wie die Frame-Leiste („Best frame: 295“); zwischen „N“ und „run“ steht ein Gedankenstrich (—). Der Hinweis bleibt auch, wenn die
  Anwenderin danach einen anderen Frame zeigt; dann gilt dieser als Referenz, der Hinweis nennt weiter den besten.
  Die Anwenderdoku (`app/src/docs/pages/frame-quality-and-limit.md`, Abschnitt „Use the best frame as reference“, dazu Verweise in
  `align-your-frames.md` und `steps-alignpoints-align-limit.md`) liegt schon gemeinsam auf `main`. Mac: `AppModel.goToBestFrame()`
  und `canGoToBestFrame` (`mac/App/Model/AppModel+Align.swift`), `AlignData.bestFrame` und `bestFrameHint` samt Text `bestFrameHint(frame:)`
  (`mac/App/Model/AlignLogic.swift`), Schaltflächen `GoToBestFrameButton` und `HintNote` (`mac/App/Views/AlignPanels.swift`),
  Karte (`mac/App/Views/FloatingCards.swift`).

## Knopf „Select Recording…“ (Mac, 2026-11-12)

- [ ] Die Schaltfläche, die eine Aufnahme öffnet, heißt **Select Recording…** statt „Select“: im Fuß des Schritts Open (gefüllt) und
      in der Werkzeugleiste (`app/src/components/Toolbar.vue`, `app/src/logic/workflow.ts` bzw. wo die Aktionsbeschriftung des Schritts
      steht), Tooltip „Select a recording: a video or a series of images“. Der Hinweis im leeren Bildbereich lautet
      „Press Select Recording to open a video or a series of images“ (`app/src/App.vue`). Grund: der Knopf öffnet Videos (SER, AVI) und
      Bildserien (TIFF, PNG, BMP, JPEG, FITS). Die Anwenderseiten (`app/src/docs/pages/`) nennen schon **Select Recording** und liegen
      gemeinsam auf `main`. Mac: `ViewStep.action` (`mac/App/Model/StepLogic.swift`), `AppToolbar` (`mac/App/Views/ContentView.swift`),
      `mac/App/Views/ImageArea.swift`.

## Menü „Recent Recordings“ (Mac, 2026-11-12)

- [ ] Das Menü der zuletzt geöffneten Aufnahmen in der Werkzeugleiste heißt **Recent Recordings** statt „MRU“
      (`app/src/components/Toolbar.vue`, `label="MRU"`). Inhalt und Verhalten bleiben. Die Anwenderseiten
      (`menus-and-settings.md`, Abschnitt „Recent Recordings“, und `window-at-a-glance.md`) liegen schon gemeinsam auf `main`.
      Mac: `AppToolbar` (`mac/App/Views/ContentView.swift`).


## Verständliche Texte für Einsteiger (MAC-014, Mac 2026-11-12)

- [ ] **Sammelpunkt: die ganze Oberfläche bekommt klare Namen, Tooltips und gesperrte Elemente ohne Wirkung.** Dieser Punkt gilt vor
  allen älteren Einträgen dieser Datei: wo ein älterer Eintrag einen alten Namen nennt (Framelist, Empty List, Goto Frame, „use it as
  reference“ …), gilt der Name aus den Tabellen unten. Die Anwenderseiten (`app/src/docs/pages/`, dazu die neue Seite **Glossary**)
  nutzen die neuen Namen schon und liegen gemeinsam auf `main`; die Windows-Oberfläche ist in diesem Commit **nicht** geändert.
  Mac-Quellen: `mac/App/Views/**`, `mac/App/Model/StepLogic.swift`, `AlignLogic.swift`, `StackLogic.swift`, `SaveLogic.swift`,
  `WaveletLogic.swift`, `ShowMenuLogic.swift`, `AppModel*.swift`, `SidereaPlanetsApp.swift`, `Settings/SettingsView.swift`. Intern
  bleiben Bezeichner, Projektdatei-Schlüssel (`showFramelist`, `alignedSize` …) und Dateiformate unverändert.

  **Regeln.** (1) Der Name **the reference program**, „the reference program“ und „das Original“ steht in keinem sichtbaren Text beider Apps (Oberfläche,
  Tooltips, Dateidialoge, Statusmeldungen, Datei- und Projektköpfe, Hilfeseiten); Tooltips tragen **keinen** „the reference program: …“-Zusatz.
  (2) US-Englisch überall (Color, Center, gray). (3) HIG-Schreibweise: **Title Case** für Schaltflächen, Menüeinträge, Gruppen- und
  Kartenüberschriften; **Sentence case** für Feldbeschriftungen, Häkchen, Radios, Hinweise und Tooltips; „…“ nur an Aktionen, die einen
  Dialog öffnen (Zeichnen einer Region ist kein Dialog). (4) Einheiten an das Feld: „(px)“, „(%)“, „(fps)“; Maße mit „×“
  („1024×680“), Prozent ohne Leerzeichen („72%“). (5) Statusmeldungen: Großbuchstabe am Anfang, kein Punkt am Ende, ein kurzer Satz.
  (6) **Ein Schalter ohne Wirkung ist gesperrt:** gedimmt (45 %), nicht klickbar, Tooltip beginnt mit „Not available yet“ (bei
  Optionen im Panel zusätzlich der Text „Not available yet“ rechts in der Zeile, wie bei den bisherigen gesperrten Optionen).

  **Tabelle 1: Umbenennungen (alt → neu).** Nur Zeilen, in denen sich ein sichtbarer Text ändert.

  | Ort | alt | neu |
  |---|---|---|
  | **Schritt 4** (Schrittleiste, Panelkopf, Menü Go to Step, Fenstertitel) | **Limit** | **Best Frames** |
  | Schritt 4, Aktionsknopf | Limit | **Keep Best Frames** (fertig: „Keep Best Frames Again“; Sperr-Tooltip der Folge-Pillen „Run Keep Best Frames first“; Name des Laufs im Fenstertitel) |
  | Werkzeugleiste | Open (Projekt) | **Open Project…** |
  | Werkzeugleiste | Save (Projekt) | **Save Project…** |
  | Werkzeugleiste, Pause | Pause-Tooltip „Hold the calculation; press again to go on“ | „Pause the calculation; press again to resume“ |
  | Menü File (Mac) | Open… | Select Recording… |
  | Menü (Mac) | Process (Cancel, Pause, Go to Step) | **Run** |
  | Menü File (Mac) | Reihenfolge Save Project…, Open Project… | Open Project… vor Save Project… (in der Gruppe zu Open) |
  | Menü View (Mac) | Fit Image | Zoom to Fit |
  | Öffnen-Dialog | Prompt „Select“ | „Open“ |
  | Schritt Open, Liste | Recent / No recent files | Recent Recordings / No recent recordings |
  | Schritt Open, Fuß (fertig) | Open another recording | Select Another Recording… |
  | Alle Schritte, Fuß (fertig) | „‹Aktion› again“ (Do All again) | „‹Aktion› Again“ (Do All Again, Keep Best Frames Again …); Schritt Open: Select Another Recording… |
  | Alle Schritte, Fuß | Tooltip „Comes in a later version“ | „Not available yet“ |
  | Alle Schritte, Hinweis | „Running it again discards the steps after it.“ | „Running it again discards the results of the later steps.“ |
  | Schritt Save | Save image | **Save Image** (Menü der Dateitypen: „TIFF (16 bit) · .tif…“, mit „…“, ein Mittelpunkt statt zwei Leerzeichen) |
  | Satz Open | „Choose a recording: a SER or AVI video or a series of images.“ | … + „You can also drop it onto the window.“ |
  | Satz Alignpoints | „Marks the details the frames are lined up on. A lower threshold gives more points.“ | „Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer.“ |
  | Satz Wavelets | „… then apply them to the whole image.“ | „… then press Do All to apply them to the whole image.“ |
  | Ergebniszeile Alignpoints | „82 set“ | „82 points“ |
  | Ergebniszeile Align | „best quality 100 %“ (**Sachfehler**, siehe unten) | „10 frames rated“ |
  | Ergebniszeile Limit | „8 of 10 kept“ | „8 of 10 frames kept“ |
  | Ergebniszeile Wavelets | „applied to the whole image“ | „Sharpening applied“ |
  | Bildbereich leer | „Press Select Recording to open a video or a series of images“ / „No image yet“ | „Select a recording or drop a video or images here“ / „No image to show yet“ |
  | Dateiinfo | Colour | Color (Zeilen Codec und Bit depth bekommen Tooltips) |
  | Schrittleiste rechts | „DIB · 1024×680 · 10 frames“ | „1024×680 · 10 frames · DIB“ (Codec ans Ende) |
  | Alignpoints, Regler | weakest · Threshold · strongest; danach „Alignpoints: 82“ | **more points · Threshold · fewer points**; danach „Threshold · 82 points“ |
  | Alignpoints | Minimum distance between | **Min. distance between points (px)** |
  | Alignpoints | Min distance from edge | **Min. distance from image edge (px)** |
  | Alignpoints | R.o.Interest | **Draw Region** (kein „…“) |
  | Alignpoints, Tooltip | „… press again to drop it“ | „… press again to remove it“ |
  | Alignpoints, Hinweis nach Go to best frame | „Best frame: 7 — run Set Alignpoints and Align again to use it as reference.“ | „Best frame: 7. Run Set Alignpoints and Align again to line all frames up on this one.“ |
  | Alignpoints, Advanced | Gruppe Set Alignpoint parameters | **Point Placement** |
  | Alignpoints, Advanced | Intensity selection: Default · 3x3 area · Lowest pixelvalue | **Brightness measure**: Default · 3×3 area · Lowest pixel value |
  | Alignpoints, Advanced | Lo / Hi (rechts neben den Radios) | **Min. brightness (0–255)** / **Max. brightness (0–255)**, je als eigene Zeile unter den Radios |
  | Alignpoints, Advanced | Number of Alignpoints | Number of alignpoints |
  | Alignpoints, Advanced (gesperrt) | Keep Alignpoints inside ScanFrame | Keep points inside the scan frame |
  | Align | Alignmentbox size | **Box size per point (px)** |
  | Align | Max Alignpoint movement | **Max. point shift (px)** |
  | Align, Advanced (gesperrt) | Align by Centre of gravity / Lum. Threshold / Estimate Rotation | Align by center of gravity / Brightness threshold / Estimate rotation |
  | Align | gesperrte Segmente „No Align · Scan Frames“ ganz oben im Panel | **in Advanced** unter der Beschriftung „Alignment mode“ (dann 5 weitere Einstellungen statt 4) |
  | Limit | Lowest Quality (%) | **At least % of the best quality** |
  | Limit | Best Frames (%) | **Best % of frames** |
  | Limit | Frames/Apoint | **Best frames per point** |
  | Limit | Best Frames (Anzahl) | **Best number of frames** |
  | Limit, Diagramm | best frame … weakest | **sharpest … blurriest** |
  | Limit / Framelist | Go to best frame | **Go to Best Frame**; Tooltip „Show the best frame and open Alignpoints to line all frames up on it“ |
  | Statusleiste Limit | „Processing limited to 8 frames“ | „Keeping 8 of 10 frames“ |
  | Statusleiste nach Limit | „Stacksize: 8  Low quality: 72.00%“ | „8 frames will be stacked · weakest kept: 72%“ (1 Frame: „1 frame will be stacked …“) |
  | Statusleiste Align | „Loading frame: 42“ | „Aligning frame: 42“ |
  | Statusleiste Align | „Alignment finished.“ | „Alignment finished“ |
  | Statusleiste Stack | „Stack size (n=10) - (1024x680)“ / „Stacking Range : 0-255“ | „Stacked 10 frames · 1024×680“ / „Value range 0–255“ |
  | Statusleiste Stack | „Stacking frame: 0“ | „Stacking frame: 0“ bleibt (Zähler wie bei Align) |
  | Statusleiste Alignpoints | „No alignpoints found: try other Lo and Hi values“ | „No alignpoints found: lower the distances or widen Min./Max. brightness under Advanced“ |
  | Statusleiste Alignpoints | „The region is too small for Min distance from edge“ | „The region is too small for the distance from the edge; draw a larger region“ |
  | Statusleiste Fehler | „Error: ‹Kerntext›“ | „‹Schritt› failed: ‹Kerntext›“ (Open, Set Alignpoints, Align, Keep Best Frames, Stack, Do All, Update Preview, Save …) |
  | Frame-Leiste | Goto Frame | **Go to frame** |
  | Frame-Leiste | „Frame (5): 5/10“ | **„Frame 5 of 10“** |
  | Karte | Framelist | **Frame List** (Show-Eintrag „Show Frame List“) |
  | Karte Frame List | Check all · Check none · Empty List | **Check All · Check None · Clear List** (Clear List gesperrt) |
  | Karte Frame List | Frame00010 | **Frame 10** |
  | Karte | Registrationgraph | **Quality Graph** (Show-Eintrag „Show Quality Graph“) |
  | Karte Stack | Alignmentpoints / „Point 1“ | **Alignpoints Used for Stacking** / „Alignpoint 1“ |
  | Show-Menü | Show ROI | Show Region |
  | Show-Menü | Show Aligndata · Show Prefilter · Show Stackgraph · Show Stacklist · Show AlignSections · Show AlignPoints · Show Registrationgraph · Show Framelist | Show Align Data · Show Prefilter · Show Stack Graph · Show Stack List · **Show Alignpoints** (Stack-Schritt, öffnet die Karte) · Show Alignpoints · Show Quality Graph · Show Frame List |
  | Show-Menü | Show Processing Area | **Show Preview Area** |
  | Zoomleiste | Before | **Before/After** |
  | Stack | StackSize: Default · Minimized · Maximized · R.of Interest | **Output size**: Default · Smallest common area · Largest area · Region |
  | Stack | Use Nearestby Alignpoints | **Use nearest points** |
  | Stack, Advanced | Normalization of frame intensity | **Even out frame brightness** |
  | Stack, Advanced | Do not stack pixels at the frameedge | **Ignore edge pixels (px)** (eine Zeile) |
  | Stack, Advanced | Create AVI: Compress Qual / Framerate | Compression quality / **Frame rate (fps)** |
  | Stack, Advanced | Create aligned Sequence / Save Aligned | Aligned video / **Save Aligned Video…** |
  | Stack, Advanced | Create integrated sequence / Save Integrated | Averaged video / **Save Averaged Video…** |
  | Stack, Advanced | Radios ohne Überschrift: Original image size · Minimum area · Maximum area · Region of Interest | Überschrift **Frame size**: Original size · Smallest common area · Largest area · Region |
  | Stack, Advanced | Sort On Quality / Use Prefilters | Sort by quality / Use prefilters (gesperrt) |
  | Stack, Advanced | Average of [5] Frames | Average of [5] frames |
  | Stack, Namen der Läufe | Save Aligned / Save Integrated (Fenstertitel, Status) | Save Aligned Video / Save Averaged Video |
  | Wavelets | Colour / LRGB | **Color / Sharpen brightness only** |
  | Wavelets, Spalten | Layer · Denoise · Sharpen · Preview (Preview zeigt den Sliderwert) | Layer · Denoise · Sharpen · Preview; der **Sliderwert steht als Text rechts neben dem Regler** („10.0“), der gesperrte Preview-Knopf bekommt ein Auge-Symbol |
  | Wavelets, Advanced | Gruppe Wavelets | Wavelet Settings |
  | Wavelets, Advanced | Automatic | **Run Do All automatically** |
  | Wavelets, Advanced | Hold Wavelet Setting | **Keep layer values for a new stack** |
  | Wavelets, Advanced | Waveletscheme: Dyadic (2^n) · Linear | **Layer spacing**: Dyadic (2^n) · Linear (damit „scheme“ nur noch die Datei meint) |
  | Wavelets, Advanced | Initial Layer / Step Incr. | **First layer size / Size step** |
  | Wavelets, Advanced | Use Linked Wavelets | **Move all sliders together** |
  | Wavelets, Advanced | Available schemes (gesperrte Liste) | **Saved Schemes** |
  | Wavelets, Advanced | Load Scheme / Save Scheme | Load Scheme… / Save Scheme… |
  | Wavelets, Dateidialog | Meldung „the reference program Wavelet scheme settings“ (Filtername `.rwv`) | **„Wavelet scheme“** (`app/src/wavelet.ts`, `schemeFilter`) |
  | Wavelets, Fuß | Process | **Update Preview**; Do All wie bisher |
  | Statusleiste Wavelets | layer setting changed | Updating preview… |
  | Statusleiste Wavelets | „Preview: 85 ms“ / „Calculating wavelets done“ | „Preview calculated in 85 ms“ / „Preview updated“ |
  | Statusleiste Wavelets | Do_all processing | Applying to whole image… |
  | Statusleiste Wavelets | „Do All: the stack changed“ | „Do All stopped: the stack changed, run it again“ |
  | Statusleiste Wavelets | „Scheme loaded: ‹voller Pfad›“ (auch saved) | nur der Dateiname |
  | Statusleiste Wavelets | „Error: no stacked image“ | „No stacked image: run Stack first“ |
  | Fenstertitel / Lauf | Wavelet | Do All bzw. Update Preview |
  | Bildbereich | Functions » (Knopf) / Functions (Panel) | **Adjust ›** / **Adjustments** |
  | Functions-Knöpfe | Colour Mixing · Denoise/ Deringing · View Stacksize · Show Linegraph | Color Mixing · Denoise/Deringing · View Stack Size · Show Line Graph |
  | Contrast/Brightness | Regler ohne Beschriftung, darunter zwei Felder | je **Beschriftung + Feld über dem Regler** (Contrast, dann Brightness) |
  | Contrast/Brightness | Link Hold | **Keep** (gesperrt, siehe Tabelle 3) |
  | Panel Adjustments | Copy To · Load to · Difference / Toggle: Current Image · Clipboard Image | **Remember · Restore · Difference** / **Show:** Current image · Remembered image |
  | Statusleiste Adjustments | Image copied · Load to: nothing copied yet · Copy To: no image yet · Copied image loaded · Difference: nothing copied yet | Image remembered · Restore: nothing remembered yet · Remember: no image yet · Remembered image restored · Difference: nothing remembered yet |
  | Karte Histogram | Black / White / Clip % | **Black point / White point / Clip (%)** |
  | Karte RGB Align | X · Y · Max shift | **X (px) · Y (px) · Max shift (px)** |
  | Karte RGB Balance | Felder Red/Green/Blue ohne Einheit | „%“ hinter dem Feld |
  | Alle Tooltips | „… needs a colour image“ | „… needs a color image“ |
  | Save, Satz | „The sharpened stack with the colour and tone settings.“ | „Saves the sharpened stack with your color and tone settings.“ |
  | Save, Satz | „Do All runs first with the current values.“ | „Your latest changes are applied to the whole image first (Do All).“ |
  | Save, Status | „Open project: file not found: …“ | „Open project: the recording was not found: ‹Datei›“ |
  | Zahlen | „100 %“ in Tooltips und Limit-Diagramm | „100%“ ohne Leerzeichen |
  | Einstellungen (Mac, Fenster) | CPUs / Processing Area „512 x 512“ | CPU cores to use / **Preview area** „512 × 512 px“ |
  | Einstellungen | Schemes … Initial Layer, Step Incr. | Wavelet schemes … First layer size, Size step |
  | About (Mac) | „A rebuild of the workflow of the reference program … Not affiliated with the original.“ | „Stacking and sharpening of planetary and lunar recordings.“ (auch `NSHumanReadableCopyright`) |

  **Tabelle 2: Tooltips (neu; Sentence case, ohne „the reference program: …“).** Schritte (Pille und Aktionsknopf):
  Alignpoints „Reference marks on surface details that every frame is lined up on“, Align „Lines up every frame on the alignpoints and
  rates how sharp each frame is“, Best Frames „Choose which frames go into the stack, the sharpest ones“, Stack „Averages the kept frames
  into one image with far less noise“, Wavelets „Sharpens the stack in layers from fine to coarse detail“, Open „Choose the video or
  the images to work on“, Save „Writes the finished image to a file“. Eine Pille nennt dazu in der zweiten Zeile den fehlenden Schritt
  (gesperrt) bzw. das Ergebnis (fertig). Aktionsknöpfe: Set Alignpoints „Places alignpoints automatically on the frame that is shown“,
  Keep Best Frames „Keeps the frames chosen in the settings above and leaves out the others“, Do All „Applies the sharpening to the whole image
  (the preview shows only the preview area)“, Update Preview „Calculates the preview area with the current settings“.
  Felder und Optionen: Threshold „How strong a detail must be to get an alignpoint; left keeps more points, right fewer“; Min.
  distance between points „Smallest distance between two alignpoints, in pixels“; Min. distance from image edge „Alignpoints keep at
  least this distance from the edge of the image or the region, in pixels“; Draw Region „Region of interest (ROI): drag a rectangle on
  the image so that only this area is used for placing alignpoints and aligning; press again to remove it“; Brightness measure „How the
  brightness at a point is measured when alignpoints are placed“ (Default „The centre pixel“, 3×3 area „The mean of the 3×3 pixels
  around the centre“, Lowest pixel value „The darkest of the 3×3 pixels around the centre“); Min./Max. brightness „Only pixels between
  Min. and Max. brightness (0–255) can get an alignpoint“; Box size per point „Edge length of the square around each alignpoint that is
  searched for in the other frames, in pixels“; Max. point shift „How far an alignpoint may move from one frame to the next, in
  pixels“; At least % of the best quality „Keep every frame whose quality is at least this share of the best frame“; Best % of frames „Keep this share of
  all frames, the sharpest ones“; Best frames per point „Each alignpoint uses its own sharpest frames; the field sets how many“; Best
  number of frames „Keep exactly this many frames, the sharpest ones“; Output size „Size of the stacked image“; Use nearest points „How
  many of the nearest alignpoints decide how a pixel is shifted: more is smoother, fewer is more local“; Correct geometry „Shifts each
  part of the image by its own movement; off moves every frame as a whole“; Even out frame brightness „Levels the brightness of the
  frames before stacking; helps when clouds pass by“; Ignore edge pixels „Pixels at the edge of each frame that are left out of the
  stack, in pixels“; Create AVI „Saves the aligned frames as a video, for example for animations or other software“; Save Aligned
  Video „Saves the kept frames, aligned, as an AVI video“; Save Averaged Video „Saves a video in which each frame is the average of
  several aligned frames in a row; the number is set below“; Moving average „Each video frame is the average of the next n aligned
  frames“; Sharpen brightness only „Sharpens only the brightness of the stack and keeps its colors“; Color „Processes the channels in
  color; off gives grayscale“; Layer „Layer 1 holds the finest detail, higher layers coarser structures; switch a layer off to leave
  it out“; Denoise „Smooths the noise in this layer“; Sharpen „Fine adjustment of this layer; the slider sets the main strength“;
  Slider „Strength of layer n“; Run Do All automatically „Applies the sharpening to the whole image half a second after each change“;
  Keep layer values for a new stack „Keeps the layer values when a new stack is made“; Layer spacing „Linear: the layers grow evenly.
  Dyadic: each layer doubles in size“; First layer size / Size step „Size of the finest layer and how much each further layer grows“;
  Wavelet filter „Gaussian is smoother than Default“; Move all sliders together „Moving one slider moves all the others by the same
  amount“; Load/Save Scheme „… a wavelet scheme file (.rwv)“; Adjust „Show or hide the panel with color, tone and other image
  adjustments“; Histogram „Black point, white point and brightness of the image“; Gamma „Brightens or darkens the midtones“; RGB Align
  „Shifts red, green and blue so that they lie on top of each other“; RGB Balance „Changes the share of red, green and blue“; Remember
  „Remembers the current image so that it can be compared later“; Restore „Brings the remembered image back“; Difference „Shows where
  the current image differs from the remembered one, amplified“; Before/After „Switches between the stack without any sharpening and
  the sharpened image“; Clip (%) „Share of the darkest and brightest pixels that Auto may cut off“; Difference (Legende Quality Graph)
  „How much a frame differs from the frame all the others are lined up on; higher is worse“; Dateiinfo Codec „How the frames are
  stored in the file“, Bit depth „Brightness levels per pixel“. Locked (Tooltip beginnt mit „Not available yet“): bei Stretch
  histogram, De-rotate images, Resampling, Drizzling zusätzlich eine Erklärung („Spreads the brightness over the full range after
  stacking“, „Turns the frames back against the rotation of the planet“, „Enlarges the result“, „Enlarges the result by combining
  shifted frames“).

  **Tabelle 3: Bedienelemente ohne Wirkung → sperren (gedimmt, „Not available yet“).** (a) Show-Menü: *Show Full Image* (in allen drei
  Varianten), *Show Align Data*, *Show Prefilter*, *Show Stack Graph*, *Show Stack List*: im Menü unter einem Abschnittskopf „Not
  available yet“, nicht anwählbar (Mac: `ShowItem.isAvailable`, `mac/App/Model/ShowMenuLogic.swift`). (b) Align, Advanced: *Show
  Alignment*. (c) Stack, Advanced: *Codec*, *Compression quality*, die Radios *Smallest common area / Largest area / Region* unter
  Frame size (nur *Original size* wirkt, immer gewählt), *Use prefilters*. (d) Framelist-Karte: *Clear List*. (e) Wavelets, Advanced:
  *Stack Again* bekommt den Tooltip „Not available yet“ (war gesperrt ohne Hinweis). (f) Panel Adjustments: die elf Knöpfe ohne Karte
  (*Color Mixing, View Zoomed, View Compare, View Stack Size, Flip and Rotate, Resize Image, Denoise/Deringing, Wavelet Filter,
  Masking, Show Line Graph, Cropping Area*) sind gedimmt und nicht klickbar; nur *Histogram, Gamma, RGB Align, RGB Balance* bleiben
  aktiv. (g) Contrast/Brightness: der Link *Keep* (früher *Hold*) hat in der Mac-App keine Wirkung (der Zustand wird nur gespeichert)
  und ist gesperrt; Windows prüft, ob *Hold* dort etwas tut, sonst ebenfalls sperren. Schon gesperrt und unverändert: Stretch histogram,
  De-rotate images, Resampling, Drizzling, Compressed, Estimate rotation, Align by center of gravity, Brightness threshold, die Liste
  *Saved Schemes*, Realign with Processed, Alignment mode (No Align · Scan Frames), Keep points inside the scan frame, der Preview-Knopf
  der Wavelet-Ebenen.

  **Sachfehler Align-Ergebnis.** Die Ergebniszeile des Schritts Align (Panel-Fuß, Pille, Tooltip) zeigte „best quality 100 %“. Die
  Qualität ist aber auf den besten Frame normiert (`qualityPercent`), der beste hat also immer 100 %: die Angabe war leer. Neu:
  **„‹n› frames rated“** (n = Zahl der bewerteten Frames). Windows: `app/src/logic/workflow.ts` bzw. wo die Ergebniszeile des Schritts
  Align gebildet wird.

  **the reference program-Erwähnungen in Windows-Texten (Stand dieses Commits, `grep -rn -i "the reference program" app/src app/src-tauri/src`).** Sichtbar
  und zu ändern: `app/src/wavelet.ts:142`: Filtername des Dateidialogs „the reference program Wavelet scheme settings“ → „Wavelet scheme“. Alle
  anderen Treffer sind Kommentare, Bezeichner (`planets_session`, `planets_app_lib`) oder Testbeschreibungen und bleiben:
  `app/src/align.ts:4`, `app/src/components/AlignStep.vue:24`, `NumberField.vue:5`, `Toolbar.vue:128,147`,
  `app/src/logic/{align.ts:96, colour.ts:113, exporting.ts:1, settings.ts:1,2,29,125, wavelet.ts:81, workflow.ts:18}`,
  `app/src/recording.ts:9,112`, `app/src/wavelet.ts:180`, `app/src-tauri/src/export_commands.rs:21,41`; Tests:
  `app/src/logic/{align,stack,settings,numbers,workflow}.test.ts`. Dazu die Windows-Beschriftungen aus den Tabellen oben (alle Texte
  der Spalte „alt“, die in `app/src/components/*.vue`, `app/src/logic/*.ts`, `app/src/App.vue` stehen). Der Windows-Installer
  (`tauri.conf.json`) nennt the reference program nirgends.

## Auslastung in der Statusleiste (Mac: MAC-015)

- [ ] Ganz links in der Statusleiste, vor dem Fortschrittsbalken, die Auslastung der App wie in Siderea: **„CPU (n)“** mit der CPU-Last
      des Prozesses als **Anteil aller n logischen Kerne** (Prozess-CPU-Zeit je Sekunde Wanduhr ÷ n, ganze Prozent, auf drei Stellen mit
      Leerzeichen aufgefüllt: `  7 %`, ` 50 %`, `100 %`, bei 100 gedeckelt; bei einem fehlenden oder nicht endlichen Wert ein „–“) und
      **„RAM“** mit dem Arbeitsspeicher des Prozesses (Windows: Working Set; Mac: `ri_phys_footprint`) in den Einheiten B, KB, MB, GB, TB zu
      je 1024: bis 1 KB ohne Nachkommastelle, ab 100 ohne, dazwischen eine, höchstens 8 Zeichen („1.5 GB“, „250 MB“). Ab 90 % aller Kerne
      ist der CPU-Wert `--accent` und fett. Tooltips: „CPU load of Siderea Planets as a share of all n processor cores. 100 % means every
      core is fully busy.“ und „Memory Siderea Planets currently holds.“ Bei Platzmangel fällt zuerst RAM weg, nie Umbruch.
      Gemessen wird einmal pro Sekunde, **nur solange das Fenster sichtbar ist** (nicht minimiert, nicht im Hintergrund versteckt).
- [ ] **Settings:** Schalter **Show CPU and memory usage** (Standard an); aus, verschwindet das Feld samt Trennlinie und die Messung stoppt.
      Mac: `mac/App/Monitor/` (`ProcessUsage.swift` Formate und Schwelle, `ResourceMonitor.swift`, `DarwinProcessSampler.swift`),
      `mac/App/Views/ResourceReadout.swift` (`ResourceField` in `StatusBar`), `mac/App/Settings/ResourceSettings.swift`; die Rechenregeln
      sind in `mac/Tests/SidereaPlanetsTests/ProcessUsageTests.swift` festgehalten. Die Anwenderseite `window-at-a-glance.md` nennt die Anzeige
      schon.

## Adjustments: Color Mixing und View Compare (Mac: MAC-019)

- [ ] **Color Mixing.** Der Knopf *Color Mixing* im Panel Adjustments öffnet eine Karte mit einer 3 × 3 Matrix: je Ausgabe (**Red output**,
      **Green output**, **Blue output**) drei Zeilen für die Eingangskanäle Red, Green, Blue mit Regler und Feld (−200 … 200, „%“ hinter
      dem Feld, Start 100 auf der Diagonale, sonst 0), oben die Zeile **Preset** mit dem Menü *Choose* (Neutral · Swap Red and Blue ·
      Luminance Gray · Red to All Channels), unten das Häkchen **Preserve brightness** (Tooltip „Keeps the brightness of every pixel while
      the colors change.“), in der Titelzeile **Reset**. Gerechnet wird im gemeinsamen Kern (**schon auf `main`**: `planets_core::color::mix_channels`,
      `ColorSettings.mix` der Sitzung mit `#[serde(default)]`, die Tauri-Befehle nehmen das Feld schon an): Windows muss die Karte bauen und
      `mix: { matrix: [9 Zahlen, zeilenweise], preserveBrightness }` in `ColorSettings` mitschicken (in `app/src/logic/colour.ts`:
      `isNeutral` kennt die Matrix, `colourPayload` sendet sie). Reihenfolge im Kern: RGB Align, **Color Mixing**, RGB Balance, Histogram,
      Gamma, Contrast/Brightness. Die Karte gilt auch für Graustufenbilder (nicht gesperrt). Projektdatei: Abschnitt `wavelet.tone.mix`
      mit `matrix` und `preserveBrightness`, beim Lesen nur übernommen, wenn es neun endliche Zahlen sind. Tooltip des Knopfs: „Builds each
      color channel from a mix of red, green and blue.“ Mac: `mac/App/Views/Wavelets/ColorMixingCard.swift`, `mac/App/Model/ColorMixLogic.swift`,
      `mac/App/Model/WaveletProject.swift`.
- [ ] **View Compare.** Der Knopf *View Compare* (Tooltip „Splits the image: the stack on the left, the result on the right.“) ist ein
      Umschalter ohne Karte: das Bild im Schritt Wavelets wird an einer ziehbaren senkrechten Linie geteilt, links das Summenbild
      (Beschriftung **Before** unten links), rechts das Ergebnis mit Schärfung und Karten (**After** unten rechts), beide mit gleichem
      Zoom und gleicher Lage. Die Linie liegt bei einem Anteil (Start 0,5, Grenzen 0,02 … 0,98) der Breite des Bildbereichs und bleibt
      beim Zoomen stehen; eine Beschriftung verschwindet, wenn weniger als 84 px bis zum Rand bleiben; oben links entfällt die Beschriftung
      „Sharpened stack“ während der Teilung. **Before/After** in der Zoom-Leiste beendet die Teilung (zeigt das Summenbild ganz), und der
      Knopf View Compare schaltet *Before* aus: nie beides zugleich. Nur im Schritt Wavelets. Der Knopf steht in `wavelet.functions` der
      Projektdatei, die Lage der Linie nicht. Rein Ansicht, kein Kernaufruf. Maße und Farben: `docs/design.md`, „Bildbereich“. Mac:
      `mac/App/Views/CompareOverlay.swift`, `mac/App/Views/ImageCanvas.swift` (`beforeImage`, `divider`), `mac/App/Model/AppModel+Compare.swift`,
      `mac/App/Model/CompareLogic.swift`.
- [ ] **Die Zahl der gesperrten Adjustments-Knöpfe** sinkt von elf auf sieben (mit MAC-018 Denoise/Deringing und Masking; Texte in den Hilfeseiten `color-and-tones.md` und
      `step-wavelets-save.md` schon angepasst); in der Windows-App ist die Liste der gesperrten Knöpfe ebenso zu kürzen.

## Denoise/Deringing und Masking (Mac: MAC-018)

- [ ] Die beiden Adjustments-Schaltflächen **Denoise/Deringing** und **Masking** öffnen je eine Karte (Aufbau und Werte in `docs/design.md`,
      Tabelle „Karten hinter den Functions-Schaltflächen“; die Schaltflächen sind bisher gesperrt, „Not available yet“, und `isToneCard` kennt sie nicht).
      **Denoise/Deringing:** drei Zeilen mit Feld (40 px) und Regler darunter, ganze Zahlen: **Denoise (%)** 0 · 0 … 100, **Deringing (%)**
      0 · 0 … 100, **Radius (px)** 4 · 1 … 10; Hinweis „Deringing compares the sharpened image with the stack“; **Reset**. **Masking:** Häkchen
      **Use mask** (aus), **Threshold** 20 · 0 … 255, **Feather (px)** 4 · 0 … 50, Häkchen **Invert** und **Show mask** (beide aus); **Reset** setzt alles
      zurück und nimmt auch Show mask heraus. Tooltips: siehe `functionHelp` und `RefineCards.swift`.
- [ ] **Show mask** ersetzt im Bildbereich das Bild durch das Bild mit blau eingefärbter Maske (`Session::refine_mask_overlay`, nur solange Show mask
      an **und** die Karte Masking offen ist und der Schritt Wavelets gezeigt wird); *Before* und die Ansichten Remembered image und Difference haben
      Vorrang. Nur Ansicht: nicht gespeichert, nicht in Save image.
- [ ] Die Werte gehen als `refine` in `ColorSettings` (`crates/session/src/color.rs`, `#[serde(default)]`: die Windows-Befehle laufen unverändert weiter) an
      `apply_colors`, `color_histogram`, `auto_histogram_points` und `save_image`; Reihenfolge **vor** allen anderen Karten: Deringing, Denoise, Maske. Die
      Histogramm-Anfrage der Karte hängt auch von diesen Werten ab (Mac: `HistogramKey.refine`). Neutral (`RefineSettings::is_neutral`: Stärken 0, Maske aus)
      zählt für „Save image schreibt ohne Karten“ mit (`isNeutral`).
- [ ] Projektdatei: eigener optionaler Abschnitt `refine` (`denoise`, `deringing`, `radius`, `maskOn`, `threshold`, `feather`, `invert`; ganze Zahlen, auf die Grenzen
      gekürzt), die Abschnitte `wavelet` und `colour` bleiben unverändert; ein Projekt ohne den Abschnitt lässt die Werte, wie sie sind. Mac:
      `mac/App/Model/RefineLogic.swift` (`refineSection`, `applyRefineSection`), `AppModel.projectSections`.
- [ ] Verfahren und Konstanten stehen im Rechenkern (`crates/core/src/refine.rs`, beschrieben in `docs/wavelets.md`, Abschnitt „Denoise/Deringing und Masking“);
      Windows braucht nur die Oberfläche. Mac: `mac/App/Views/Wavelets/RefineCards.swift`, `mac/App/Model/AppModel+Wavelets.swift` (Show mask),
      Anwenderseiten (`color-and-tones.md`, `step-wavelets-save.md`, Glossary) liegen schon gemeinsam auf `main`.

## Zuschneiden vor dem Speichern (Mac: MAC-016)

- [ ] **Werkzeug Crop** für das Speichern des geschärften Bildes (Schritte Wavelets und Save). Die Schaltfläche **Cropping Area** des
      Panels Adjustments heißt **Crop** und ist nicht mehr gesperrt (sie war es nach MAC-014, Tabelle 3 (f), zusammen mit zehn weiteren;
      mit MAC-019 und diesem Punkt sperrt Windows nur noch **sechs** Knöpfe: View Zoomed, View Stack Size, Flip and Rotate, Resize Image,
      Wavelet Filter, Show Line Graph; mit MAC-018 sind Denoise/Deringing und Masking frei; mit MAC-017 auch Flip and Rotate und Resize Image,
      es bleiben **vier**: View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph). Im Schritt **Save** kommt über Save Image die Gruppe **Crop** („The whole picture is saved.“ /
      „Only 800 × 600 px is saved.“) mit **Crop…** und, solange ein Zuschnitt gilt, dem Link **Remove Crop** dazu; beides öffnet dasselbe
      Werkzeug.
- [ ] **Verhalten:** Rechteck über dem Bild (Außenseite abgedunkelt, Rand, Drittel-Raster, **kleines Fadenkreuz in der Mitte**, acht Griffe,
      Größe „W × H“ auf einer Platte), Karte **Crop** mit den **Formaten Free · 1:1 · 3:2 · 4:3**, Größe in Pixeln, **Reset**, **Cancel** und
      **Done**. Ziehen im Rechteck verschiebt, an Kante und Ecke ändert die Größe, außerhalb zeichnet ein neues; ein festes Format bleibt
      beim Ziehen erhalten (liegend oder stehend, wie das Rechteck gerade ist, **kein Umschalter**), ein anderes Format formt das Rechteck um
      (gleiche Mitte und Fläche); Mindestgröße 8 × 8 Bildpunkte, nie außerhalb des Bildes. **View Compare geht beim Öffnen des Werkzeugs aus und ist gesperrt, solange es offen ist** (Statuszeile „View Compare: finish or cancel the crop first“); Start auf der Mitte (80 % jeder Seite) oder dem
      angewendeten Rechteck; ein Rechteck, das das ganze Bild deckt, ist kein Zuschnitt. Alle Maße, Deckkräfte und Texte stehen in
      `docs/design.md`, „Zuschneiden (Mac-016)“; die Geometrie (Ziehen, feste Formate, Klemmen) ist reiner Code ohne Oberfläche in
      `mac/App/Model/CropRectangle.swift` und die Tests dazu in `mac/Tests/SidereaPlanetsTests/CropRectangleTests.swift`, Zustand und Regeln in
      `mac/App/Model/AppModel+Crop.swift` und `CropLogic.swift`; Zeichnen in `mac/App/Views/CropOverlay.swift`, Ziehen in
      `mac/App/Views/ImageCanvas.swift`, Karte und Gruppe in `mac/App/Views/Save/CropViews.swift`.
- [ ] **Wirkung:** Der Zuschnitt gilt für **jedes Format** von Save Image im Schritt Wavelets und Save (das Bild des Schritts Stack wird immer
      ganz geschrieben), wird **nach** den Farb- und Tonwerten ausgeschnitten und ist bis zum nächsten Stack oder einer anderen Aufnahme
      gültig. Er steht **nicht in der Projektdatei** (das Stack-Bild auch nicht). **Rechenkern schon gemeinsam auf `main`:**
      `planets_core::crop::CropRect` (`clamp`, `is_full`) und `Image::cropped`, `Session::save_image(…, crop: Option<CropRect>)`; die Tauri-Befehle
      `save_image` und `save_image_as` reichen vorerst `None` durch (`app/src-tauri/src/export_commands.rs`), Windows ergänzt dort das
      Argument `crop` (`{x, y, width, height}`) und im Frontend `app/src/saving.ts`/`exporting.ts` den Aufruf. Die Anwenderseiten
      (`save-and-export.md`, `step-wavelets-save.md`, `color-and-tones.md`, `glossary.md`) beschreiben das Werkzeug schon.

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
