# Ein- und Ausgabeformate von the reference program

Stand: RK-003, beobachtet an the reference program.1.0.0 (Dateidatum 02-04-2011) unter Windows 11.

## Eingabe

Dateitypen im Dialog „Open file(s)“ (Schaltfläche **Select**), in dieser Reihenfolge, Standard ist der erste:

| Filter im Dialog | Endungen | Stand im Klon |
|---|---|---|
| Video and Mpeg | `*.avi *.mpg *.mpeg` | AVI fertig: unkomprimiert (`DIB`), `MJPG`, `HFYU`, `FFVH`, `LAGS`; MPEG offen |
| Still frame(s) | `*.bmp` | fertig |
| Fit frame(s) | `*.fit` | fertig (auch `.fits`, `.fts`) |
| Still frame(s) | `*.jpg` | fertig |
| Tiff frame(s) | `*.tif` | fertig |
| Png frame(s) | `*.png` | fertig |
| LuCam recorder SER | `*.ser` | fertig |
| the reference program Framelist | _Endung unbekannt_ | offen, siehe `docs/offene-fragen.md` |

### Was der Klon liest (RK-004)

- **Einzelbilder** werden als Serie geöffnet: mehrere Dateien im Dialog wählen, jede Datei ist ein Frame, sortiert in
  natürlicher Reihenfolge (`frame2` vor `frame10`). Alle Frames brauchen die Größe und Kanalzahl des ersten.
  8 und 16 Bit, Mono und RGB; Alpha wird verworfen.
- **FITS**: nur die primäre HDU. `BITPIX` 8, 16, 32, −32, −64 mit `BZERO`/`BSCALE`, 2 Achsen (Mono) oder 3 Achsen mit
  3 Ebenen (RGB). Ganzzahlen werden auf den Bereich des Typs normiert. Bei Fließkomma gilt: Maximum ≤ 1 bleibt,
  Maximum ≤ 65535 wird durch 65535 geteilt, sonst durch das Maximum des Frames (_Annahme_, nicht mit dem Original
  verglichen). Nicht unterstützt: Erweiterungs-HDUs, komprimiertes FITS, `BITPIX` 64.
- **AVI-Codecs**, alle eigene Dekoder ohne Systemcodecs, YUV nach BT.601 (begrenzter Bereich) in RGB umgerechnet:
  - `MJPG`: Baseline-JPEG, Grau und Farbe, auch Frames ohne eigene Huffman-Tabellen. Interlaced MJPEG wird abgelehnt.
  - `HFYU`/`FFVH`: YUY2 (4:2:2), RGB24, RGB32, bei FFVH auch 4:2:0 und planare Varianten mit 8 Bit; Prädiktoren
    left, gradient, median. Alte HuffYUV-1.x-Dateien ohne Tabellen werden abgelehnt.
  - `LAGS` (Lagarith): RGB24, RGB32/RGBA, YUY2, YV12, einfarbige und unkomprimierte Frames. „Reduced resolution“
    wird abgelehnt.
- **Verworfene Frames**: Leere Chunks im AVI wiederholen das vorige Frame, wie es Capture-Programme vorsehen.
- Der Codec erscheint in der Dateiinfo (`DIB`, `MJPG` …), wie im Design der Reiterleiste vorgesehen.

- Der Dateiname „Open file(s)“ und die Bilderfilter deuten darauf hin, dass mehrere Einzelbilder auf einmal als
  Sequenz geöffnet werden können (_Vermutung_ fürs Original, nicht ausprobiert; der Klon macht es so).
- AVI liest the reference program über die Video-for-Windows-Schnittstelle. Die Titelleiste zeigt den Codec (`DIB vids` bei
  unkomprimiertem RGB). Welche Codecs gehen, hängt also von den auf dem Rechner installierten Codecs ab. Der Klon
  muss die gängigen selbst dekodieren.
- *Settings ▸ Stretch FIT/TIFF* (Always Ask · Yes · No) legt fest, ob FIT- und TIFF-Eingaben auf den vollen
  Wertebereich gestreckt werden.
- *Settings ▸ Swap Red/Blue Channel* tauscht beim Laden Rot und Blau.

## Ausgabe

**Stack-Reiter, Save image** (Summenbild):

| Filter im Dialog | Endung | Anmerkung |
|---|---|---|
| BMP (RGB) | `.bmp` | 8 Bit je Kanal, Standard |
| JPEG (RGB) | `.jpg` | 8 Bit |
| FIT (16-bit) | `.fit` | |
| FIT (32-bit) | `.fit` | |
| Special the reference program Fits | `.fit` | _Inhalt unbekannt_ |
| TIFF (16 bit RGB) | `.tif` | geprüft: unkomprimiert, 1 Strip, RGB interleaved, 16 Bit |
| PNG (16 bit RGB) | `.png` | |
| FIT (32 bit Float) | `.fit` | |

**Wavelet-Reiter, Save image** (bearbeitetes Bild), kleinere Auswahl:

| Filter im Dialog | Anmerkung |
|---|---|
| BMP (8BIT*RGB) | Standard |
| JPEG(8BIT*RGB) | |
| FIT (16BIT) RGB separate | _vermutlich_ je Kanal eine Datei |
| TIFF(16BIT *RGB) | geprüft wie oben |
| PNG (16BIT*RGB) | |

Beobachtungen:
- Als Dateiname schlägt the reference program den Namen der Aufnahme vor (`JUPITER.BMP`) und speichert im Ordner der Aufnahme.
  Wechselt man den Dateityp, passt the reference program die Endung selbst an.
- Der gewählte **Filter bestimmt das Format**, nicht die eingetippte Endung. Ein Versuch mit Filter „BMP“ und
  Name `….tif` ergab `….bmp`.
- 16-Bit-Ausgaben nutzen den vollen Bereich: Das 16-Bit-TIFF entspricht dem 8-Bit-BMP mal **256**
  (PSNR BMP gegen TIFF ÷ 256: 56,95 dB, gegen TIFF ÷ 257: 54,31 dB).
- Die Bildgröße bleibt beim Stacken mit Standardwerten gleich (1024×680), solange weder Resampling noch Drizzling an ist.
- *Settings ▸ Save to Subdir* und *Tools ▸ Record Processing Information in Filename* verändern _vermutlich_
  Zielordner und Dateinamen (nicht ausprobiert).

**Stack-Reiter, Create AVI**: Save Aligned (ausgerichtete Frames) und Save Integrated (gleitender Mittelwert über
n Frames) schreiben AVI mit einem Video-for-Windows-Codec, Qualität und Bildrate. Nicht ausprobiert.

### Was der Klon schreibt (RK-009)

*Save image* in beiden Reitern schreibt über `planets_io::export` (Befehl `save_image`, Dateidialog des Betriebssystems):

| Format | Endung | Inhalt |
|---|---|---|
| BMP (8 bit) | `.bmp` | RGB, bei Einkanalbildern Grau |
| JPEG (8 bit) | `.jpg` | Qualität 95 (_Annahme_, das Original nennt keinen Wert) |
| TIFF (16 bit) | `.tif` | RGB oder Grau, 16 Bit je Kanal, unkomprimiert |
| PNG (16 bit) | `.png` | RGB oder Grau, 16 Bit je Kanal |
| FITS (16 bit) | `.fit` | `BITPIX` 16, `BZERO` 32768 (Unsigned-Konvention), voller Wertebereich |
| FITS (32 bit) | `.fit` | `BITPIX` 32, `BZERO` 2147483648, voller Wertebereich |
| FITS (32 bit float) | `.fit` | `BITPIX` −32, Werte 0 … 1 unverändert |

- Der Stack-Reiter bietet alle sieben Einträge an, der Wavelet-Reiter BMP, JPEG, FITS 16, TIFF und PNG, jeweils in der Reihenfolge
  des Originals.
- **Das Format wählt die Oberfläche**, nicht die Endung (wie beim Filter des Originals). Der Dialog zeigt nur den Filter des
  gewählten Formats. Passt die getippte Endung nicht zum Format (auch `.tiff`, `.jpeg`, `.fits` zählen als passend), ersetzt der Klon
  sie, wie es das Original bei `….tif` mit Filter BMP tat.
- FITS: nur die primäre HDU. Einkanalbilder haben `NAXIS` 2, Farbbilder `NAXIS` 3 mit den Ebenen R, G, B **in einer Datei**; die
  Datei lässt sich mit dem Leser des Klons wieder öffnen. Die Zeilen werden von unten nach oben abgelegt, wie FITS es vorsieht.
  Die Variante „RGB separate“ (je Kanal eine Datei) und „Special the reference program Fits“ gibt es nicht, ihr Aufbau ist unbekannt.
- **Wertebereich**: Der Klon rechnet in Fließkomma und bildet 1.0 auf den Höchstwert des Formats ab (255, 65535 …). Das Original
  liefert 16 Bit als 8 Bit × 256, also höchstens 65280 (siehe Beobachtungen oben). Die Abweichung ist **entschieden** (Philipp,
  2026-11-11): 16 Bit bleiben im vollen Bereich bis 65535, siehe `docs/offene-fragen.md`, Punkt 22. Wer gegen ein Original-TIFF
  vergleicht, rechnet dessen Werte durch 256 (so macht es `docs/stacking.md`).
- **Farb- und Tonwerte**: *Save image* im **Wavelet-Reiter** schreibt das Bild so, wie das Fenster es zeigt: das geschärfte Bild mit
  den Werten von RGB Align, RGB Balance, Histogram, Gamma und Contrast/Brightness (RK-008). Die Oberfläche schickt die Werte mit
  (`adjustments` bei `save_image` und `save_image_as`), der Befehl rechnet `planets_core::color::adjust` auf dem
  Bild in voller Genauigkeit, nicht auf der 8-Bit-Anzeige. Sind alle Werte neutral, wird das Bild unverändert geschrieben, wie zuvor.
  Der **Stack-Reiter** zeigt das Summenbild ohne diese Werte und schreibt es auch so.
- Geschrieben wird in eine Datei mit Endung `.part` und danach umbenannt, ein Fehler lässt eine vorhandene Datei also unverändert.
- Nicht umgesetzt: *Save to Subdir*, *Record Processing Information in Filename* (im Original nie beobachtet).

**Create AVI**: Der Schreiber `planets_io::avi_write::AviWriter` erzeugt unkomprimierte AVI (24 Bit), Bild für Bild und damit auch für
tausende Frames. Die Codec-Auswahl und *Compress Qual* des Originals entfallen, weil der Klon keine Video-for-Windows-Codecs
benutzt. Grenze 4 GiB (kein OpenDML); ein Video, das darüber käme, bricht mit Meldung ab und lässt keine Datei zurück.

Umsetzung: Befehle `choose_video_path` (Dateidialog, Endung `.avi`), `save_aligned` und `save_integrated`
(`app/src-tauri/src/sequence_commands.rs`), Kern `planets_core::sequence` (`aligned_frames`, `MovingAverage`) und
`Stacker::warp`. Fortschritt über das Ereignis `sequence-progress`; Cancel und Pause teilen die Merker von *Stack*.

- **Save Aligned**: Jeder Frame, der nach *Limit* bleibt (die Häkchen der Frameliste), wird wie beim Stacken auf das Bezugsframe
  geschoben (*Correct geometry*, *Normalization* und *Use Nearestby Alignpoints* der Stacking options gelten) und als ein Videobild
  geschrieben. Was der Frame nicht abdeckt, bleibt schwarz. Reihenfolge: die der Frameliste; mit *Sort On Quality* der beste zuerst.
  Die Verschiebungen kommen aus dem letzten *Align* (`AlignState::results_for`); passen sie nicht mehr (Punkte seither an- oder
  abgehakt), registriert der Befehl neu. *Sort On Quality* braucht das Ergebnis von *Align* und meldet sonst einen Fehler.
- **Save Integrated** (_Vermutung_, siehe `docs/offene-fragen.md`, Punkt 23a): gleitender Mittelwert (*Method: Moving average*) über
  je *Average of n Frames* aufeinanderfolgende ausgerichtete Frames; jedes Videobild ist der Mittelwert eines Fensters, das um einen
  Frame weiterrückt. Bei N Frames entstehen N − n + 1 Bilder; sind es weniger als n, ein Bild mit dem Mittelwert aller.
- **Bildrate** ist *Framerate* (Standard 25). Nicht umgesetzt: Codec, *Compress Qual*, *Image size* (immer Originalgröße, d. h.
  Größe des Bezugsframes), *Use Prefilters*, die Methode *Compressed*.

## Wavelet-Schema (`.rwv`)

*Save Scheme* speichert die Wavelet-Einstellungen als INI-Datei mit der Endung **`.rwv`** (Filter „the reference program Wavelet
scheme settings“). Beispiel für Ebene 1 auf Reglerstellung 10.0, alles andere Standard:

```ini
[scheme]
itemindex=1      ; Waveletscheme: 0 = Dyadic, 1 = Linear
initial=1        ; Initial Layer
step=0           ; Step Increment
[method]
itemindex=1      ; Wavelet filter: 0 = Default, 1 = Gaussian
[checkbox]
layer1=1         ; … layer6, Ebene an (1) / aus (0)
[slider]
layer1=100       ; … layer6, Reglerposition (= Anzeige × 10)
[Gauss]
layer1=10        ; … layer6, Feld „Sharpen“ × 100
[Blur]
layer1=0         ; … layer6, Feld „Denoise“ × 100
```

Die Kommentare hat TARS ergänzt, die Datei selbst enthält keine. Zeilenenden CRLF. Die Zuordnung `itemindex` → Auswahl
ist aus dem Zustand beim Speichern geschlossen (Linear und Gaussian waren gewählt). Den Wert 0 haben wir nicht gegengeprüft.
Der Klon liest **und** schreibt dieses Format unverändert (RK-007, `crates/core/src/wavelet/scheme.rs`), ein eigenes Format gibt es
nicht. Zwischen Oberfläche und Rust gehen die Einstellungen als JSON, die Datei ist nur der Speicherort. Einzelheiten
(tolerantes Lesen, Rundung, Befehle `save_wavelet_scheme`/`load_wavelet_scheme`): `docs/wavelets.md`.

## Projektdatei (`.rkproj`)

Gibt es im Original nicht. Sie hält fest, was zum Wiederöffnen einer Sitzung nötig ist: die Aufnahme und die Einstellungen der Schritte.
Umsetzung: `planets_io::project` (Befehle `project_save`, `project_save_as`, `project_load`, `project_open`).

JSON, UTF-8, Endung **`.rkproj`**, Felder in camelCase:

```json
{
  "formatVersion": 1,
  "recording": ["C:/Aufnahmen/JUPITER.AVI"],
  "sections": {
    "align":   { },
    "stack":   { },
    "wavelet": { },
    "colour":  { }
  }
}
```

- `formatVersion`: Aufbau der Datei. Eine höhere Zahl als die des Programms wird abgelehnt (Meldung statt Raten).
- `recording`: die Dateien, die *Select* bekam: ein Video oder die Einzelbilder einer Serie in ihrer Reihenfolge. Pfade stehen so,
  wie sie beim Speichern waren (absolut); fehlt eine Datei beim Öffnen, meldet `project_load` sie in `missingFiles`.
- `sections`: je Schritt ein frei aufgebauter JSON-Wert. Wer einen Schritt baut, legt dessen Aufbau fest und liest und schreibt ihn mit
  `Project::section` und `Project::set_section`. Abschnitte, die ein Programm nicht kennt, bleiben beim Speichern erhalten. Namen:
  `align`, `stack`, `wavelet`, `colour` (Konstanten `SECTION_*`). Fehlt ein Abschnitt, gelten die Standardwerte.
- Inhalt der Abschnitte in der Oberfläche (`app/src/logic/projectfile.ts`): `align`, `stack` und `wavelet` enthalten alle
  Einstellungen des Reiters mit ihren Namen aus `settings.ts` (bei `align` ohne den Schwellenwert, der zu den Punkten des aktuellen
  Frames gehört), `colour` die beiden Häkchen `colour` und `lrgb`. Beim Öffnen werden nur Schlüssel übernommen, die es gibt und die
  denselben Typ haben; alles andere behält den Standardwert.
- Nicht in der Datei: die Bilder selbst (Ergebnis von Align und Stack). Sie werden beim Öffnen neu berechnet.

Das Wavelet-Schema des Originals (`.rwv`, Abschnitt oben) bleibt ein eigenes Dateiformat. *Load Scheme* und *Save Scheme* lesen und
schreiben es (Rechenkern `crates/core/src/wavelet/scheme.rs`, Befehle `load_wavelet_scheme` und `save_wavelet_scheme`, RK-007). Gelesen werden Schema, Filter,
Initial Layer, Step Increment und je Ebene Häkchen, Regler, Sharpen (÷ 100) und Denoise (÷ 100); gespeichert wird in der
Reihenfolge des Beispiels oben mit CRLF. Die Zuordnung der Werte 0 und 1 ist bisher nur aus dem Beispiel (Linear, Gaussian)
geschlossen. 

## Einstellungsdateien

the reference program legt seine Einstellungen als INI-Dateien **neben der EXE** ab (nicht im Benutzerprofil):

| Datei | Inhalt |
|---|---|
| `the reference program.ini` | Abschnitt `[Form]`: Fensterposition und die zuletzt benutzten Werte der Bedienelemente |
| `the reference program_Layout6.ini` | Andockpositionen, Größe und Sichtbarkeit der Panels, gewählter Skin |

Läuft the reference program ohne Administratorrechte, leitet Windows Schreibzugriffe in `Program Files (x86)` in den
VirtualStore des Benutzers um. Für den Klon heißt das: Einstellungen gehören ins Benutzerprofil (plattformgerecht).

### `the reference program.ini`

Ein Eintrag pro Bedienelement, Schema `<Elementname>_<Eigenschaft>=<Wert>`. Die Eigenschaft ist
`Value` (Zahlenfeld), `Checked` (Häkchen), `ItemIndex` (Auswahlgruppe/Liste), `Caption` oder `Tag`.
Werte nach der Installation (= Standardwerte):

| Schlüssel | Wert | Bedeutung (sichtbar in der Oberfläche) |
|---|---|---|
| `Dist_Value` | 30 | Align: „Minimum distance between“ (Alignpoints) |
| `Edgewidth_Value` | 0 | _unklar; die Oberfläche zeigt „Min distance from edge“ = 20_ |
| `Intensity_selection_ItemIndex` | 0 | Align: Intensity_selection = Default |
| `blocksize_Value` | 35 | _unklar; die Oberfläche zeigt „Alignmentbox size“ = 30_ |
| `MaxMove_Value` | 5 | Align: „Max Alignpoint movement“ |
| `LimitMethod_ItemIndex` | 0 | Align/Limit Setup: Lowest Quality (%) |
| `OptimizeQualitySpin_Value` | 80 | _unklar_; nach der Testsitzung stand 95 in der Datei, obwohl das Feld „Lowest Quality (%)“ 80 zeigte |
| `LimitNoOfFrames_Value` | 200 | Align/Limit Setup: Anzahl (bei „Frames/Apoint“, „Best Frames“) |
| `Processarea_ItemIndex` | 2 | Settings: Processing Area = 512 |
| `RecFITTIFSTRETCH_ItemIndex` | 0 | Settings: Stretch FIT/TIFF = Always Ask |
| `Cores_Value` | 1 | CPUs |
| `UseAdvControls_Checked` | False | Settings: Show Advanced Controls |
| `ProjectSounds_Checked` | False | Settings: Play Sounds |
| `CamWhiteBalance_Checked` | False | _vermutlich_ Settings: Swap Red/Blue Channel |
| `Checkupdates_Checked` | False | Settings: Check for the reference program Update |
| `AutoDetectColour_Checked` | False | Settings: Autodetect Colour/BW |
| `Colourstack_Checked` | False | Align: Colour (nach dem Test mit Farbvideo `True`) |
| `Autoprocessbox_Checked` | False | Settings: Automatic Processing |
| `AligntoStack1_Checked`, `StacktoWavelet1_Checked`, `AligntoWavelet1_Checked` | False | Settings ▸ Automatic Processing (Unterpunkte) |
| `ShowFramelist1_Checked`, `ShowGraphbox1_Checked`, `ShowAPwindow_Checked` | False | Anzeige-Häkchen |
| `HoldWavelets_Checked`, `HoldContrastBrightness_Checked` | False | Wavelet: Werte beim Neuladen behalten (_Vermutung_) |
| `LevelStep_Value` | 1 | Wavelet: Schrittweite der Ebenen (_Vermutung_) |
| `Multiplicant_Value` | 0 | Wavelet: Initial Layer (_Vermutung_) |
| `CopyConvolveMethod_ItemIndex` | 1 | Wavelet: Filtertyp (_Vermutung_) |
| `ResizeRatio_Value` | 100 | Größenänderung in % |
| `MaxMultiScalePoints_Value`, `MultiScaleLevels_Value`, `MultiScaleSearcharea_Value` | 20, 2, 40 | Erweiterte Ausrichtung (Advanced Controls) |
| `sensitivity_Tag` | 50 | _vermutlich_ Align: Threshold-Regler; nach der Testsitzung stand 10 in der Datei (Regler war auf 9) |
| `MatrixEdgesetting_Value`, `BlockScan1_Value` | 20, 30 | _unklar_ |
| `Videomode_ItemIndex`, `AviSizeMode_ItemIndex` | 0, 2 | AVI-Erzeugung (Create AVI) |
| `NewIntensBalance_Checked` | False | _unklar_ |
| `VersionNo_Caption` | URL | Adresse der Update-Prüfung (Klon braucht das nicht) |

### `the reference program_Layout6.ini`

Abschnitte `[LastLayout @ <Panelname>]` mit `Visible`, `DockedTo`, Größen für angedockt und schwebend.
Die Panelnamen verraten den Funktionsumfang: `Alignmentpoints_Panel`, `Registergraph_Panel`, `Framelist_Panel`,
`Stackgraphs_Panel`, `StackList_Panel`, `SigmaEvent_Panel`, `CreateAVI_Panel`, `ImageInfo_Panel`,
`Gamma_Panel`, `HistoRGB_Panel`, `WaveletFilter_Panel`, `WaveletSection_panel`, `RGBAlign_Panel`, `Zoom_Panel`,
`Linegraphpanel`, `Histogram_Panel`, `Filter_Panel`, `Layer1Filterpanel`, `Compare_Panel`, `LRGBMix_panel1`,
`Create_Mask_Panel`, `FlipandRotate`, `MultiAlign_Panel`, `AdvAlign_panel`, `ChangelistPanel`. Skin: `Office 2007 Silver`.

Den Klon betrifft das Layout nicht (eigenes, modernes Design, RK-010).

## Testdaten

Siehe `docs/testdaten.md`.
