# Wavelet-Schärfung

Was der Rechenkern (`crates/core/src/wavelet/`) tut und warum. Bedienelemente und Standardwerte stehen in `docs/ui.md`, der Ablauf in
`docs/workflow.md` (Abschnitt 4), das Schemaformat in `docs/formats.md`. Was gegen gespeicherte eigene Referenzergebnisse geprüft ist
und was nicht, steht im letzten Abschnitt „Regressionsprüfung und PSNR-Schwelle“; alles andere ist Entwurfsentscheidung.

## Modell

Das Bild wird in sechs Detailebenen und einen Rest zerlegt. Ebene `n` ist die Differenz zwischen dem
`n−1`-mal und dem `n`-mal geglätteten Bild, enthält also die Strukturen einer Größenordnung (Ebene 1 die
feinsten). Jede Ebene bekommt Verstärkung, optional Entrauschen, dann wird alles wieder addiert:

```
G0 = Bild,  G(n) = Glätten(G(n−1), Abstand_n)
Ebene_n    = G(n−1) − G(n)
Ergebnis   = Bild + Σ_n ( Gain_n · Denoise(Ebene_n) − Ebene_n )      // Rest = G6, bleibt unverändert
```

Bei neutralen Einstellungen (alle Regler 1.0, Denoise 0) liefern die Ebenen **bitgleich** das Eingabebild (`wavelet::process`).
Danach folgt die Tonkennlinie (nächster Abschnitt); erst sie macht *Do All* und die Vorschau zum fertigen Ergebnis.
Jeder Farbkanal wird für sich gerechnet, solange *Colour* an und *LRGB* aus ist (Abschnitt Colour und LRGB). Ebenen, die nichts ändern, kosten nichts; hinter der letzten
veränderten Ebene wird nicht weiter zerlegt.

### Glätten

Getrennter 3- bzw. 5-Tap-Kern (erst waagerecht, dann senkrecht), Taps `Abstand_n` Pixel auseinander
(à trous), Bildränder durch Wiederholen der Randpixel.

| Wavelet filter | Kern (eindimensional) | Hinweis |
|---|---|---|
| Gaussian (Standard) | `0.05 · 0.90 · 0.05` | fest, Sharpen ändert die Kernbreite nicht; durch den Regressionstest für Ebene 1 gedeckt |
| Default | B3-Spline `1 4 6 4 1 / 16` | Entwurfsentscheidung (übliche Wahl bei à-trous-Wavelets), nicht gegen eine Referenz geprüft |

| Waveletscheme | Abstand der Taps in Ebene `n` (n ab 0) | Hinweis |
|---|---|---|
| Linear (Standard) | `Initial + n · (1 + Step)`, mit Standardwerten 1, 2, 3, 4, 5, 6 | Ebene 1 durch den Regressionstest gedeckt, die übrigen Ebenen nicht |
| Dyadic (2^n) | `Initial · 2^n`: 1, 2, 4, 8, 16, 32 | Entwurfsentscheidung, nicht gegen eine Referenz geprüft |

Wegen des Taps-Abstands hängt ein Pixel des Ergebnisses von Pixeln bis zu `Σ Abstand · Reichweite` entfernt ab
(Standard 21 px, Dyadic mit Gaussian 63 px). Diesen Rand (`Structure::margin`) rechnet die Vorschau mit.

### Tonkennlinie

Das Bild bekommt auch bei neutralen Reglern eine leichte Tonkennlinie. Sie wird am Ende von *Do All* und der Vorschau angewendet
(`wavelet/output.rs`, `output_curve`), nach den Ebenen und vor dem Zurückholen der Farbe bei LRGB:

```
y = 0,988231 · x + 0,015595 · x³          x, y auf 0…1 bezogen, x vorher auf 0…1 begrenzt, y höchstens 1
```

Eigenschaften der Kennlinie, wie der Code sie anwendet:

| Punkt | Festlegung |
|---|---|
| Ein Eingabewert gibt immer denselben Ausgabewert, für alle Kanäle gleich; es gibt keinen räumlichen Anteil | Entwurf: reine Punktabbildung |
| Form: ungerades Polynom dritten Grades ohne quadratischen Anteil. Eine freie Anpassung `a·x + b·x² + c·x³` an die Stützwerte ergibt `b ≈ −0,0005`, also null; ein Polynom zweiten Grades verfehlte bis zu 18 von 65535, dieses höchstens 3 | Die Koeffizienten sind angepasste Parameter (Test `the_curve_follows_the_reference_samples`, Toleranz 3 von 65535) |
| Wirkung: dunkelt die Mitten um etwa ein Prozent ab; Abstand zur Identität bis zu einer 8-Bit-Stufe (256 von 65535) um die Mitte des Wertebereichs der Stützwerte, in den Tiefen etwa 1,2 % | Entwurf |
| Bereich: die Stützwerte reichen nur bis 40063 (0,61). Darüber ist die Kurve eine Fortsetzung. Sie steigt weiter, erreicht bei 1,0 den Wert 1,0038 und wird dort auf 1 begrenzt | Entwurfsentscheidung |
| Auflösung: Der Code rechnet in Fließkomma und rundet erst beim Speichern; die Kennlinie wirkt also in 16-Bit-Auflösung, nicht auf einer 8-Bit-Treppe | Entwurf |

*Before* zeigt das Summenbild ohne die Kennlinie. Bei Graustufen wird sie auf alle Kanalzahlen gleich angewendet.

### Colour und LRGB

Die Häkchen *Colour* und *LRGB* (Align- und Stack-Reiter, ein Wert für alle Reiter, `docs/ui.md`) wählen `wavelet::ColourMode`:

| Colour | LRGB | `ColourMode` | Rechnung |
|---|---|---|---|
| an | aus | `Colour` | jeder Kanal für sich |
| an | an | `Lrgb` | Ebenen auf der Luminanz (`Image::to_luma`), danach Kennlinie, dann `color::with_luminance`: Farbton und Sättigung des Summenbilds, Luminanz = Ergebnis |
| aus | | `Gray` | Luminanz des Summenbilds, Ebenen, Kennlinie; das Ergebnis hat einen Kanal |

LRGB zählt ohne Colour nicht. Bei Bildern mit weniger als drei Kanälen (Graustufenaufnahmen) ändert sich nichts. Die Zuordnung folgt den
Namen der Häkchen (Entwurfsentscheidung).

### Einstellungen je Ebene

| Feld | Wirkung | Hinweis |
|---|---|---|
| Regler (Gain, −5.0 … 100.0, Reglerposition ÷ 10) | Faktor der Ebene, 1.0 = unverändert, 0 = Ebene entfernt, negativ kehrt sie um | Ebene 1 mit 10.0 durch den Regressionstest gedeckt |
| Sharpen (Standard 0.100) | skaliert die Abweichung des Reglers von 1.0: `1 + (Gain − 1) · Sharpen / 0.1`. Bei 0.100 wirkt der Regler genau wie angezeigt | Entwurfsentscheidung; nur der Wert 0.100 ist durch den Regressionstest gedeckt |
| Denoise (Standard 0.00) | Soft-Threshold der Ebene vor der Verstärkung: Schwelle = `Denoise · σ`, σ = 1,4826 · Median der Beträge der Ebene (je Kanal). 0 = aus | Entwurfsentscheidung |
| Häkchen | aus = Ebene bleibt unverändert (Gain 1, kein Denoise) | Entwurfsentscheidung |

Grenzen: Gain wird auf −5 … 100 begrenzt, Denoise und Sharpen auf 0 … 100,
Initial Layer auf 1 … 64, Step Increment auf 0 … 64 (`WaveletSettings::clamped`); beim Tippen werden also beliebige Werte angenommen und
auf diese Grenzen gekürzt.

*Automatic*, *Hold Wavelet Setting*, *Use Linked Wavelets* und die Schaltfläche *Preview* je Ebene betreffen nur
die Bedienung und liegen nicht im Rechenkern.

## Vorschau und Do All

Maßnahmen aus `docs/gui-toolkit.md` (Spike):

- **Ausschnitt statt Gesamtbild**: `Previewer::render` verarbeitet nur den Bereich (*Processing Area*, Standard
  512 px, `Region::centered`) plus den Rand, den die Ebenen brauchen. Das Ergebnis ist pixelgleich zum
  entsprechenden Ausschnitt von `process`. Einziger Unterschied: das Rauschmaß für *Denoise* wird aus dem
  verarbeiteten Bereich geschätzt, nicht aus dem ganzen Bild.
- **Ebenen bleiben erhalten**: Die Zerlegung (sechs Ebenen des Bereichs) wird nur neu gebaut, wenn sich Waveletscheme,
  Filter, Initial Layer oder Step Increment ändern oder der Bereich den gespeicherten verlässt. Ein Regler, Denoise
  oder Sharpen ändern nur die Summe; Ebenen, die nichts ändern, entfallen dabei ganz.
- **RGBA mit `rayon`**: `rgba_region` wandelt zeilenweise parallel um und schneidet dabei den Bereich aus.
- **Übertragung**: weiter per `invoke` mit Binärantwort. Eigenes URI-Schema oder `Channel` sind **nicht umgesetzt und
  nicht gemessen**.
- **Messung**: `crates/core/tests/wavelet_preview_timing.rs` (ignoriert, nur im Release-Build sinnvoll) misst einen
  Reglerschritt im 512-px-Bereich bei 1024×680 und 2048×2048 gegen 50 ms (20 Bilder/s). **Gemessen am 2026-11-11** (Release-Build, Tonkennlinie eingerechnet):
  1024×680: erste Vorschau 31,1 ms, Reglerschritt 4,8 ms (Median aus 15); 2048×2048: 27,5 ms und 3,7 ms. Die Übertragung steckt darin nicht, sie wird in Phase 2 mit der Oberfläche gemessen (Spike:
  rund 35 ms für 1024×680-RGBA und 20 Bilder/s, 5,3 Bilder/s bei 2048²).

**Do All** (`wavelet::do_all`) rechnet das ganze Bild im Strom (drei Puffer, nicht sieben), dann Kennlinie und Farbbehandlung
(`output::finish`), und liefert ein auf 0…1 begrenztes Bild. Die Vorschau ruft dieselbe `finish`-Funktion auf ihrem Ausschnitt
auf, die Pixel sind gleich (Test `preview_equals_do_all_in_every_colour_mode`). Außerhalb der Processing Area zeigt das Bild
im Fenster weiter das Summenbild ohne Kennlinie, bis *Do All* gelaufen ist; am Rand der Area ist deshalb ein leichter
Helligkeitssprung zu sehen.

## Anbindung im Wavelet-Reiter (`app/src/wavelet.ts`, `app/src/logic/wavelet.ts`)

- Jede Änderung der Werte (Regler, Denoise, Sharpen, Häkchen, Scheme, Filter, Initial, Step), der Processing Area oder des
  Summenbilds fordert `wavelet_preview` für die Processing Area an (`processingArea` aus `logic/viewport.ts`, gleiche
  Rundung wie `Region::centered`). Es läuft immer nur eine Anfrage, weitere Änderungen werden zu einer zusammengefasst.
  Gerechnet wird nur im Schritt Wavelets (`previewWanted`), nicht im Schritt Save, und nicht, solange ein Do All für diese
  Werte und dieses Summenbild vorliegt: Sonst würde das Betreten von Save oder Wavelets das Ergebnis von Do All durch
  Summenbild plus Vorschau-Ausschnitt ersetzen und „Do All finished“ in der Statusleiste überschreiben
  (ein Prüflauf der App: Wechsel nach Save zeigte „Preview: 79 ms“).
- Die Antwort wird in einen Puffer geschrieben, der eine Kopie des Summenbilds ist; `session.processedImage` teilt sich
  diesen Puffer, eine Vorschau kostet also nur den Bereich. Außerhalb der Processing Area bleibt das Summenbild.
- Die Statusleiste zeigt die Zeit von Anfrage bis Zeichnen (`Preview: n ms`). Das ist die Messung der Übertragung, die
  `docs/gui-toolkit.md` verlangt; bisher gibt es dafür **nur einen einzelnen Wert** (79 ms beim Wechsel nach Save), keine Messreihe.
- **Process** und **Do All** laufen über `runJob`. Do All ersetzt `session.processedImage` durch das ganze Bild.
- **Load Scheme / Save Scheme**: Dateidialog (Plugin `dialog`, Filter `.rwv`) in `pickSchemePath`.
- **Realign with Processed** ist nicht angebunden (gesperrt, „Not available yet“).
- Ohne gestapeltes Bild im Backend (`WaveletState::set_source` wurde nicht aufgerufen) meldet die Statusleiste
  „no image to sharpen“. Der Reiter ist ohne Stack ohnehin gesperrt.
- Automatic, Hold und Linked sind Entwurfsentscheidungen der Bedienung.

## Tauri-Befehle (`app/src-tauri/src/wavelet_commands.rs`)

Die Befehle sind dünne Hüllen über `planets_session::Session` (`crates/session/src/wavelet.rs`, dort Zustand und Rechnung).

Einstellungen gehen als JSON mit den Feldern von `WaveletSettings` (camelCase): `scheme` (`"dyadic"`/`"linear"`),
`filter` (`"default"`/`"gaussian"`), `initial`, `step`, `layers[6]` mit `enabled`, `gain`, `denoise`, `sharpen`.

| Befehl | Zweck |
|---|---|
| `wavelet_default_settings()` | Werte für *Reset Wavelets* |
| `wavelet_set_source_frame(index)` | Ersatz: Frame der Aufnahme als Bild des Reiters; im Ablauf setzt *Stack* die Quelle (`WaveletState::set_source`) |
| `wavelet_preview(settings \| null, area, mode)` | Vorschau mit Kennlinie und Farbbehandlung wie *Do All*; `mode` ist `"colour"`, `"lrgb"` oder `"gray"`. Antwort binär: x, y, Breite, Höhe (4 × u32 LE, der auf das Bild zugeschnittene Bereich), dann RGBA. `null` = unbearbeitet (*Before*) |
| `wavelet_do_all(settings, mode)` | ganzes Bild mit Kennlinie und Farbbehandlung; Antwort binär wie `read_frame` (Breite, Höhe als u32 LE, RGBA). Das Ergebnis bleibt in `WaveletState::processed()` für *Save image* |
| `save_wavelet_scheme(path, settings)` / `load_wavelet_scheme(path)` | `.rwv` schreiben und lesen |

## Denoise/Deringing und Masking (eigener Entwurf)

Die Karten **Denoise/Deringing** und **Masking** des Adjustments-Panels sind ein eigener Entwurf, einfach gehalten und im Rechenkern
(`crates/core/src/refine.rs`) umgesetzt. Sie arbeiten auf zwei Bildern: dem **geschärften Bild** *S* (Ergebnis von Do All, in der Vorschau das
Summenbild mit der geschärften Processing Area) und dem **ungeschärften Summenbild** *R*, aus dem es entstanden ist (`ExportState::stack`).
Alle Schritte laufen vor RGB Align, Balance, Histogram, Gamma und Contrast/Brightness, sowohl für die Anzeige (`Session::apply_colors`, Histogramm)
als auch beim Speichern (`save` in `crates/session/src/export.rs`, nur für das Wavelet-Bild). Reihenfolge: **Deringing, Denoise, Maske**.
Stärke 0 (und Maske aus) gibt *S* **bitgleich** zurück; fehlt *R* oder hat es eine andere Größe, laufen Deringing und Maske nicht (ein Graustufen-*R* wird
für ein RGB-*S* auf drei Kanäle gebracht).

### Deringing

Der dunkle Ring am Planetenrand entsteht, weil das Schärfen neben einer starken Kante unter den Himmelswert fällt. Das Verfahren hebt
genau diese Unterschreitung an, und nur dort, wo eine starke Kante in der Nähe liegt:

1. Je Kanal und Pixel aus *R* das Minimum `lo` und das Maximum `hi` im Quadratfenster mit Radius `r` (1 bis 10 px, Standard 4), getrennt in zwei
   Durchläufen (waagerecht, senkrecht), am Bildrand abgeschnitten.
2. Kantengewicht `t = smoothstep((hi − lo − 0,10) / (0,30 − 0,10))`: unter 0,10 Kontrast keine Wirkung (Wolkenbänder und Krater haben 0,05 bis
   0,15), ab 0,30 volle Wirkung (ein Planetenrand hat 0,3 und mehr). Die Schwellen sind **geschätzt**, nicht gegen eine Referenz geprüft.
3. `S' = S + Stärke · t · max(0, lo − S)`. *S'* ist nie kleiner als *S*, und wo *S* ≥ `lo` ist, ändert sich nichts. Bei Stärke 100 % und
   vollem Kantengewicht liegt *S'* nie unter dem lokalen Minimum des ungeschärften Bildes *R* im Radius `r`.

Nur Unterschreitungen werden behandelt, nicht das helle Überschwingen im Planeten. Grenze des Verfahrens: ein Ring, der breiter als `r` ist,
bleibt in der Mitte stehen; dann hilft ein größerer Radius.

### Denoise

Kantenerhaltende Glättung, ein **bilateraler Filter** über ein 5 × 5-Fenster, vom Ergebnis mit dem ungefilterten Bild mit der Stärke gemischt:

- Gewicht eines Nachbarn: `exp(−d² / 2σs²) · exp(−e² / 2σr²)`, `d` Abstand in Pixeln (σs = 1,2), `e` die Wurzel des mittleren Quadrats der
  Kanaldifferenzen zum mittleren Pixel (alle Kanäle zugleich, damit auch Farbkanten bleiben).
- `σr` wächst mit der Stärke *s* (0 bis 1) von 0,01 auf 0,15: `σr = 0,01 + 0,14 · s`.
- `S' = S + s · (Mittel − S)`; Nachbarn außerhalb des Bildes zählen nicht.

Eigenschaften (Tests): Stärke 0 bitgleich; ein ebenes Bild bleibt eben; das Rauschen eines ebenen Feldes nimmt mit der Stärke ab; eine Kante mit
Kontrast 0,75 ändert sich um weniger als 0,01.

### Maske

Beschränkt das Schärfen auf einen Bereich, meist die Scheibe:

1. Binärmaske aus der **Helligkeit von *R*** (Luminanz): 1 bei `L ≥ Schwelle / 255`, sonst 0 (Schwelle 0 bis 255, Standard 20).
2. Weiche Kante: Gauß-Unschärfe mit σ = `Feather / 2` Pixel (Feather 0 bis 50, 0 = harte Kante), abgeschnitten bei 3σ, Rand wiederholt. Ein überall
   gleiches Maskenbild bleibt exakt 0 oder 1 (Werte nahe 0 und 1 werden gesetzt).
3. **Invert** dreht die Maske um (`1 − m`).
4. Mischung: `S' = R + m · (S − R)`, bei `m = 0` genau *R*, bei `m = 1` genau *S*. Weil die Maske **zuletzt** wirkt, bleibt auch das
   Entrauschen und Entringen auf den maskierten Bereich beschränkt.

**Show mask** (nur Ansicht): `mask_overlay` legt die Maske blau (0,1 / 0,6 / 1,0; Deckkraft 0,55 bei Maske 1) über das Bild, Sitzungsmethode
`Session::refine_mask_overlay`. Das Bild unter der Einfärbung ist das Ausgangsbild der Karten, ohne die anderen Karten.

### Einbindung

- Werte: `RefineSettings` (Kern, serde mit `default`) im Feld `refine` von `ColorSettings` (Sitzung, `#[serde(default)]`: fehlt das Feld,
  bleibt das Bild unverändert).
- Projektdatei: eigener, optionaler Abschnitt `refine` neben `align`, `stack`, `wavelet` und `colour` (`denoise`, `deringing`, `radius`,
  `maskOn`, `threshold`, `feather`, `invert`; Show mask wird nicht gespeichert). Ein Projekt ohne den Abschnitt lässt die Werte der Karten,
  wie sie sind.
- Zurücksetzen: **Reset** in der Karte und `resetTone` (Restore) setzen auf „aus“; der Standard kommt aus dem Kern
  (`refine_default_settings`).

## Schemadateien

Wavelet-Schemata liegen im **`.rwv`-Format** (INI-Datei, `docs/formats.md`). Es gilt: Groß-/Kleinschreibung egal, fehlende Schlüssel
behalten ihren Standardwert, unbekannte werden ignoriert, Werte werden auf die Grenzen oben gekürzt, ein Fehler nennt die Zeile.
Geschrieben wird mit CRLF, byteidentisch zu einer Schemadatei im Format von `docs/formats.md` (Test `writes_the_reference_scheme_file`). Für *Available schemes* fehlen nur
Ordner und Dateidialog; Lesen und Schreiben stehen (`scheme::from_rwv`, `scheme::to_rwv`).
Die Datei speichert Gain mit einer Nachkommastelle und Sharpen und Denoise mit zwei (ganze Zahlen ×10 bzw. ×100); beim Schreiben wird darauf gerundet.

## Regressionsprüfung und PSNR-Schwelle

Die Tests in `crates/io/tests/wavelet_reference.rs` vergleichen das Ergebnis von *Do All* mit gespeicherten Ergebnissen **dieses Programms**
für Referenzfall R1 (Jupiter-Testaufnahme). Sie liegen in `re-local/reference-own/` (`jupiter-stack.tif` als Eingabe,
`jupiter-wavelet-default.tif` und `jupiter-wavelet-l1-10.tif` als Ergebnisse, `jupiter-l1-10.rwv` als Schema) und sind mit
`scripts/gen-reference.sh` erzeugt (Vorgehen und Zeitpunkt: `docs/stacking.md`, „Regressionsprüfung“). `re-local/` ist nicht in Git; fehlt
`re-local/reference-own`, werden die Tests übersprungen (`PLANETS_RE_LOCAL` zeigt auf ein anderes `re-local/`, etwa aus einem
Worktree). Gerechnet wird mit dem 16-Bit-Spitzenwert 65535; Eingabe ist das gespeicherte Summenbild, damit das Stacking nicht mitzählt.

Die Tests fangen **Änderungen am Verhalten** ab, sie sagen nichts darüber, ob das Verhalten richtig ist; der Vergleich mit einem fremden
Programm ist kein Teil der Prüfung mehr.

| Test | Was er prüft | Schwelle |
|---|---|---|
| `default_settings_match_the_stored_reference` | Standardwerte (mit Kennlinie) gegen das gespeicherte Ergebnis | **≥ 60 dB** |
| `layer_one_at_ten_matches_the_stored_reference` | Ebene 1 = 10.0 gegen das gespeicherte Ergebnis; das gespeicherte Schema muss als dieselben Werte gelesen werden; das gespeicherte Ergebnis muss mehr als die Schwelle vom ungeschärften Summenbild entfernt sein (sonst prüfte der Test die Schärfung nicht) | **≥ 60 dB** |

Warum 60 dB: Bei gleichem Code und gleicher Eingabe ist das Ergebnis bitgleich (unendlich dB; gemessen im Debug- und im Release-Bau am Mac).
Zwischen Plattformen können Gleitkomma-Rundungen (FMA, Bibliotheksmathematik, Summenreihenfolge) einzelne Proben um wenige Einheiten von
65535 verschieben, das liegt weit über 90 dB. Jede am Bild sichtbare Änderung der Algorithmen kostet viel mehr: ein Prozent Helligkeit sind
54 dB, ein Graustufenwert von 255 über das ganze Bild 48 dB. 60 dB liegt dazwischen. Ebene 1 = 10.0 liegt beim ungeschärften Summenbild
bei 53,98 dB, also unter der Schwelle: Eine Schärfung, die nicht wirkt, fiele durch.

Die Kennlinie (`output_curve`) wurde an einem Ergebnis eines anderen Programms angepasst (Tabelle im Unit-Test in `output.rs`). Die
Regressionstests schützen sie nur noch gegen Änderungen, nicht gegen das fremde Ergebnis.

Für die Ebenen 2 bis 6, Dyadic, den Filter Default, Denoise, Sharpen ungleich 0.100, Initial Layer und Step Increment ungleich 1 und 0
sowie abgeschaltete Häkchen gibt es kein gespeichertes Ergebnis; dort prüfen nur Unit-Tests die Eigenschaften (Bitgleichheit bei neutralen
Reglern, Vorschau gleich *Do All*, Skalierung durch Sharpen).
