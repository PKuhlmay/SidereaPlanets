# Oberfläche des Referenzprogramms

Beobachtet am Referenzprogramm, Version 6.1.0.0, unter Windows 11 (Skin „Office 2007 Silver“). Die Werte wurden aus den laufenden
Bedienelementen ausgelesen und mit Screenshots abgeglichen. Screenshots und Steuerelement-Listen liegen nur lokal in
`re-local/screenshots/`. Der Klon übernimmt **Aufbau und Begriffe**, nicht den Look (SP-010).

**Namen in Siderea Planets (MAC-014).** Dieses Dokument beschreibt die Oberfläche des Referenzprogramms mit dessen Begriffen. Wo die App
einen Bedienbegriff anders nennt, steht er in der Spalte „Name in Siderea Planets“ neben dem Begriff des Referenzprogramms, der unverändert
bleibt; Listen nennen den App-Namen in „(in der App: …)“. Die App-Namen folgen MAC-014 und der Zuordnung in
`docs/parity/WINDOWS_PARITY.md` (Abschnitt „Verständliche Texte für Einsteiger“). Regeln dort: kein Name des Referenzprogramms und kein „das
Original“ in sichtbaren Texten der App, US-Englisch, Title Case für Schaltflächen, Menüs und Gruppenüberschriften, Sentence case
für Beschriftungen, Häkchen und Tooltips, Einheiten am Feld. Ein „—“ in der Spalte heißt: Name unverändert.

Legende: *Standard* = Wert direkt nach der Installation. „Bereich“ steht nur da, wo er beobachtet wurde.
Zahlenfelder (Spin-Felder) übernehmen beim Tippen auch Werte außerhalb sinnvoller Grenzen (z. B. Denoise −5 oder 99)
ohne zu korrigieren. Ihre Grenzen gelten deshalb als unbekannt, siehe `docs/offene-fragen.md`.

## Hauptfenster

| Bereich | Inhalt | Name in Siderea Planets |
|---|---|---|
| Titelleiste | „‹Programmname› V6“, nach dem Laden „‹Programmname› processing AVI: ‹Pfad›(‹Codec› vids) (‹B›x‹H›)“ | „Siderea Planets“, mit Aufnahme „Siderea Planets · ‹Dateiname›“, während eines Schritts „Siderea Planets · Align 40%“ |
| Werkzeugleiste | **Select** · **MRU ▾** · **Flat/Dark/Reference ▾** · **Tools ▾** · **Settings ▾** · **Cancel** · **Pause** · **About** · **CPUs:** [Zahl, Standard 1; im Klon Standard = Zahl der logischen Kerne, Bereich 1 bis dahin, siehe Abweichungen in `design.md`] | **Select Recording** · **Recent Recordings ▾** · **Flat/Dark/Reference ▾** · **Tools ▾** · **Settings ▾** · **Open Project…** · **Save Project…** · **Cancel** · **Pause** · **Help** · **About** · **CPU cores to use** |
| Reiterleiste | **Align** · **Stack** · **Wavelet**, rechts daneben „File Version: 6.1.0.0“, Datum und „Memory Used/Free/Total: …Mb“ | Schrittleiste statt Reiter: **Open · Alignpoints · Align · Best Frames · Stack · Wavelets · Save**, rechts „1024×680 · 10 frames · DIB“ |
| Bildbereich | Vorschau mit Scrollbalken. Je nach Reiter liegen Punkte oder Rahmen darüber | Bildbereich mit **Show ▾** und im Wavelets-Schritt **Adjust ›**; leer: „Select a recording or drop a video or images here“ / „No image to show yet“ |
| Statusleiste | Fortschrittsbalken in %, aktuelle Aktion („Loading frame: 10“), Ergebnis („Alignment finished.“), „Time used / Time left“ | „Aligning frame: 42“, „Alignment finished“ (ohne Punkt), „Time used / Time left“ |
| Taskleiste | Fortschritt im Fenstertitel der Anwendung, z. B. „R 6:Align 100%“ | Fenstertitel „Siderea Planets · Align 40%“ |

## Menüs

**MRU**: zuletzt geöffnete Dateien (im Test leer). In der App: **Recent Recordings** (leer: „No recent recordings“).

**Flat/Dark/Reference**

| Eintrag | Zustand vor dem Laden | Anmerkung |
|---|---|---|
| Load Darkframe | aktiv | |
| Use Darkframe | gesperrt | |
| Use Raw-Darkframe | gesperrt | |
| Create Darkframe | gesperrt | darunter Zahlenfeld, Standard 1,00 |
| Load Flatfield | aktiv | |
| Use Flatfield | gesperrt | |
| Create Flatfield | gesperrt | |
| Load Referenceframe | aktiv | |
| Use Referenceframe | gesperrt | |

**Tools**: Show MACRO/Batch window · Record Processing Information in Filename

**Settings**

| Eintrag | Art | Standard / Auswahl | Name in Siderea Planets |
|---|---|---|---|
| Automatic Processing ▸ | Untermenü | Align to Wavelet · Align to Stack · Stack to Wavelet (alle aus) | — |
| Autodetect Colour/BW | Häkchen | aus | Autodetect Color/BW |
| Processing Area | Auswahl | 2048 · 1024 · **512** · 256 · 128 | **Preview area** (Werte in Pixel je Seite, Mac „512 × 512 px“) |
| Luminance Setting ▸ | Untermenü | Reset Luminance; Red **0,299**, Green **0,587**, Blue **0,114** | — |
| Stretch FIT/TIFF | Auswahl | **Always Ask** · Yes · No | — |
| Show Advanced Controls | Häkchen | aus | — |
| Play Sounds | Häkchen | aus | — |
| Swap Red/Blue Channel | Häkchen | aus | — |
| Wait when loading a single image | Häkchen | aus | — |
| Maximize during messages | Häkchen | aus | — |
| Check for ‹Programmname› Update | Häkchen | aus (im Klon weglassen) | fehlt |
| Layouts ▸ / Set Skin ▸ | Untermenü | Fensterlayout und Skin (im Klon weglassen) | fehlt; **Theme** (Dark · Light, Mac: folgt macOS) ersetzt Set Skin |
| Save to Subdir | Textfeld | „Subdirname“ | — |
| Bias sutract | Zahlenfeld | 0 (Schreibfehler im Original, im Klon „Bias subtract“) | — |

## Align-Reiter

**Kopfzeile**

- Schaltflächen: **Set Alignpoints** · **Align** · **Limit** (in der App: Schritt **Best Frames**, Schaltfläche **Keep Best Frames**, erledigt „Keep Best Frames Again“). Jeweils nur der nächste mögliche Schritt ist aktiv.
- Häkchen: Colour · LRGB · Show Full Image · Show ROI · Show Alignpoints · Show Aligndata · Show Framelist ·
  Show Registrationgraph · Show Prefilter
  (in der App im Menü **Show ▾**: Show Full Image · Show Region · Show Alignpoints · Show Align Data · Show Frame List ·
  Show Quality Graph · Show Prefilter; Show Full Image, Show Align Data und Show Prefilter sind gesperrt, **Not available yet**;
  Colour und LRGB heißen **Color** und **Sharpen brightness only** und liegen im Wavelets-Schritt)

**Panel „Alignment“ (links)**

| Gruppe | Element | Art | Standard | Beobachtet | Name in Siderea Planets |
|---|---|---|---|---|---|
| Set Alignpoint parameters | Minimum distance between | Zahl | 30 | | Min. distance between points (px) |
| | Min distance from edge | Zahl | 20 | | Min. distance from image edge (px) |
| Intensity_selection | Default · 3x3 area · Lowest pixelvalue | Auswahl | Default | | Gruppe **Point Placement**; **Brightness measure**: Default · 3×3 area · Lowest pixel value |
| | Lo | Zahl | 30 | | **Min. brightness (0–255)** (eigene Zeile) |
| | Hi | Zahl | 230 | | **Max. brightness (0–255)** (eigene Zeile) |
| | Threshold (weakest … strongest) | Regler | | vor dem Laden 0..0; nach Set Alignpoints 0..91, Position 9 | **Threshold**: more points … fewer points; danach „Threshold · 82 points“ |
| | Number of Alignpoints | Zahl | 0 | gesperrt; zeigt nach dem Setzen 82 | Number of alignpoints |
| | Keep Alignpoints inside ScanFrame | Häkchen | aus | gesperrt | Keep points inside the scan frame (gesperrt) |
| Alignment setup | Show Alignment | Häkchen | aus | | Show Alignment (gesperrt) |
| | No Align · R.o.Interest · Scan Frames | Schaltflächen | | No Align und R.o.Interest nach dem Laden aktiv, Scan Frames nach Align | **No Align · Scan Frames** (gesperrt, unter Advanced, Beschriftung „Alignment mode“); R.o.Interest heißt **Draw Region** (Schritt Alignpoints) |
| | Alignmentbox size | Zahl | 30 | | **Box size per point (px)** |
| | Max Alignpoint movement | Zahl | 5 | | **Max. point shift (px)** |
| | Align by Centre of gravity | Häkchen | aus | gesperrt | Align by center of gravity (Mac: frei seit MAC-021, eigene Umsetzung; Windows gesperrt) |
| | Lum. Threshold | Zahl | 0 | gesperrt | Brightness threshold, 0–255, Standard **20** (Mac: frei seit MAC-021, nur mit dem Häkchen aktiv; Windows gesperrt) |
| | Estimate Rotation | Häkchen | aus | gesperrt | Estimate rotation (gesperrt) |
| Limit Setup | Lowest Quality (%) · Best Frames (%) · Frames/Apoint · Best Frames | Auswahl | Lowest Quality (%) | | **At least % of the best quality** · **Best % of frames** · Best frames per point · **Best number of frames**; Standard At least % of the best quality; Gruppe im Schritt **Best Frames** (Original: Limit Setup) |
| | Prozentwert | Zahl | 80 | | — |
| | Anzahl | Zahl | 200 | gesperrt bei Prozent-Modi | — |

**Fußzeile**: Regler über alle Frames (bei 10 Frames: 0..9) · *Goto Frame* (Zahl, Standard 1) · „Frame (n): n/N“.
In der App: **Go to frame** und „Frame 5 of 10“.
Links ein Farbfeld: hellgrün nach dem Laden, dunkelgrün nach Align. Die Bedeutung ist nicht geklärt.

**Schwebende Panels**
- *Framelist*: Check all · Check none · Empty List; Liste „Frame00001“ … mit Häkchen, nach Align nach Qualität sortiert.
  In der App: **Frame List** mit **Check All · Check None · Clear List** (gesperrt) und „Frame 1“ …
- *RegiStrationgraph*: Liniendiagramm über die Frames, Linien „Quality“ und „Difference“. In der App: **Quality Graph**.

## Stack-Reiter

**Kopfzeile**

- Schaltflächen: **Stack** · **Save image** (in der App: **Save Image**)
- Häkchen: Colour · LRGB · Show Full Image · Show AlignSections · Show Stackgraph · Show Stacklist ·
  **Resampling** (eingeschaltet erscheinen Methode Bell · Bspline · **Lanczos** · Mitchell und Faktor **2,0**) ·
  **Drizzling** (eingeschaltet erscheint Faktor **0,5**; laut Tooltip wird das Ergebnis doppelt so groß)
- *Show AlignSections* öffnet die Liste *Alignmentpoints* („Point 1“ … mit Häkchen).
  In der App: Menü **Show ▾** mit Show Full Image · **Show Alignpoints** (öffnet die Karte **Alignpoints Used for Stacking**,
  „Alignpoint 1“ …) · Show Stack Graph · Show Stack List; Full Image, Stack Graph und Stack List sind gesperrt.

**Panel „Stacking options“**

| Element | Art | Standard | Name in Siderea Planets |
|---|---|---|---|
| StackSize: Default · Minimized · Maximized · R.of Interest | Auswahl | Default | **Output size**: Default · Smallest common area · Largest area · Region (nur Default wirkt) |
| Use Nearestby Alignpoints | Zahl | 3 | **Use nearest points** |
| Normalization of frame intensity | Häkchen | aus | **Even out frame brightness** |
| Correct geometry | Häkchen | **an** | — |
| Stretch histogram after stacking | Häkchen | aus | — |
| De-rotate images | Häkchen | aus | — |
| Do not stack ‹n› pixels at the frameedge | Zahl | 0 | **Ignore edge pixels (px)** |

**Panel „Create AVI“**

| Element | Art | Standard | Name in Siderea Planets |
|---|---|---|---|
| Codec | Auswahl | DIB (dazu die auf dem System installierten Video-for-Windows-Codecs; im Test cvid, i420, iyuv, mrle, msvc, uyvy, yuy2, yvu9, yvyu) | Codec (gesperrt) |
| Compress Qual | Zahl | 7500 | **Compression quality** (gesperrt) |
| Framerate | Zahl | 25 | **Frame rate (fps)** |
| Create aligned Sequence: **Save Aligned** | Schaltfläche | | Aligned video: **Save Aligned Video…** |
| Image size: Original image size · Minimum area · Maximum area · Region of Interest | Auswahl | Original image size | Überschrift **Frame size**: **Original size** · Smallest common area · Largest area · Region (nur Original size wirkt, die anderen gesperrt) |
| Sort On Quality | Häkchen | aus | Sort by quality |
| Use Prefilters | Häkchen | aus | Use prefilters (gesperrt) |
| Create integrated sequence: **Save Integrated** | Schaltfläche | | Averaged video: **Save Averaged Video…** |
| Method: Moving average · Compressed | Auswahl | Moving average (Compressed gesperrt) | Method: Moving average · Compressed (gesperrt) |
| Average of ‹n› Frames | Zahl | 5 | Average of [n] frames |

## Wavelet-Reiter

**Kopfzeile**

- Schaltflächen: **Process** · **Do All** · **Save image** · **Realign_with Processed** · **Stack Again**
  (gesperrt) · **»** (klappt das rechte Funktionsfenster ein und aus)
  (in der App: **Update Preview** · **Do All** · **Save Image** · Realign with Processed · Stack Again, beide gesperrt · **Adjust ›**)
- Häkchen: Show Full Image · Show AlignPoints · Show Processing Area
  (in der App: Show Full Image (gesperrt) · Show Alignpoints · **Show Preview Area**)

**Panel „Wavelets“ (links)**, Kopf mit **Reset Wavelets**. In der App heißt die Gruppe unter Advanced **Wavelet Settings**.

| Element | Art | Standard | Beobachtet | Name in Siderea Planets |
|---|---|---|---|---|
| Automatic | Häkchen | aus | | **Run Do All automatically** |
| Hold Wavelet Setting | Häkchen | aus | | **Keep layer values for a new stack** |
| Waveletscheme: Dyadic (2^n) · Linear | Auswahl | **Linear** | | **Layer spacing**: Dyadic (2^n) · Linear |
| Initial Layer | Zahl | 1 | | **First layer size** |
| Step Increment | Zahl | 0 | | **Size step** |
| Wavelet filter: Default · Gaussian | Auswahl | **Gaussian** | | — |
| Use Linked Wavelets | Häkchen | aus | | **Move all sliders together** |
| Ebene 1 … 6: Häkchen „n : 1“ | Häkchen | an | Beschriftung zeigt _vermutlich_ Ebene und Skalierung | — |
| Ebene n: **Denoise** | Zahl, 2 Nachkommastellen | 0,00 | im Schema als ×100 gespeichert | — |
| Ebene n: **Sharpen** | Zahl, 3 Nachkommastellen | 0,100 | im Schema als ×100 gespeichert | — |
| Ebene n: **Preview** | Schaltfläche | „1.0“ | zeigt den Reglerwert ÷ 10 | Preview (gesperrt, Auge-Symbol); der Sliderwert steht als Text rechts neben dem Regler |
| Ebene n: Regler | Regler | Position 10 (= 1.0) | Bereich −50 … 1000, also −5.0 … 100.0; Pfeiltaste ändert um 1 | — |
| Available schemes | Auswahl | leer | | **Saved Schemes** (gesperrt) |
| **Load Scheme** · **Save Scheme** | Schaltflächen | | Dateiformat in `docs/formats.md` | **Load Scheme… · Save Scheme…** |

Die Statusleiste zeigt beim Zeigen ins Bild die Position und Kanalwerte, z. B.
„X=200 Y=201 Stack=9 PT=18 RGB=raw(1 1 1) user(1 1 1)“.

**Panel „Functions“ (rechts)** (in der App: **Adjustments**; die Schaltflächen heißen Color Mixing · Denoise/Deringing · View Stack Size ·
Show Line Graph, nur Histogram, Gamma, RGB Align und RGB Balance sind aktiv, die elf übrigen gesperrt), Schaltflächen in einem 3er-Raster:
Histogram · Gamma · Colour Mixing · View Zoomed · View Compare · View Stacksize · Flip and Rotate · RGB Align ·
RGB Balance · Resize Image · Denoise/Deringing · Wavelet Filter · Masking · Show Linegraph · Cropping Area.
Jede Schaltfläche öffnet ein eigenes Panel. Die Panelnamen stehen in `docs/formats.md`, Abschnitt Layout.
Die Panels wurden noch nicht einzeln erfasst, siehe `docs/offene-fragen.md`.

**Panel „Contrast/Brightness“**, Kopf mit Hold · Reset (in der App: **Keep**, gesperrt, · Reset)

| Element | Art | Standard | Beobachtet | Name in Siderea Planets |
|---|---|---|---|---|
| oberer Regler | Regler | Position 250 | Bereich 1 … 1000; Zuordnung zu Contrast unklar | — |
| unterer Regler | Regler | Position 0 | Bereich −512 … 512 | — |
| Contrast | Zahl | 100 | | **Contrast**: Beschriftung und Feld über dem Regler |
| Brightness | Zahl | 0 | | **Brightness**: Beschriftung und Feld über dem Regler |

**Panel „Copy To / Load to / Difference“**: Toggle *Current Image* · *Clipboard Image* (gesperrt, bis etwas
kopiert wurde). In der App: **Remember · Restore · Difference** und „Show:“ *Current image* · *Remembered image*; die Karten
tragen die Felder **Black point · White point · Clip (%)** (Histogram), **X (px) · Y (px) · Max shift (px)** (RGB Align) und
Prozentfelder mit „%“ (RGB Balance).

## Rückfragen und Meldungen

| Situation | Meldung (sinngemäß) | Antworten |
|---|---|---|
| Farbvideo geladen | Bild ist offenbar farbig, in Farbe verarbeiten? | Yes · No, dazu „nicht mehr fragen, automatisch erkennen“ |
| Fehler beim Laden (siehe offene Fragen) | Access violation … | OK |
