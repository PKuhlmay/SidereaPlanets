# Ein- und Ausgabeformate von the reference program

Stand: RK-003, beobachtet an the reference program.1.0.0 (Dateidatum 02-04-2011) unter Windows 11.

## Eingabe

Dateitypen im Dialog „Open file(s)“ (Schaltfläche **Select**), in dieser Reihenfolge, Standard ist der erste:

| Filter im Dialog | Endungen | Stand im Klon |
|---|---|---|
| Video and Mpeg | `*.avi *.mpg *.mpeg` | AVI unkomprimiert fertig (RK-004); komprimierte Codecs und MPEG offen |
| Still frame(s) | `*.bmp` | offen (RK-004) |
| Fit frame(s) | `*.fit` | offen (RK-004) |
| Still frame(s) | `*.jpg` | offen (RK-004) |
| Tiff frame(s) | `*.tif` | offen (RK-004) |
| Png frame(s) | `*.png` | offen (RK-004) |
| LuCam recorder SER | `*.ser` | fertig (RK-004) |
| the reference program Framelist | _Endung unbekannt_ | offen, siehe `docs/offene-fragen.md` |

- Der Dateiname „Open file(s)“ und die Bilderfilter deuten darauf hin, dass mehrere Einzelbilder auf einmal als
  Sequenz geöffnet werden können (_Vermutung_, nicht ausprobiert).
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
Ziel für den Klon: dieses Format lesen können, damit vorhandene Schemata weiter nutzbar sind (RK-007).

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
