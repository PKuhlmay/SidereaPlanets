# Oberfläche von the reference program

Beobachtet an the reference program.1.0.0 unter Windows 11 (Skin „Office 2007 Silver“). Die Werte wurden aus den laufenden
Bedienelementen ausgelesen und mit Screenshots abgeglichen. Screenshots und Steuerelement-Listen liegen nur lokal in
`re-local/screenshots/`. Der Klon übernimmt **Aufbau und Begriffe**, nicht den Look (RK-010).

Legende: *Standard* = Wert direkt nach der Installation. „Bereich“ steht nur da, wo er beobachtet wurde.
Zahlenfelder (Spin-Felder) übernehmen beim Tippen auch Werte außerhalb sinnvoller Grenzen (z. B. Denoise −5 oder 99)
ohne zu korrigieren. Ihre Grenzen gelten deshalb als unbekannt, siehe `docs/offene-fragen.md`.

## Hauptfenster

| Bereich | Inhalt |
|---|---|
| Titelleiste | „the reference program V6“, nach dem Laden „the reference program processing AVI: ‹Pfad›(‹Codec› vids) (‹B›x‹H›)“ |
| Werkzeugleiste | **Select** · **MRU ▾** · **Flat/Dark/Reference ▾** · **Tools ▾** · **Settings ▾** · **Cancel** · **Pause** · **About** · **CPUs:** [Zahl, Standard 1; im Klon Standard = Zahl der logischen Kerne, Bereich 1 bis dahin, siehe Abweichungen in `design.md`] |
| Reiterleiste | **Align** · **Stack** · **Wavelet**, rechts daneben „File Version: 6.1.0.0“, Datum und „Memory Used/Free/Total: …Mb“ |
| Bildbereich | Vorschau mit Scrollbalken. Je nach Reiter liegen Punkte oder Rahmen darüber |
| Statusleiste | Fortschrittsbalken in %, aktuelle Aktion („Loading frame: 10“), Ergebnis („Alignment finished.“), „Time used / Time left“ |
| Taskleiste | Fortschritt im Fenstertitel der Anwendung, z. B. „R 6:Align 100%“ |

## Menüs

**MRU**: zuletzt geöffnete Dateien (im Test leer).

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

| Eintrag | Art | Standard / Auswahl |
|---|---|---|
| Automatic Processing ▸ | Untermenü | Align to Wavelet · Align to Stack · Stack to Wavelet (alle aus) |
| Autodetect Colour/BW | Häkchen | aus |
| Processing Area | Auswahl | 2048 · 1024 · **512** · 256 · 128 |
| Luminance Setting ▸ | Untermenü | Reset Luminance; Red **0,299**, Green **0,587**, Blue **0,114** |
| Stretch FIT/TIFF | Auswahl | **Always Ask** · Yes · No |
| Show Advanced Controls | Häkchen | aus |
| Play Sounds | Häkchen | aus |
| Swap Red/Blue Channel | Häkchen | aus |
| Wait when loading a single image | Häkchen | aus |
| Maximize during messages | Häkchen | aus |
| Check for the reference program Update | Häkchen | aus (im Klon weglassen) |
| Layouts ▸ / Set Skin ▸ | Untermenü | Fensterlayout und Skin (im Klon weglassen) |
| Save to Subdir | Textfeld | „Subdirname“ |
| Bias sutract | Zahlenfeld | 0 (Schreibfehler im Original, im Klon „Bias subtract“) |

## Align-Reiter

**Kopfzeile**

- Schaltflächen: **Set Alignpoints** · **Align** · **Limit**. Jeweils nur der nächste mögliche Schritt ist aktiv.
- Häkchen: Colour · LRGB · Show Full Image · Show ROI · Show Alignpoints · Show Aligndata · Show Framelist ·
  Show Registrationgraph · Show Prefilter

**Panel „Alignment“ (links)**

| Gruppe | Element | Art | Standard | Beobachtet |
|---|---|---|---|---|
| Set Alignpoint parameters | Minimum distance between | Zahl | 30 | |
| | Min distance from edge | Zahl | 20 | |
| Intensity_selection | Default · 3x3 area · Lowest pixelvalue | Auswahl | Default | |
| | Lo | Zahl | 30 | |
| | Hi | Zahl | 230 | |
| | Threshold (weakest … strongest) | Regler | | vor dem Laden 0..0; nach Set Alignpoints 0..91, Position 9 |
| | Number of Alignpoints | Zahl | 0 | gesperrt; zeigt nach dem Setzen 82 |
| | Keep Alignpoints inside ScanFrame | Häkchen | aus | gesperrt |
| Alignment setup | Show Alignment | Häkchen | aus | |
| | No Align · R.o.Interest · Scan Frames | Schaltflächen | | No Align und R.o.Interest nach dem Laden aktiv, Scan Frames nach Align |
| | Alignmentbox size | Zahl | 30 | |
| | Max Alignpoint movement | Zahl | 5 | |
| | Align by Centre of gravity | Häkchen | aus | gesperrt |
| | Lum. Threshold | Zahl | 0 | gesperrt |
| | Estimate Rotation | Häkchen | aus | gesperrt |
| Limit Setup | Lowest Quality (%) · Best Frames (%) · Frames/Apoint · Best Frames | Auswahl | Lowest Quality (%) | |
| | Prozentwert | Zahl | 80 | |
| | Anzahl | Zahl | 200 | gesperrt bei Prozent-Modi |

**Fußzeile**: Regler über alle Frames (bei 10 Frames: 0..9) · *Goto Frame* (Zahl, Standard 1) · „Frame (n): n/N“.
Links ein Farbfeld: hellgrün nach dem Laden, dunkelgrün nach Align. Die Bedeutung ist nicht geklärt.

**Schwebende Panels**
- *Framelist*: Check all · Check none · Empty List; Liste „Frame00001“ … mit Häkchen, nach Align nach Qualität sortiert.
- *RegiStrationgraph*: Liniendiagramm über die Frames, Linien „Quality“ und „Difference“.

## Stack-Reiter

**Kopfzeile**

- Schaltflächen: **Stack** · **Save image**
- Häkchen: Colour · LRGB · Show Full Image · Show AlignSections · Show Stackgraph · Show Stacklist ·
  **Resampling** (eingeschaltet erscheinen Methode Bell · Bspline · **Lanczos** · Mitchell und Faktor **2,0**) ·
  **Drizzling** (eingeschaltet erscheint Faktor **0,5**; laut Tooltip wird das Ergebnis doppelt so groß)
- *Show AlignSections* öffnet die Liste *Alignmentpoints* („Point 1“ … mit Häkchen).

**Panel „Stacking options“**

| Element | Art | Standard |
|---|---|---|
| StackSize: Default · Minimized · Maximized · R.of Interest | Auswahl | Default |
| Use Nearestby Alignpoints | Zahl | 3 |
| Normalization of frame intensity | Häkchen | aus |
| Correct geometry | Häkchen | **an** |
| Stretch histogram after stacking | Häkchen | aus |
| De-rotate images | Häkchen | aus |
| Do not stack ‹n› pixels at the frameedge | Zahl | 0 |

**Panel „Create AVI“**

| Element | Art | Standard |
|---|---|---|
| Codec | Auswahl | DIB (dazu die auf dem System installierten Video-for-Windows-Codecs; im Test cvid, i420, iyuv, mrle, msvc, uyvy, yuy2, yvu9, yvyu) |
| Compress Qual | Zahl | 7500 |
| Framerate | Zahl | 25 |
| Create aligned Sequence: **Save Aligned** | Schaltfläche | |
| Image size: Original image size · Minimum area · Maximum area · Region of Interest | Auswahl | Original image size |
| Sort On Quality | Häkchen | aus |
| Use Prefilters | Häkchen | aus |
| Create integrated sequence: **Save Integrated** | Schaltfläche | |
| Method: Moving average · Compressed | Auswahl | Moving average (Compressed gesperrt) |
| Average of ‹n› Frames | Zahl | 5 |

## Wavelet-Reiter

**Kopfzeile**

- Schaltflächen: **Process** · **Do All** · **Save image** · **Realign_with Processed** · **Stack Again**
  (gesperrt) · **»** (klappt das rechte Funktionsfenster ein und aus)
- Häkchen: Show Full Image · Show AlignPoints · Show Processing Area

**Panel „Wavelets“ (links)**, Kopf mit **Reset Wavelets**

| Element | Art | Standard | Beobachtet |
|---|---|---|---|
| Automatic | Häkchen | aus | |
| Hold Wavelet Setting | Häkchen | aus | |
| Waveletscheme: Dyadic (2^n) · Linear | Auswahl | **Linear** | |
| Initial Layer | Zahl | 1 | |
| Step Increment | Zahl | 0 | |
| Wavelet filter: Default · Gaussian | Auswahl | **Gaussian** | |
| Use Linked Wavelets | Häkchen | aus | |
| Ebene 1 … 6: Häkchen „n : 1“ | Häkchen | an | Beschriftung zeigt _vermutlich_ Ebene und Skalierung |
| Ebene n: **Denoise** | Zahl, 2 Nachkommastellen | 0,00 | im Schema als ×100 gespeichert |
| Ebene n: **Sharpen** | Zahl, 3 Nachkommastellen | 0,100 | im Schema als ×100 gespeichert |
| Ebene n: **Preview** | Schaltfläche | „1.0“ | zeigt den Reglerwert ÷ 10 |
| Ebene n: Regler | Regler | Position 10 (= 1.0) | Bereich −50 … 1000, also −5.0 … 100.0; Pfeiltaste ändert um 1 |
| Available schemes | Auswahl | leer | |
| **Load Scheme** · **Save Scheme** | Schaltflächen | | Dateiformat in `docs/formats.md` |

Die Statusleiste zeigt beim Zeigen ins Bild die Position und Kanalwerte, z. B.
„X=200 Y=201 Stack=9 PT=18 RGB=raw(1 1 1) user(1 1 1)“.

**Panel „Functions“ (rechts)**, Schaltflächen in einem 3er-Raster:
Histogram · Gamma · Colour Mixing · View Zoomed · View Compare · View Stacksize · Flip and Rotate · RGB Align ·
RGB Balance · Resize Image · Denoise/Deringing · Wavelet Filter · Masking · Show Linegraph · Cropping Area.
Jede Schaltfläche öffnet ein eigenes Panel. Die Panelnamen stehen in `docs/formats.md`, Abschnitt Layout.
Die Panels wurden noch nicht einzeln erfasst, siehe `docs/offene-fragen.md`.

**Panel „Contrast/Brightness“**, Kopf mit Hold · Reset

| Element | Art | Standard | Beobachtet |
|---|---|---|---|
| oberer Regler | Regler | Position 250 | Bereich 1 … 1000; Zuordnung zu Contrast unklar |
| unterer Regler | Regler | Position 0 | Bereich −512 … 512 |
| Contrast | Zahl | 100 | |
| Brightness | Zahl | 0 | |

**Panel „Copy To / Load to / Difference“**: Toggle *Current Image* · *Clipboard Image* (gesperrt, bis etwas
kopiert wurde).

## Rückfragen und Meldungen

| Situation | Meldung (sinngemäß) | Antworten |
|---|---|---|
| Farbvideo geladen | Bild ist offenbar farbig, in Farbe verarbeiten? | Yes · No, dazu „nicht mehr fragen, automatisch erkennen“ |
| Fehler beim Laden (siehe offene Fragen) | Access violation … | OK |
