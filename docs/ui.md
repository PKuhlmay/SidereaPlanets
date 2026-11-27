# Bedienelemente und Werte

Die Bedienelemente von Siderea Planets mit ihren Namen, Standardwerten und Bereichen, so wie die App sie zeigt (`app/src/components/`,
Standardwerte in `app/src/logic/settings.ts`). Aufbau und Aussehen legt `docs/design.md` fest, die Abfolge der Schritte
`docs/workflow.md`.

Namen und Schreibweise folgen den Regeln in `docs/design.md` (Abschnitt „Texte und Schreibweise“): US-Englisch, Title Case für
Schaltflächen, Menüs und Gruppenüberschriften, Sentence case für Beschriftungen, Häkchen und Tooltips, Einheiten am Feld.

Legende: *Standard* = Wert nach dem Start. Zahlenfelder übernehmen getippte Werte auch außerhalb sinnvoller Grenzen, ohne zu
korrigieren; ein Bereich gilt nur, wo er hier genannt ist. Elemente ohne Funktion sind gesperrt (auf 45 % gedimmt), ihr Tooltip
beginnt mit „Not available yet“; sie stehen unten in der jeweiligen Liste.

## Hauptfenster

| Bereich | Inhalt |
|---|---|
| Titelleiste | „Siderea Planets“, mit Aufnahme „Siderea Planets — ‹Dateiname›“, während eines Schritts „Siderea Planets · Align 40%“ |
| Werkzeugleiste | **Select Recording…** · **Recent Recordings ▾** · **Settings ▾** · **Open Project…** · **Save Project…** · **Cancel** · **Pause** (nach dem Anhalten **Resume**) · **Help** · **About** (Dialog mit Version, Lizenz, Quellcode und den Lizenzen der Bestandteile) · **CPUs** (Zahl der Kerne für alle Berechnungen) |
| Schrittleiste | **Open · Alignpoints · Align · Best Frames · Stack · Wavelets · Save**, rechts „1024×680 · 10 frames · DIB“ (Größe, Frame-Zahl, Codec) |
| Schrittpanel (links) | Kopf „Step n of 7“, Name und ein Satz des offenen Schritts; die wichtigen Einstellungen; **Advanced** (n more settings) mit dem Rest; die Aktion am Fuß |
| Bildbereich | Vorschau mit Zoom, Ziehen und Mausrad; oben links eine Kapsel („Frame 5 of 10“, „Stack of 8 frames“, „Sharpened stack“), oben rechts **Show ▾** und im Schritt Wavelets **Adjust ›**; leer: „Select a recording or drop a video or images here“ / „No image to show yet“ |
| Frame-Leiste | Statuspunkt „Loaded“ / „Aligned“ · **Go to frame** · Vor- und Zurück-Pfeile (← →) · Regler über alle Frames · „Frame 5 of 10“ |
| Statusleiste | optional „CPU (12)“ und „RAM“ · Fortschrittsbalken in % · aktuelle Aktion („Aligning frame: 42“) · Ergebnis („Alignment finished“) · Position und Pixelwerte · „Time used / Time left“ · Version des Builds |

**CPUs**: Standard ist die Zahl der logischen Kerne, Bereich 1 bis dahin. Die Zahl wirkt auf alle Berechnungen (eigener Thread-Pool im
Rechenkern) und steht nicht in der Projektdatei.

## Menüs

**Recent Recordings**: zuletzt geöffnete Aufnahmen; leer „No recent recordings“.

**Settings**

| Eintrag | Art | Standard / Auswahl |
|---|---|---|
| Preview Area | Auswahl | 2048 · 1024 · **512** · 256 · 128 (Pixel je Seite, angezeigt als „512 × 512 px“): Größe des Quadrats in der Bildmitte, das die Wavelet-Vorschau rechnet |
| Theme | Auswahl | **Dark** · Light (auf dem Mac folgt die App dem System, der Eintrag fehlt dort) |
| Show Advanced Controls | Häkchen | aus; öffnet den Abschnitt Advanced in jedem Schritt |
| Show CPU and Memory Usage | Häkchen | an; blendet „CPU“ und „RAM“ in der Statusleiste ein |

Gesperrt unter „Not available yet“: Automatic Processing · Autodetect Color/BW · Luminance Setting · Stretch FIT/TIFF · Play Sounds ·
Swap Red/Blue Channel · Wait When Loading a Single Image · Maximize During Messages · Save to Subdir · Bias Subtract. Ihre Werte
stehen in `settings.menu` und werden nirgends gelesen. Für die Helligkeit eines Farbpixels gelten im Rechenkern die Gewichte Rot **0,299**,
Grün **0,587**, Blau **0,114** (`planets_core::LUMINANCE`).

**Show ▾** (am Bild, je nach Schritt)

| Schritt | Einträge |
|---|---|
| Alignpoints, Align, Best Frames | Show Region · Show Alignpoints · Show Frame List · Show Quality Graph |
| Stack | Show Alignpoints (öffnet die Karte **Alignpoints Used for Stacking**) |
| Wavelets, Save | Show Alignpoints · Show Preview Area |

Gesperrt: Show Full Image · Show Align Data · Show Prefilter (alle Schritte), Show Stack Graph · Show Stack List (Stack). Show Region ist
nur mit gezeichneter Region wählbar.

**Karten über dem Bild**: **Frame List** (Check All · Check None · Clear List gesperrt · Go to Best Frame; Liste „Frame 1“ … mit Häkchen und
Qualität, nach Align nach Qualität sortiert), **Quality Graph** (Liniendiagramm Quality und Difference je Frame),
**Alignpoints Used for Stacking** („Alignpoint 1“ … mit Häkchen).

## Schritt Open

Der Schritt zeigt nach dem Laden die Datei mit Codec, Größe und Frame-Zahl, davor nichts. Aktion: **Select Recording…**; erledigt
**Select Another Recording…**.

## Schritt Alignpoints

Aktion **Set Alignpoints** (erledigt „Set Alignpoints Again“).

| Gruppe | Element | Art | Standard | Anmerkung |
|---|---|---|---|---|
| wichtig | Threshold | Regler | 0 | Enden „more points“ · „fewer points“; vor dem Setzen ohne Bereich (gesperrt), danach 0 bis zur Zahl aller Kandidaten, Beschriftung „Threshold · 82 points“ |
| | Min. distance between points (px) | Zahl | 30 | |
| | Min. distance from image edge (px) | Zahl | 20 | |
| | Draw Region | Umschalter | | zeichnet eine Region im Bild; nochmals gedrückt wird sie entfernt |
| Advanced: Point Placement | Brightness measure | Auswahl | Default | Default (Mittelpixel) · 3×3 area (Mittel) · Lowest pixel value (dunkelster Pixel der 3×3-Umgebung) |
| | Min. brightness (0–255) | Zahl | 30 | |
| | Max. brightness (0–255) | Zahl | 230 | |
| | Number of alignpoints | Zahl | 0 | nur Anzeige |
| | Keep points inside the scan frame | Häkchen | aus | gesperrt |

## Schritt Align

Aktion **Align** (erledigt „Align Again“).

| Gruppe | Element | Art | Standard | Anmerkung |
|---|---|---|---|---|
| wichtig | Box size per point (px) | Zahl | 30 | |
| | Max. point shift (px) | Zahl | 5 | |
| Advanced: Alignment Setup | Alignment mode: No Align · Scan Frames | Segmente | | gesperrt |
| | Show alignment | Häkchen | aus | gesperrt |
| | Align by center of gravity | Häkchen | aus | folgt dem Planeten über den Schwerpunkt seiner Helligkeit |
| | Brightness threshold | Zahl 0–255 | 20 | nur mit dem Häkchen aktiv; Pixel bis zu dieser Helligkeit zählen für den Schwerpunkt nicht |
| | Estimate rotation | Häkchen | aus | gesperrt |

## Schritt Best Frames

Aktion **Keep Best Frames** (erledigt „Keep Best Frames Again“; gesperrt „Run Align first“).

| Element | Art | Standard | Anmerkung |
|---|---|---|---|
| At least % of the best quality | Auswahl und Zahl | **gewählt**, 80 | behält jedes Frame, dessen Qualität mindestens diesen Anteil des besten erreicht |
| Best % of frames | Auswahl | | behält diesen Anteil aller Frames (das Prozentfeld der ersten Zeile gilt) |
| Best frames per point | Auswahl und Zahl | Zahl 200 | jeder Punkt nimmt seine eigenen schärfsten Frames |
| Best number of frames | Auswahl | | behält genau so viele Frames (das Zahlenfeld der dritten Zeile gilt) |

Darunter das **Qualitätsdiagramm** (ein Balken je Frame, bestes links; behaltene Frames in der Akzentfarbe, die Grenze gestrichelt) und
**Go to Best Frame** (bis Align gelaufen ist gesperrt).

## Schritt Stack

Aktion **Stack** (erledigt „Stack Again“).

| Gruppe | Element | Art | Standard | Anmerkung |
|---|---|---|---|---|
| wichtig | Output size | Auswahl | Default | Default · Smallest common area · Largest area · Region; nur Default wirkt, die übrigen sind gesperrt |
| | Use nearest points | Zahl | 3 | |
| | Correct geometry | Häkchen | **an** | |
| Advanced: Stacking Options | Even out frame brightness | Häkchen | aus | gleicht die Helligkeit der Frames vor dem Stacken an |
| | Stretch histogram after stacking | Häkchen | aus | gesperrt |
| | De-rotate images | Häkchen | aus | öffnet Planet (Jupiter · Saturn · Mars), Start time, Camera angle (°), Mirrored image (`docs/stacking.md`) |
| | Ignore edge pixels (px) | Zahl | 0 | |
| | Resampling · Drizzling | Häkchen | aus | gesperrt |
| Advanced: Create AVI | Codec | Auswahl | DIB | gesperrt, das Video ist immer unkomprimiert |
| | Compression quality | Zahl | 7500 | gesperrt |
| | Frame rate (fps) | Zahl | 25 | |
| | **Save Aligned Video…** | Schaltfläche | | speichert die behaltenen Frames, ausgerichtet |
| | Frame size | Auswahl | **Source size** | Source size · Smallest common area · Largest area · Region; nur Source size (Größe des Bezugsframes) wirkt |
| | Sort by quality | Häkchen | aus | beste Frames zuerst |
| | Use prefilters | Häkchen | aus | gesperrt |
| | **Save Averaged Video…** | Schaltfläche | | gleitender Mittelwert über mehrere ausgerichtete Frames |
| | Method | Auswahl | Moving average | Moving average · Compressed (gesperrt) |
| | Average of [n] frames | Zahl | 5 | |

## Schritt Wavelets

Aktion **Do All** (Anwenden auf das ganze Bild), links daneben **Update Preview**.

| Gruppe | Element | Art | Standard | Anmerkung |
|---|---|---|---|---|
| wichtig | Color | Häkchen | an | verarbeitet die Kanäle in Farbe; gesperrt bei Mono |
| | Sharpen brightness only | Häkchen | aus | schärft nur die Helligkeit und behält die Farben; gesperrt bei Mono |
| Wavelets (6 Ebenen), Kopf mit **Reset Wavelets** | Häkchen mit Ebenennummer | Häkchen | an | schaltet die Ebene ein und aus |
| | Denoise | Zahl, 2 Nachkommastellen | 0,00 | im Schema als ×100 gespeichert |
| | Sharpen | Zahl, 3 Nachkommastellen | 0,100 | im Schema als ×100 gespeichert |
| | Preview | Schaltfläche (Auge) | | gesperrt |
| | Regler der Ebene | Regler | 10 (= 1.0) | Bereich −50 … 1000, also −5.0 … 100.0; der Wert steht als Text rechts daneben (Regler-Position ÷ 10) |
| Advanced: Wavelet Settings | Run Do All automatically | Häkchen | aus | wendet Do All eine halbe Sekunde nach jeder Änderung an |
| | Keep layer values for a new stack | Häkchen | aus | |
| | Layer spacing | Segmente | **Linear** | Dyadic (2^n) · Linear |
| | First layer size | Zahl | 1 | |
| | Size step | Zahl | 0 | |
| | Wavelet filter | Segmente | **Gaussian** | Default · Gaussian |
| | Move all sliders together | Häkchen | aus | |
| Advanced: Saved Schemes | Saved Schemes | Auswahl | leer | gesperrt |
| | **Load Scheme…** · **Save Scheme…** | Schaltflächen | | Dateiformat `.rwv` in `docs/formats.md` |
| Advanced | Realign with Processed · Stack Again | Schaltflächen | | gesperrt |

Die Statusleiste zeigt beim Zeigen ins Bild die Position und die Kanalwerte.

**Adjustments (rechts)**, Schaltflächen im Raster aus drei Spalten: Histogram · Gamma · Color Mixing · View Zoomed · View Compare · View
Stack Size · Flip and Rotate · RGB Align · RGB Balance · Resize Image · Denoise/Deringing · Wavelet Filter · Masking · Show Line Graph ·
Crop. Aktiv sind Histogram, Gamma, Color Mixing, View Compare, Flip and Rotate, RGB Align, RGB Balance, Resize Image,
Denoise/Deringing, Masking und Crop; View Zoomed, View Stack Size, Wavelet Filter und Show Line Graph sind gesperrt. Jede aktive
Schaltfläche öffnet eine Karte über dem Bild (View Compare ist eine Ansicht Vorher/Nachher, keine Karte).

**Contrast/Brightness**, Kopf mit **Keep** (gesperrt) und **Reset**

| Element | Art | Standard | Bereich | Anmerkung |
|---|---|---|---|---|
| Contrast | Zahl und Regler | 100 (Regler 250) | Regler 1 … 1000 | Feld = Regler ÷ 2,5 (Entwurfsentscheidung); der Rechenkern arbeitet mit dem Feldwert |
| Brightness | Zahl und Regler | 0 | −512 … 512 | |

**Remember · Restore · Difference** merken das aktuelle Bild, bringen es zurück und zeigen die verstärkte Differenz (Faktor 4); „Show:“
**Current image** · **Remembered image**.

## Karten hinter den Adjustments

| Karte | Felder | Standard | Bereich |
|---|---|---|---|
| Histogram | Input · Result (Ansicht), **Black point**, **White point**, **Auto**, **Clip (%)** | 0, 255, Clip 0,1 | Black point 0 … White point −1, White point Black point +1 … 255, Clip 0 … 49 |
| Gamma | Regler und Zahl **Gamma** | 1,00 | 0,1 … 5 |
| Color Mixing | Preset (Neutral · Swap Red and Blue · Luminance Gray · Red to All Channels), neun Gewichte in %, **Preserve brightness** | Neutral (100 auf der Diagonalen), aus | −200 … 200 % |
| RGB Align | **X (px)** und **Y (px)** je Kanal, **Max shift**, **Auto** (verschiebt Rot und Blau auf Grün) | 0, Max shift 5 | Verschiebung ±20 px, Max shift 1 … 20 |
| RGB Balance | Rot, Grün, Blau in % | 100 | 0 … 200 |
| Denoise/Deringing | **Denoise (%)**, **Deringing (%)**, **Radius (px)** | 0, 0, 4 | Prozent 0 … 100, Radius 1 … 10 |
| Masking | **Use mask**, **Threshold**, **Feather (px)**, **Invert**, **Show mask** | aus, 20, 4, aus, aus | Threshold 0 … 255, Feather 0 … 50 |
| Flip and Rotate | Flip: Horizontal · Vertical; Rotate: Left 90° · Right 90° · 180°; **Angle (°)**; Interpolation (Bilinear · Bicubic) | keine Änderung | Angle −180 … 180 |
| Resize Image | Percent · Pixels; **Size (%)** oder **Width (px)** und **Height (px)**; Methode (Lanczos3 · Bicubic · Bilinear · Nearest Neighbor) | Percent, 100 % | Prozent 10 … 400, Seite 1 … 16384 px |
| Crop | Format Free · 1:1 · 3:2 · 4:3, **Cancel**, **Done** | Free | |

Die Karten wirken auf das Bild, das **Save Image** schreibt (`docs/formats.md`), Denoise/Deringing und Masking zuerst
(`docs/wavelets.md`).

## Schritt Save

Aktion **Save Image** mit dem Menü der Dateitypen („TIFF (16 bit) · .tif…“); der Text darüber nennt, was gespeichert wird („Saves the
sharpened stack with your color and tone settings.“). Die Gruppe **Crop** zeigt den Zuschnitt (**Crop…**, **Remove Crop**), die Gruppe
**Size** die Größe der Datei nach Drehen und Skalieren.

## Meldungen

Statustexte beginnen mit einem Großbuchstaben, haben keinen Punkt am Ende und bestehen aus einem kurzen Satz („Alignment finished“);
Fehler lauten „‹Schritt› failed: ‹Grund›“.
