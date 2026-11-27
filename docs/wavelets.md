# Wavelet-Schärfung (SP-007)

Was der Rechenkern (`crates/core/src/wavelet/`) tut, was davon am Original **belegt** ist und was **geraten**.
Bedienelemente und Standardwerte stehen in `docs/ui.md`, der Ablauf in `docs/workflow.md` (Abschnitt 4), das
Schemaformat in `docs/formats.md`. Offene Punkte: `docs/offene-fragen.md` (ab Frage 22).

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
Danach folgt die Tonkennlinie des Originals (nächster Abschnitt); erst sie macht *Do All* und die Vorschau zum Ergebnis des Originals.
Jeder Farbkanal wird für sich gerechnet, solange *Colour* an und *LRGB* aus ist (Abschnitt Colour und LRGB). Ebenen, die nichts ändern, kosten nichts; hinter der letzten
veränderten Ebene wird nicht weiter zerlegt.

### Glätten

Getrennter 3- bzw. 5-Tap-Kern (erst waagerecht, dann senkrecht), Taps `Abstand_n` Pixel auseinander
(à trous), Bildränder durch Wiederholen der Randpixel.

| Wavelet filter | Kern (eindimensional) | Stand |
|---|---|---|
| Gaussian (Standard) | `0.05 · 0.90 · 0.05` | **belegt** für Ebene 1 (siehe unten) |
| Default | B3-Spline `1 4 6 4 1 / 16` | **geraten** (übliche Wahl bei à-trous-Wavelets) |

| Waveletscheme | Abstand der Taps in Ebene `n` (n ab 0) | Stand |
|---|---|---|
| Linear (Standard) | `Initial + n · (1 + Step)`, mit Standardwerten 1, 2, 3, 4, 5, 6 | Ebene 1 **belegt**, Rest **geraten** |
| Dyadic (2^n) | `Initial · 2^n`: 1, 2, 4, 8, 16, 32 | **geraten** |

Wegen des Taps-Abstands hängt ein Pixel des Ergebnisses von Pixeln bis zu `Σ Abstand · Reichweite` entfernt ab
(Standard 21 px, Dyadic mit Gaussian 63 px). Diesen Rand (`Structure::margin`) rechnet die Vorschau mit.

### Tonkennlinie des Originals

Das Original verändert das Bild auch bei neutralen Reglern (Beobachtung 1 unten). Der Klon wendet dieselbe Kennlinie am Ende
von *Do All* und der Vorschau an (`wavelet/output.rs`, `output_curve`), nach den Ebenen und vor dem Zurückholen der Farbe bei LRGB:

```
y = 0,988231 · x + 0,015595 · x³          x, y auf 0…1 bezogen, x vorher auf 0…1 begrenzt, y höchstens 1
```

Was die Daten belegen und was nicht (alles aus `jupiter-stack.tif` gegen `jupiter-wavelet-default.tif`, 16 Bit, ausgewertet mit Python):

| Punkt | Stand |
|---|---|
| Ein Eingabewert gibt immer denselben Ausgabewert (1455 verschiedene Werte, größte Streuung 1 von 65535), für alle Kanäle gleich | **belegt** |
| Form: ungerades Polynom dritten Grades ohne quadratischen Anteil. Die freie Anpassung `a·x + b·x² + c·x³` ergibt `b ≈ −0,0005`, also null. Ein Polynom zweiten Grades (frühere Näherung 0,984·x + 0,0153·x²) verfehlt bis zu 18 von 65535, dieses höchstens 3 | **belegt** als Näherung; die wahre Formel des Originals ist unbekannt |
| Genauigkeit: mit Rundung auf die nächste Ganzzahl 98,8 dB gegen die Standardausgabe (44 % der Samples exakt, höchstens 3 daneben) | **belegt** |
| Abstand zur Identität: bis zu einer Stufe von 8 Bit (256 von 65535) um die Mitte des belegten Bereichs, in den Tiefen etwa 1,2 % | **belegt** |
| Bereich: im Referenzbild kommen nur Werte bis 40063 (0,61) vor. Darüber ist die Kurve eine Fortsetzung. Sie steigt weiter, erreicht bei 1,0 den Wert 1,0038 und wird dort auf 1 begrenzt; ob das Original oben so abbiegt, ist unbekannt (`docs/offene-fragen.md`, Frage 22) | **geraten** |
| 8-Bit-Rundung: Die 16-Bit-Ausgabe hat Werte, die keine Vielfachen von 256 sind; die Kennlinie wirkt also in 16-Bit-Auflösung, nicht auf einer 8-Bit-Treppe. Der Klon rechnet in Fließkomma und rundet erst beim Speichern | **belegt** (keine Treppe) |
| Ursache (Contrast/Brightness 100/0, ein Skalierungsschritt, die Zusammensetzung der Ebenen?) | **unbekannt** |

*Before* zeigt das Summenbild ohne die Kennlinie. Bei Graustufen wurde sie nicht beobachtet (die Referenz ist farbig); der Klon wendet sie
auf alle Kanalzahlen gleich an.

### Colour und LRGB

Die Häkchen *Colour* und *LRGB* (Align- und Stack-Reiter, ein Wert für alle Reiter, `docs/ui.md`) wählen `wavelet::ColourMode`:

| Colour | LRGB | `ColourMode` | Rechnung |
|---|---|---|---|
| an | aus | `Colour` | jeder Kanal für sich |
| an | an | `Lrgb` | Ebenen auf der Luminanz (`Image::to_luma`), danach Kennlinie, dann `color::with_luminance`: Farbton und Sättigung des Summenbilds, Luminanz = Ergebnis |
| aus | | `Gray` | Luminanz des Summenbilds, Ebenen, Kennlinie; das Ergebnis hat einen Kanal |

LRGB zählt ohne Colour nicht. Bei Bildern mit weniger als drei Kanälen (Graustufenaufnahmen) ändert sich nichts. Die Zuordnung ist aus den
Namen der Häkchen geschlossen und **nicht** am Original beobachtet (`docs/offene-fragen.md`, Frage 26 im Abschnitt Farbe und Tonwerte).

### Einstellungen je Ebene

| Feld | Wirkung im Klon | Stand |
|---|---|---|
| Regler (Gain, −5.0 … 100.0, Reglerposition ÷ 10) | Faktor der Ebene, 1.0 = unverändert, 0 = Ebene entfernt, negativ kehrt sie um | **belegt** für Ebene 1 mit 10.0 |
| Sharpen (Standard 0.100) | skaliert die Abweichung des Reglers von 1.0: `1 + (Gain − 1) · Sharpen / 0.1`. Bei 0.100 wirkt der Regler genau wie angezeigt | **geraten**, nur der Wert 0.100 ist belegt |
| Denoise (Standard 0.00) | Soft-Threshold der Ebene vor der Verstärkung: Schwelle = `Denoise · σ`, σ = 1,4826 · Median der Beträge der Ebene (je Kanal). 0 = aus | **geraten** |
| Häkchen | aus = Ebene bleibt unverändert (Gain 1, kein Denoise) | **geraten** |

Grenzen: Der Klon begrenzt Gain auf −5 … 100 (wie die Regler des Originals), Denoise und Sharpen auf 0 … 100,
Initial Layer auf 1 … 64, Step Increment auf 0 … 64 (`WaveletSettings::clamped`). Das Original lässt beim Tippen
beliebige Werte zu (`docs/offene-fragen.md`, Frage 1).

*Automatic*, *Hold Wavelet Setting*, *Use Linked Wavelets* und die Schaltfläche *Preview* je Ebene betreffen nur
die Bedienung und liegen nicht im Rechenkern (SP-007 Phase 2).

## Was am Original beobachtet wurde

Alles aus `re-local/reference/` (Referenzfall R1, `docs/testdaten.md`), Eingabe ist das Original-Summenbild
`jupiter-stack.tif`. Ausgewertet mit einem Python-Skript (numpy), **nicht** mit dem Rust-Code. Es liegt nicht im
Repository; die Schritte stehen hier, damit sie sich wiederholen lassen.

1. **Standardwerte (`jupiter-wavelet-default.tif`) verändern das Bild nur durch eine Tonwertkennlinie** (im Klon umgesetzt, Abschnitt oben).
   Dasselbe Eingabe-Sample ergibt immer dasselbe Ausgabe-Sample (Streuung 0 bei 1448 verschiedenen Werten im
   Rotkanal), die Zuordnung ist für alle drei Kanäle gleich. Es gibt keinen räumlichen Anteil. Die Kennlinie
   macht das Bild leicht dunkler und ist leicht gekrümmt: `y ≈ 0,98394·x + 0,01534·x² + 0,0003` (x, y auf 0…1
   bezogen; größte Abweichung der Näherung 9 von 65535 oberhalb von 3000). Die genauere Form ist ein Polynom dritten Grades
   ohne quadratischen Anteil (Abschnitt Tonkennlinie; die größte Streuung bei gleichem Eingabewert ist 1, nicht 0, durch Rundung).
   Woher sie kommt, ist offen.
2. **Ebene 1 auf 10.0 (`jupiter-wavelet-l1-10.tif`)**: Die Differenz zur Standardausgabe lässt sich als lineares
   Filter des Summenbilds beschreiben. Eine freie 7×7-Anpassung (R² = 0,979) liefert Gewichte
   Mitte 1,649 · Kreuz-Nachbarn −0,3875 · Diagonalen −0,023 · alles andere unter 0,003, Summe ≈ 0. Das ist
   „Bild minus leicht geglättetes Bild“, mit dem getrennten Kern `0.05 · 0.90 · 0.05` und einem Faktor
   9 = 10 − 1.
3. **Modell gegen die Referenz**: `L ≈ f(S + 9·(S − K·S))` mit `K` = getrennter Kern, `f` die Kennlinie aus Punkt 1
   (Tabelle aus Eingabe/Standardausgabe), ergibt gegen `jupiter-wavelet-l1-10.tif` **85,2 dB** (Ränder gespiegelt).
   Schon kleine Abweichungen verschlechtern das deutlich: Tap 0,045 → 75,4 dB, 0,055 → 76,6 dB, Faktor 8 oder 10 →
   höchstens 75,5 dB. Ohne Kennlinie sind es 55,5 dB.

Damit sind **Kernform und Verstärkung von Ebene 1** bei Linear, Gaussian, Sharpen 0.100, Initial 1, Step 0 und
Denoise 0 gut belegt. Dass `0.05 = Sharpen / 2` ist, passt, belegt aber nicht, dass Sharpen die Kernbreite steuert
(Frage 24).

**Nicht gegen das Original geprüft** (es gibt dazu keine Referenzdatei): Ebenen 2 bis 6, Dyadic, der Filter
Default, Denoise, Sharpen ungleich 0.100, Initial Layer und Step Increment ungleich 1 und 0, abgeschaltete Häkchen.
Dafür bräuchte es weitere Läufe mit dem Original (Vorschlag in Frage 23).

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
  (SP-010-Prüflauf: Wechsel nach Save zeigte „Preview: 79 ms“).
- Die Antwort wird in einen Puffer geschrieben, der eine Kopie des Summenbilds ist; `session.processedImage` teilt sich
  diesen Puffer, eine Vorschau kostet also nur den Bereich. Außerhalb der Processing Area bleibt das Summenbild.
- Die Statusleiste zeigt die Zeit von Anfrage bis Zeichnen (`Preview: n ms`). Das ist die Messung der Übertragung, die
  `docs/gui-toolkit.md` verlangt; **es liegen noch keine Messwerte vor** (App nicht gestartet).
- **Process** und **Do All** laufen über `runJob`. Do All ersetzt `session.processedImage` durch das ganze Bild.
- **Load Scheme / Save Scheme**: Dateidialog (Plugin `dialog`, Filter `.rwv`) in `pickSchemePath`, das ist die Stelle, die
  SP-009 durch `pick_scheme_path` ersetzt.
- **Realign with Processed** ist nicht angebunden (braucht Align/Stack, SP-005/SP-006).
- Ohne gestapeltes Bild im Backend (`WaveletState::set_source` wurde nicht aufgerufen) meldet die Statusleiste
  „no image to sharpen“. Solange SP-006 fehlt, ist der Reiter ohnehin gesperrt.
- Automatic, Hold und Linked: geraten, siehe `docs/offene-fragen.md`, Frage 29.

## Tauri-Befehle (`app/src-tauri/src/wavelet_commands.rs`)

Die Befehle sind dünne Hüllen über `planets_session::Session` (`crates/session/src/wavelet.rs`, dort Zustand und Rechnung).

Einstellungen gehen als JSON mit den Feldern von `WaveletSettings` (camelCase): `scheme` (`"dyadic"`/`"linear"`),
`filter` (`"default"`/`"gaussian"`), `initial`, `step`, `layers[6]` mit `enabled`, `gain`, `denoise`, `sharpen`.

| Befehl | Zweck |
|---|---|
| `wavelet_default_settings()` | Werte für *Reset Wavelets* |
| `wavelet_set_source_frame(index)` | Platzhalter: Frame der Aufnahme als Bild des Reiters, bis SP-006 `WaveletState::set_source` nach *Stack* aufruft |
| `wavelet_preview(settings \| null, area, mode)` | Vorschau mit Kennlinie und Farbbehandlung wie *Do All*; `mode` ist `"colour"`, `"lrgb"` oder `"gray"`. Antwort binär: x, y, Breite, Höhe (4 × u32 LE, der auf das Bild zugeschnittene Bereich), dann RGBA. `null` = unbearbeitet (*Before*) |
| `wavelet_do_all(settings, mode)` | ganzes Bild mit Kennlinie und Farbbehandlung; Antwort binär wie `read_frame` (Breite, Höhe als u32 LE, RGBA). Das Ergebnis bleibt in `WaveletState::processed()` für *Save image* (SP-009) |
| `save_wavelet_scheme(path, settings)` / `load_wavelet_scheme(path)` | `.rwv` schreiben und lesen |

## Denoise/Deringing und Masking (MAC-018, eigener Entwurf)

Die Karten **Denoise/Deringing** und **Masking** des Adjustments-Panels (Mac) haben im Original keine beobachtete Entsprechung
(das Original wurde dazu nicht beobachtet); die Verfahren sind unser Entwurf, einfach gehalten und im Rechenkern
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
   vollem Kantengewicht liegt *S'* nie unter dem lokalen Minimum des Originals im Radius `r`.

Nur Unterschreitungen werden behandelt, nicht das helle Überschwingen im Planeten. Grenze des Verfahrens: ein Ring, der breiter als `r` ist,
bleibt in der Mitte stehen; dann hilft ein größerer Radius.

### Denoise

Kantenerhaltende Glättung, ein **bilateraler Filter** über ein 5 × 5-Fenster, vom Ergebnis mit dem Original mit der Stärke gemischt:

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

- Werte: `RefineSettings` (Kern, serde mit `default`) im Feld `refine` von `ColorSettings` (Sitzung, `#[serde(default)]`: die Tauri-Befehle der
  Windows-App senden es nicht und lassen das Bild unverändert). Mac: `RefineSettings` und `MaskSettings` der FFI, Swift `RefineValues`
  (`mac/App/Model/RefineLogic.swift`).
- Projektdatei: eigener, optionaler Abschnitt `refine` neben `align`, `stack`, `wavelet` und `colour` (`denoise`, `deringing`, `radius`,
  `maskOn`, `threshold`, `feather`, `invert`; Show mask wird nicht gespeichert). Die Abschnitte `wavelet` und `colour` bleiben im Layout der
  Windows-App; sie kennt den neuen Abschnitt nicht und ignoriert ihn, und ein Projekt ohne ihn lässt die Werte der Karten, wie sie sind.
- Zurücksetzen: **Reset** in der Karte und `resetTone` (Restore) setzen auf „aus“; der Standard kommt aus dem Kern
  (`refine_default_settings`).

## Schemadateien

Der Klon liest und schreibt das **`.rwv`-Format des Originals** (`docs/formats.md`), damit vorhandene Schemata weiter
gelten. Zusätzlich gilt: Groß-/Kleinschreibung egal, fehlende Schlüssel behalten ihren Standardwert, unbekannte
werden ignoriert, Werte werden auf die Grenzen oben gekürzt, ein Fehler nennt die Zeile. Geschrieben wird mit CRLF,
byteidentisch zu `jupiter-l1-10.rwv`. Der Klon führt kein eigenes Format ein. SP-009 muss für *Available schemes* nur
Ordner und Dateidialog ergänzen; Lesen und Schreiben stehen (`scheme::from_rwv`, `scheme::to_rwv`).
Die Datei speichert Gain mit einer Nachkommastelle und Sharpen und Denoise mit zwei (ganze Zahlen ×10 bzw. ×100); beim Schreiben wird darauf gerundet.

## Vergleich mit dem Original und PSNR-Schwellen

Gerechnet wird mit 16-Bit-Spitzenwert 65535, Eingabe ist das Original-Summenbild (damit zählt das Stacking nicht mit).

| Schwelle | Was sie prüft | Wert |
|---|---|---|
| A, Ticket | Ergebnis gegen die Original-Ausgabe bei gleichen Reglerwerten (Standard und Ebene 1 = 10.0) | **≥ 45 dB** |
| B, streng | Ebene 1 = 10.0, Ergebnis des Klons mit seiner Kennlinie direkt gegen die Referenz | **≥ 65 dB** |
| C, Kennlinie | Standardwerte direkt gegen die Referenz. Die Kennlinie ist an genau diesem Bild angepasst, die Schwelle schützt sie, nicht die Schärfung | **≥ 85 dB** |

Warum zwei: Das Summenbild liegt schon bei 55,6 dB zur Standardausgabe und bei 52,8 dB zur Ebene-1-Ausgabe, A
besteht also auch ein Klon, der gar nicht schärft. B liegt klar über den 56,0 dB, die „Original-Standardausgabe gegen
Original-Ebene-1“ erreicht, und beweist damit, dass die Schärfung stimmt. Für das Modell in Python wurden 85,2 dB
ermittelt (Punkt 3 oben).

Geprüft von `crates/io/tests/wavelet_reference.rs` (ohne `re-local/reference` übersprungen; `PLANETS_RE_LOCAL` zeigt auf ein anderes
`re-local/`, etwa aus einem Worktree). **Gemessen am 2026-11-11 mit dem Rust-Code**, Eingabe `jupiter-stack.tif`, Ergebnis auf 16 Bit gerundet:

| Fall | Gemessen | Schwelle |
|---|---|---|
| Standardwerte gegen `jupiter-wavelet-default.tif` | **98,81 dB** (ohne Kennlinie 55,62 dB) | A 45, C 85 |
| Ebene 1 = 10.0 gegen `jupiter-wavelet-l1-10.tif` | **84,73 dB** (ohne Kennlinie 55,61 dB, das Summenbild allein 52,84 dB) | A 45, B 65 |

Der Wert für Ebene 1 liegt knapp unter den 85,2 dB des Python-Modells (dort mit der gemessenen Wertetabelle statt der Kurve, Ränder gespiegelt statt
wiederholt). Für die Ebenen 2 bis 6, Dyadic, Denoise und die übrigen Werte gibt es keine Referenz.
