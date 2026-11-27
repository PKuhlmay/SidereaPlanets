# Ablauf

Der Weg von der Aufnahme zum geschärften Bild in Siderea Planets. Bedienelemente, Werte und Bereiche stehen in `docs/ui.md`,
das Aussehen der Schritte in `docs/design.md`, die Rechnung in `docs/stacking.md` und `docs/wavelets.md`.

## Überblick

Die Schrittleiste oben im Fenster zeigt sieben Schritte, das Panel links nur den offenen Schritt mit seiner Aktion:

```
Open ► Alignpoints ► Align ► Best Frames ► Stack ► Wavelets ► Save
Select     Set          Align   Keep Best    Stack   Do All     Save
Recording… Alignpoints          Frames                          Image
```

Jeder Schritt hat einen Zustand: erledigt (✓), nächster offener Schritt, möglich oder gesperrt. Ein Schritt lässt sich erst
anwählen, wenn der vorige gelaufen ist; nach jedem Lauf öffnet sich der nächste offene Schritt von selbst. Die Zustände und das
Verhalten beim erneuten Ausrichten stehen in `docs/design.md`, Abschnitt „Zustände“. Hinter dem Schritt Best Frames steckt die
Funktion *Limit* des Rechenkerns (`crates/session/src/align.rs`); die Namen im Code und in den Tickets sind deshalb „Limit“.

## 1. Laden (Open)

1. **Select Recording…** (Werkzeugleiste oder Schritt Open, auch per Drag and Drop auf das Fenster) öffnet einen Dateidialog
   „Open file(s)“. Dateitypen siehe `docs/formats.md`. *Recent Recordings* öffnet eine der zuletzt benutzten Aufnahmen.
2. Nach dem Laden erscheint das erste Frame im Bildbereich. Rechts in der Schrittleiste stehen Größe, Frame-Zahl und Codec,
   unten die Frame-Leiste mit *Go to frame*, Pfeiltasten und einem Regler über alle Frames („Frame 5 of 10“).
3. Ob die Aufnahme farbig ist, erkennt die App am Inhalt (Mono oder Farbe) und setzt danach das Häkchen **Color**; eine Rückfrage gibt
   es nicht.
4. Danach ist **Set Alignpoints** möglich. **Align**, **Keep Best Frames** und alle weiteren Schritte sind gesperrt.

## 2. Alignpoints, Align und Best Frames

### 2.1 Set Alignpoints
- Setzt automatisch Ausrichtungspunkte auf das aktuelle Frame, als rote Punkte über dem Bild.
- Wie viele Punkte entstehen, hängt von *Min. distance between points*, *Min. distance from image edge*,
  *Brightness measure* mit *Min. brightness* und *Max. brightness* und vom Regler *Threshold* ab. Eine Region (**Draw Region**) schränkt
  die Punkte auf einen Bildausschnitt ein.
- Die Beschriftung über dem Regler wechselt von „Threshold“ zu „Threshold · n points“, das Feld *Number of alignpoints* zeigt n. Der Regler
  reicht nach dem Setzen von 0 bis zur Zahl aller Kandidaten: links bleiben mehr Punkte, rechts weniger.
- Danach ist **Align** möglich.

### 2.2 Align
- Richtet alle Frames an den Alignpoints aus und bewertet dabei die Qualität jedes Frames.
- Fortschritt erscheint in der Statusleiste („Aligning frame: n“, Prozentbalken) und im Fenstertitel („Siderea Planets · Align 40%“).
  Zum Schluss steht in der Statusleiste „Alignment finished“ mit benötigter und verbleibender Zeit.
- Danach ist **Keep Best Frames** möglich. **Set Alignpoints** und **Align** bleiben bedienbar: Man kann nach Limit, Stack und
  Wavelets andere Punkte, eine andere Region oder eine andere Boxgröße ausprobieren, ohne die Aufnahme neu zu öffnen. Ein neuer Lauf
  verwirft Limit, Stack, Wavelet-Ergebnis, Farbanpassung und Frame-Auswahl (Einzelheiten in `docs/design.md`, Abschnitt „Zustände“).
- Mit *Show ▾ ▸ Show Frame List* erscheint die Frame-Liste, sortiert nach Qualität (bestes Frame oben), jedes Frame mit
  Häkchen. Mit *Show Quality Graph* erscheint ein Liniendiagramm mit Qualität und Differenz je Frame.

### 2.3 Keep Best Frames (Limit)
- Begrenzt die Frames nach den Einstellungen des Schritts. Standard: *At least % of the best quality* = 80.
- Die App wechselt danach **selbst in den Stack-Schritt**. Die Statusleiste meldet „Keeping n of m frames“ und
  „n frames will be stacked · weakest kept: x%“, wobei x die Qualität des schwächsten behaltenen Frames in Prozent des besten ist
  (Prozentwerte der Frame-Liste; Entwurfsentscheidung).
- Im Bild erscheinen die Alignpoints als gelbe Kreise mit grünen Strichen. Die Striche zeigen den mittleren Versatz des Punkts.
- *Go to Best Frame* zeigt das beste Frame der Rangliste und öffnet den Schritt Alignpoints, damit man alle Frames auf dieses
  Frame ausrichten kann.

## 3. Stack

- **Stack** summiert die behaltenen Frames. Die Statusleiste meldet danach „Stacked n frames · 1024×680“ und
  „Value range 0–255“, im Bild steht das Summenbild.
- Optional, vor dem Stacken einstellbar: die Stacking Options (Ausgabegröße, *Use nearest points*, *Correct geometry*, *Even out
  frame brightness*, *De-rotate images*, *Ignore edge pixels*). *Resampling* und *Drizzling* sind angelegt, aber noch ohne Funktion
  (gesperrt).
- Unabhängig vom Stacken lassen sich ausgerichtete oder gemittelte Sequenzen als AVI speichern (Gruppe *Create AVI* unter Advanced).
- Der Schritt Wavelets ist nach dem Stacken möglich.

## 4. Wavelets

- Das Summenbild erscheint im Bildbereich. Links liegt der Schritt Wavelets mit 6 Ebenen, rechts das Panel *Adjustments*
  (Histogram, Gamma, RGB-Abgleich, Color Mixing und weitere) mit *Contrast/Brightness*; **Adjust ›** oben im Bild blendet es ein und aus.
- Verschiebt man einen Ebenen-Regler, rechnet die App die Vorschau neu („Updating preview…“, danach „Preview calculated in n ms“).
  Die Vorschau rechnet nur im quadratischen Bereich in der Bildmitte, der *Preview area* (Settings, Standard 512 Pixel), sichtbar mit
  *Show Preview Area*.
- **Update Preview** rechnet die Vorschau mit den aktuellen Einstellungen neu. **Do All** wendet sie auf das ganze Bild an; mit dem
  Häkchen *Run Do All automatically* geschieht das kurz nach jeder Änderung.
- Wavelets gilt als erledigt, solange das Ergebnis von Do All zu den aktuellen Werten passt. Ändert man danach eine Ebene, ist
  Wavelets wieder der nächste offene Schritt.
- Wavelet-Einstellungen lassen sich als Schema speichern und laden (*Save Scheme…*, *Load Scheme…*), Dateiformat siehe `docs/formats.md`.
- *Realign with Processed* und *Stack Again* sind angelegt, aber noch ohne Funktion (gesperrt).

## 5. Save

- **Save Image** schreibt das geschärfte Bild mit den Farb- und Tonwerten der Adjustments in die gewählte Datei (Formate siehe
  `docs/formats.md`). Ist das Ergebnis von Do All veraltet, wird Do All vorher nachgeholt; Save ist deshalb schon nach Stack
  möglich.
- Der Schritt Save zeigt außerdem die Gruppen *Crop* (Bild auf ein Rechteck zuschneiden) und *Size* (Größe der Datei nach Flip and
  Rotate und Resize Image).
- Der Schritt gilt als erledigt, solange das gespeicherte Bild dem aktuellen Do-All-Ergebnis entspricht; jede Änderung an den
  Wavelet-Werten, ein neuer Stack oder ein erneuter früherer Schritt nimmt den Haken wieder weg.

## 6. Ergänzende Funktionen außerhalb der Schritte

- **Recent Recordings**: zuletzt geöffnete Aufnahmen.
- **Open Project…** und **Save Project…** (Werkzeugleiste): die Aufnahme und die Einstellungen einer Sitzung als Projektdatei,
  siehe `docs/formats.md`.
- **Settings**: *Preview Area*, *Theme*, *Show Advanced Controls* (öffnet den Abschnitt Advanced in jedem Schritt), *Show CPU and Memory
  Usage*. Einträge ohne Funktion sind gesperrt und unter „Not available yet“ zusammengefasst.
- **Cancel** und **Pause** unterbrechen laufende Berechnungen (Align, Stack, Update Preview, Do All, AVI-Speichern). **CPUs** legt die
  Zahl der Threads fest; Standard ist die Zahl der logischen Kerne.
- Fortschritt und Abbrechen gelten für Align, Stack, Do All und die AVI-Läufe (SP-010).
