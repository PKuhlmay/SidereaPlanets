# Ein- und Ausgabeformate

Welche Dateien Siderea Planets liest und schreibt, mit den Grenzen jedes Formats. Die Werkzeuge dahinter sind die Crates
`planets-io` (Leser und Schreiber) und `planets-core` (Wavelet-Schema).

## Eingabe

Dateitypen im Dialog „Open file(s)“ (**Select Recording…**), in dieser Reihenfolge, Standard ist der erste:

| Filter im Dialog | Endungen | Unterstützung |
|---|---|---|
| Video (`*.avi *.ser`) | `.avi`, `.ser` | AVI: unkomprimiert (`DIB`), `MJPG`, `HFYU`, `FFVH`, `LAGS`; SER; MPEG nicht |
| Still frames | `.bmp`, `.jpg`, `.jpeg`, `.tif`, `.tiff`, `.png` | ja |
| FITS frames | `.fit`, `.fits`, `.fts` | ja, Einzelheiten unten |
| All files | alle | |

Aufnahmen lassen sich auch auf das Fenster ziehen.

### Was gelesen wird

- **Einzelbilder** werden als Serie geöffnet: mehrere Dateien im Dialog wählen, jede Datei ist ein Frame, sortiert in
  natürlicher Reihenfolge (`frame2` vor `frame10`). Alle Frames brauchen die Größe und Kanalzahl des ersten.
  8 und 16 Bit, Mono und RGB; Alpha wird verworfen.
- **FITS**: nur die primäre HDU. `BITPIX` 8, 16, 32, −32, −64 mit `BZERO`/`BSCALE`, 2 Achsen (Mono) oder 3 Achsen mit
  3 Ebenen (RGB). Ganzzahlen werden auf den Bereich des Typs normiert. Bei Fließkomma gilt (Entwurfsentscheidung): Maximum ≤ 1
  bleibt, Maximum ≤ 65535 wird durch 65535 geteilt, sonst durch das Maximum des Frames. Nicht unterstützt: Erweiterungs-HDUs,
  komprimiertes FITS, `BITPIX` 64.
- **SER**: Version 3 des SER-Formats; Mono, RGB und BGR werden gelesen, Bayer-Aufnahmen kommen als rohe Mono-Ebene.
- **AVI-Codecs**, alle eigene Dekoder ohne Systemcodecs (die Ergebnisse hängen nicht von installierten Codecs ab), YUV nach BT.601
  (begrenzter Bereich) in RGB umgerechnet:
  - `DIB`: unkomprimiertes RGB.
  - `MJPG`: Baseline-JPEG, Grau und Farbe, auch Frames ohne eigene Huffman-Tabellen. Interlaced MJPEG wird abgelehnt.
  - `HFYU`/`FFVH`: YUY2 (4:2:2), RGB24, RGB32, bei FFVH auch 4:2:0 und planare Varianten mit 8 Bit; Prädiktoren
    left, gradient, median. Alte HuffYUV-1.x-Dateien ohne Tabellen werden abgelehnt.
  - `LAGS` (Lagarith): RGB24, RGB32/RGBA, YUY2, YV12, einfarbige und unkomprimierte Frames. „Reduced resolution“
    wird abgelehnt.
- **Verworfene Frames**: Leere Chunks im AVI wiederholen das vorige Frame, wie es Capture-Programme vorsehen.
- Der Codec erscheint in den Aufnahmedetails (`DIB`, `MJPG` …) und rechts in der Schrittleiste.
- Ob eine Aufnahme farbig ist, entscheidet ihr Inhalt; das Häkchen **Color** im Schritt Wavelets wird danach gesetzt.

## Ausgabe

*Save Image* (Schritt Save, in der App mit dem Menü der Dateitypen) schreibt über `planets_io::export` (Befehl `save_image`,
Dateidialog des Betriebssystems):

| Format | Endung | Inhalt |
|---|---|---|
| BMP (8 bit) | `.bmp` | RGB, bei Einkanalbildern Grau |
| JPEG (8 bit) | `.jpg` | Qualität 95 (Entwurfsentscheidung: visuell verlustfrei) |
| TIFF (16 bit) | `.tif` | RGB oder Grau, 16 Bit je Kanal, unkomprimiert, 1 Strip |
| PNG (16 bit) | `.png` | RGB oder Grau, 16 Bit je Kanal |
| FITS (16 bit) | `.fit` | `BITPIX` 16, `BZERO` 32768 (Unsigned-Konvention), voller Wertebereich |
| FITS (32 bit) | `.fit` | `BITPIX` 32, `BZERO` 2147483648, voller Wertebereich |
| FITS (32 bit float) | `.fit` | `BITPIX` −32, Werte 0 … 1 unverändert |

- Der Schritt Save bietet BMP, JPEG, FITS 16, TIFF und PNG an (`ExportFormat::WAVELET`). Für das Summenbild als Quelle (`ImageSource::Stack`)
  kennt der Befehl alle sieben Einträge (`ExportFormat::STACK`: BMP, JPEG, FITS 16, FITS 32, TIFF, PNG, FITS 32 float).
- **Das Format wählt die Oberfläche**, nicht die Endung. Der Dialog zeigt nur den Filter des gewählten Formats. Passt die getippte
  Endung nicht zum Format (auch `.tiff`, `.jpeg`, `.fits` zählen als passend), ersetzt die App sie.
- Als Dateiname schlägt die App den Namen der Aufnahme vor und speichert im Ordner der Aufnahme.
- FITS: nur die primäre HDU. Einkanalbilder haben `NAXIS` 2, Farbbilder `NAXIS` 3 mit den Ebenen R, G, B **in einer Datei**; die
  Datei lässt sich mit dem Leser der App wieder öffnen. Die Zeilen werden von unten nach oben abgelegt, wie FITS es vorsieht.
- **Wertebereich**: Gerechnet wird in Fließkomma; 1.0 wird auf den Höchstwert des Formats abgebildet (255, 65535 …). 16 Bit nutzen
  also den vollen Bereich bis 65535.
- **Farb- und Tonwerte**: *Save Image* schreibt das Bild so, wie das Fenster es im Schritt Wavelets zeigt: das geschärfte Bild mit
  den Werten von RGB Align, RGB Balance, Histogram, Gamma und Contrast/Brightness. Die Oberfläche schickt die Werte mit
  (`adjustments` bei `save_image` und `save_image_as`), der Befehl rechnet `planets_core::color::adjust` auf dem
  Bild in voller Genauigkeit, nicht auf der 8-Bit-Anzeige. Sind alle Werte neutral, wird das Bild unverändert geschrieben.
  Das Summenbild als Quelle wird ohne diese Werte geschrieben.
- Die Bildgröße bleibt beim Stacken mit Standardwerten gleich, solange weder Resampling noch Drizzling an ist.
- Geschrieben wird in eine Datei mit Endung `.part` und danach umbenannt, ein Fehler lässt eine vorhandene Datei also unverändert.
- Nicht umgesetzt: eine FITS-Ausgabe mit je einer Datei je Kanal, *Save to Subdir* und *Record Processing Information in Filename*.

### Create AVI

Der Schreiber `planets_io::avi_write::AviWriter` erzeugt unkomprimierte AVI (24 Bit), Bild für Bild und damit auch für
tausende Frames. Es gibt keine Codec-Auswahl und keine Kompressionsqualität, weil die App keine Systemcodecs benutzt. Grenze
4 GiB (kein OpenDML); ein Video, das darüber käme, bricht mit Meldung ab und lässt keine Datei zurück.

Umsetzung: Befehle `choose_video_path` (Dateidialog, Endung `.avi`), `save_aligned` und `save_integrated`
(`app/src-tauri/src/sequence_commands.rs`, Rechnung in `crates/session/src/sequence.rs`), Kern `planets_core::sequence` (`aligned_frames`, `MovingAverage`) und
`Stacker::warp`. Fortschritt über das Ereignis `sequence-progress`; Cancel und Pause teilen die Merker von *Stack*.

- **Save Aligned Video…**: Jeder Frame, der nach *Keep Best Frames* bleibt (die Häkchen der Frame-Liste), wird wie beim Stacken auf das
  Bezugsframe geschoben (*Correct geometry*, *Even out frame brightness* und *Use nearest points* der Stacking Options gelten) und als ein
  Videobild geschrieben. Was der Frame nicht abdeckt, bleibt schwarz. Reihenfolge: die der Frame-Liste; mit *Sort by quality* der beste
  zuerst. Die Verschiebungen kommen aus dem letzten *Align* (`AlignState::results_for`); passen sie nicht mehr (Punkte seither an- oder
  abgehakt), registriert der Befehl neu. *Sort by quality* braucht das Ergebnis von *Align* und meldet sonst einen Fehler.
- **Save Averaged Video…** (*Method: Moving average*): gleitender Mittelwert über je *Average of n frames* aufeinanderfolgende
  ausgerichtete Frames; jedes Videobild ist der Mittelwert eines Fensters, das um einen Frame weiterrückt. Bei N Frames entstehen
  N − n + 1 Bilder; sind es weniger als n, ein Bild mit dem Mittelwert aller (Entwurfsentscheidung).
- **Bildrate** ist *Frame rate (fps)* (Standard 25). Die Bildgröße ist immer die des Bezugsframes. Nicht umgesetzt: Codec,
  Kompressionsqualität, andere Bildgrößen, Vorfilter, die Methode *Compressed*.

## Wavelet-Schema (`.rwv`)

*Save Scheme…* speichert die Wavelet-Einstellungen als INI-Datei mit der Endung **`.rwv`**. Beispiel für Ebene 1 auf Reglerstellung
10.0, alles andere Standard:

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

Die Kommentare dienen hier nur der Erklärung, die Datei enthält keine. Zeilenenden CRLF. Die Zuordnung der Werte 0 und 1 von `itemindex`
folgt dem Beispiel (Linear und Gaussian gleich 1).
Die App liest **und** schreibt dieses Format (`crates/core/src/wavelet/scheme.rs`). Gelesen werden Schema, Filter,
Initial Layer, Step Increment und je Ebene Häkchen, Regler, Sharpen (÷ 100) und Denoise (÷ 100); gespeichert wird in der Reihenfolge des
Beispiels mit CRLF. Zwischen Oberfläche und Rust gehen die Einstellungen als JSON, die Datei ist nur der Speicherort. Einzelheiten
(tolerantes Lesen, Rundung, Befehle `save_wavelet_scheme`/`load_wavelet_scheme`): `docs/wavelets.md`.

## Projektdatei (`.sidereaplanets`)

Sie hält fest, was zum Wiederöffnen einer Sitzung nötig ist: die Aufnahme und die Einstellungen der Schritte.
Umsetzung: `planets_io::project` (Befehle `project_save`, `project_save_as`, `project_load`, `project_open`).

JSON, UTF-8, Endung **`.sidereaplanets`**, Felder in camelCase:

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
- `recording`: die Dateien, die *Select Recording…* bekam: ein Video oder die Einzelbilder einer Serie in ihrer Reihenfolge. Pfade stehen so,
  wie sie beim Speichern waren (absolut); fehlt eine Datei beim Öffnen, meldet `project_load` sie in `missingFiles`.
- `sections`: je Schritt ein frei aufgebauter JSON-Wert. Wer einen Schritt baut, legt dessen Aufbau fest und liest und schreibt ihn mit
  `Project::section` und `Project::set_section`. Abschnitte, die ein Programm nicht kennt, bleiben beim Speichern erhalten. Namen:
  `align`, `stack`, `wavelet`, `colour` (Konstanten `SECTION_*`). Fehlt ein Abschnitt, gelten die Standardwerte.
- Inhalt der Abschnitte in der Oberfläche (`app/src/logic/projectfile.ts`): `align`, `stack` und `wavelet` enthalten alle
  Einstellungen des Schritts mit ihren Namen aus `settings.ts` (bei `align` ohne den Schwellenwert, der zu den Punkten des aktuellen
  Frames gehört), `colour` die beiden Häkchen `colour` und `lrgb`. Beim Öffnen werden nur Schlüssel übernommen, die es gibt und die
  denselben Typ haben; alles andere behält den Standardwert.
- Nicht in der Datei: die Bilder selbst (Ergebnis von Align und Stack). Sie werden beim Öffnen neu berechnet.
