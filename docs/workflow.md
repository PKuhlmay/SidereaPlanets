# Ablauf im Referenzprogramm

Beobachtet am Referenzprogramm, Version 6.1.0.0 (Dateidatum 02-04-2011) mit `JUPITER.AVI` (siehe `docs/testdaten.md`).
Bedienelemente, Werte und Bereiche stehen in `docs/ui.md`. Ungeklärtes steht in `docs/offene-fragen.md`.

## Überblick

```
Select ──► Align-Reiter ──────────────────────────► Stack-Reiter ──────► Wavelet-Reiter
           Set Alignpoints ► Align ► Limit ─(auto)─► Stack ► Save image  Do All ► Save image
```

Es gibt drei Reiter: **Align**, **Stack** und **Wavelet**. Stack und Wavelet lassen sich erst anwählen, wenn
der jeweils vorige Schritt gelaufen ist. Einen eigenen Schritt „Optimize“ wie in Version 5 des Referenzprogramms gibt es nicht mehr.
Ebenso fehlt der frühere Reiter „Final“, dessen Funktionen im Wavelet-Reiter liegen (Panel *Functions*).

## 1. Laden (Select)

1. **Select** in der Werkzeugleiste öffnet einen Dateidialog mit dem Titel „Open file(s)“.
   Dateitypen siehe `docs/formats.md`.
2. Nach dem Laden erscheint das erste Frame im Bildbereich. Die Titelleiste zeigt Pfad, Codec und Größe,
   z. B. `‹Programmname› processing AVI: …\JUPITER.AVI(DIB vids) (1024x680)`. Unter dem Bild stehen „Frame (1): 1/10“
   und ein Schieberegler über alle Frames.
3. Bei einem Farbvideo fragt das Referenzprogramm: *Image seems to be in colour, process in colour?* (Yes/No), dazu die Option
   „nicht mehr fragen, automatisch erkennen“. Diese Option entspricht *Settings ▸ Autodetect Colour/BW*.
   Mit **Yes** wird im Align-Reiter das Häkchen **Colour** gesetzt.
4. Im Align-Reiter sind danach **Set Alignpoints** sowie **No Align** und **R.o.Interest** aktiv.
   **Align** und **Limit** sind noch gesperrt.

## 2. Align-Reiter

### 2.1 Set Alignpoints
- Setzt automatisch Ausrichtungspunkte auf das aktuelle Frame, als rote Punkte über dem Bild.
- Wie viele Punkte entstehen, hängt von *Minimum distance between*, *Min distance from edge*,
  *Intensity_selection* mit *Lo*/*Hi* und vom Schieberegler *Threshold* ab.
- Mit den Standardwerten entstehen bei `JUPITER.AVI` **82 Alignpoints**. Die Beschriftung über dem Regler wechselt
  von „Threshold“ zu „Alignpoints :82“, das Feld *Number of Alignpoints* zeigt 82.
  Der Regler reicht nach dem Setzen von 0 bis 91 und steht auf 9.
- Danach ist **Align** aktiv.

### 2.2 Align
- Richtet alle Frames an den Alignpoints aus und bewertet dabei die Qualität jedes Frames.
- Fortschritt erscheint in der Statusleiste („Loading frame: n“, Prozentbalken) und im Taskleistentitel
  („R 6:Align 100%“). Zum Schluss steht in der Statusleiste „Alignment finished.“ mit benötigter und
  verbleibender Zeit.
- Danach ist **Limit** aktiv, **Set Alignpoints** und **Align** sind gesperrt (so im Original beobachtet; der Klon
  weicht hier bewusst ab, siehe „Was der Klon daraus übernimmt“).
- Mit *Show Framelist* erscheint die Frameliste, sortiert nach Qualität (bestes Frame oben), jedes Frame mit
  Häkchen. Mit *Show Registrationgraph* erscheint ein Liniendiagramm mit „Quality“ und „Difference“ pro Frame.

### 2.3 Limit
- Begrenzt die Frames nach *Limit Setup*. Standard: *Lowest Quality (%)* = 80.
- Das Referenzprogramm wechselt danach **selbst in den Stack-Reiter**. Die Statusleiste meldet
  „Processing limited to 10 frames“ und „Stacksize: 10 Low quality :100,00%“.
  Der Klon zeigt „Stacksize: n  Low quality: x.xx%“ mit x = Qualität des schwächsten behaltenen Frames in Prozent des besten
  (Prozentwerte der Frameliste). Das ist eine **Vermutung**: Beim Referenzfall steht im Original bei allen zehn Frames 100,00 %,
  der Klon käme dort auf den Wert des schwächsten Frames (etwa 97 %), weil die zehn Qualitätswerte nur etwa 3 % streuen.
- Im Bild erscheinen die Alignpoints als gelbe Kreise mit grünen Strichen. Die Striche zeigen _vermutlich_ den
  mittleren Versatz des Punkts.

## 3. Stack-Reiter

- **Stack** summiert die ausgewählten Frames. Die Statusleiste zeigt danach „Stack size (n=10) - (1024x680)“ und
  „Stacking Range : 0-255“, im Bild steht das Summenbild.
- **Save image** speichert das Summenbild (Formate siehe `docs/formats.md`).
- Optional, vor dem Stacken einstellbar: *Stacking options*, *Resampling* (Methode und Faktor) und
  *Drizzling* (Faktor). Laut Tooltip vergrößert Drizzling das Ergebnis auf das Doppelte, Resampling
  rechnet in 8 Bit auf ein größeres Bild.
- Unabhängig vom Stacken lassen sich ausgerichtete oder gemittelte Sequenzen als AVI speichern (Panel *Create AVI*).
- Der Wavelet-Reiter ist nach dem Stacken anwählbar.

## 4. Wavelet-Reiter

- Das Summenbild erscheint im Bildbereich. Links liegt das Panel *Wavelets* mit 6 Ebenen, rechts das Panel
  *Functions* (Histogramm, Gamma, RGB-Abgleich und weitere) sowie *Contrast/Brightness*.
- Verschiebt man einen Ebenen-Regler, meldet die Statusleiste „layer setting changed“ und nach der Berechnung
  „Calculating wavelets done“. Die Vorschau rechnet _vermutlich_ nur im Bereich *Processing Area*
  (Settings, Standard 512 Pixel), sichtbar mit *Show Processing Area*.
- **Process** wendet die Einstellungen an, _vermutlich_ auf den Vorschaubereich.
  **Do All** wendet sie auf das ganze Bild an, die Statusleiste zeigt dabei „Do_all processing“.
- **Save image** speichert das bearbeitete Bild, mit einer kleineren Formatliste als im Stack-Reiter.
- **Realign_with Processed** und **Stack Again** wiederholen _vermutlich_ Ausrichtung und Stacking mit dem
  geschärften Bild als Referenz. *Stack Again* war im Test gesperrt.
- Wavelet-Einstellungen lassen sich als Schema speichern und laden (*Save Scheme*, *Load Scheme*, Liste
  *Available schemes*), Dateiformat siehe `docs/formats.md`.

## 5. Ergänzende Funktionen außerhalb der Reiter

- **MRU**: zuletzt geöffnete Dateien.
- **Flat/Dark/Reference**: Darkframe laden, verwenden, als Raw-Darkframe verwenden oder erzeugen (mit Faktor,
  Standard 1,00); Flatfield laden, verwenden oder erzeugen; Referenzframe laden und verwenden.
  Die „Use …“- und „Create …“-Einträge sind gesperrt, bis etwas geladen ist.
- **Tools**: *Show MACRO/Batch window*; *Record Processing Information in Filename*.
- **Settings ▸ Automatic Processing**: *Align to Wavelet*, *Align to Stack*, *Stack to Wavelet*. Damit laufen
  mehrere Schritte ohne Klick hintereinander durch (_Vermutung_ nach Beschriftung, nicht ausprobiert).
- **Cancel** und **Pause** unterbrechen laufende Berechnungen. **CPUs** legt die Zahl der Threads fest.

## Was der Klon daraus übernimmt

- Gleiche Reihenfolge und Sperrlogik: Laden → Set Alignpoints → Align → Limit → (automatisch) Stack → Wavelet.
  **Bewusste Abweichung (Philipp, SP-005):** Set Alignpoints und Align bleiben nach Align, Limit, Stack und Wavelet
  bedienbar. Ein neuer Lauf verwirft Limit, Stack, Wavelet-Ergebnis, Farbanpassung und Frame-Auswahl, damit man ohne
  erneutes Öffnen der Aufnahme neu ausrichten kann. Das Original sperrt beide Schritte ab Align und kennt dieses
  Verhalten nicht; Einzelheiten in `docs/design.md`, Abschnitt „Zustände“.
- Gleiche Begriffe auf Schaltflächen und Reglern (Liste in `docs/ui.md`).
- Fortschritt und Abbrechen bei Align, Stack und Do All (SP-010).
- Die Nachfrage „in Farbe verarbeiten?“ ersetzt der Klon durch die automatische Erkennung, die im Original als
  Option existiert. Das ist eine bewusste Abweichung, die Philipp bestätigen sollte (siehe `docs/offene-fragen.md`).
