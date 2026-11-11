# Design der Oberfläche

Verbindliche Vorgabe für alles, was im Klon sichtbar ist. Umgesetzt wird sie im Frontend unter `app/src` (RK-010);
jedes Ticket, das Bedienelemente anfasst, hält sich daran.

Die Arbeitsteilung der Dokumente:

| Frage | Quelle |
|---|---|
| Welche Elemente gibt es, wie heißen sie, welche Standardwerte und Bereiche haben sie? | `docs/ui.md` |
| In welcher Reihenfolge läuft die Bedienung, was ist wann gesperrt? | `docs/workflow.md` |
| Wie sieht es aus, wo sitzt es, wie verhält es sich optisch? | dieses Dokument |

Bei Widersprüchen gewinnt für Begriffe und Werte `docs/ui.md`, für Aussehen und Anordnung dieses Dokument.

## Dateien

| Datei | Inhalt |
|---|---|
| `docs/design/tokens.css` | Farben, Schrift, Maße. Einzige Quelle für diese Werte, wird ins Frontend übernommen |
| `docs/design/oberflaeche-entwurf.html` | Klickbarer Entwurf aller drei Reiter, im Browser öffnen |
| `docs/design/screenshots/` | Soll-Bilder der Zustände bei 1440×900, erzeugt aus dem Entwurf |
| `docs/design/siderea-planets.figcraft` | Die drei Reiter und die Doku als FigCraft-Datei, je dunkel und hell (acht Frames 1440×900). Farben sind Variablen der Collection „Theme“ mit den Modi Dark und Light |
| `docs/design/figcraft-build.mjs` | Baut die FigCraft-Datei neu und liest die Farben dabei aus `tokens.css`. Aufruf: `node docs/design/figcraft-build.mjs` |

Die sechs Frames der Reiter entstehen aus dem Skript, die zwei Doku-Frames (RK-012) sind dagegen von Hand in
FigCraft gebaut und stehen nicht im Skript. Ein Lauf des Skripts schreibt die Datei neu und entfernt sie. Wer ihn
braucht, sichert die Doku-Frames vorher oder baut sie nach der Beschreibung unter [Doku](#doku) neu.

Der Entwurf ist eine Vorlage zum Ansehen, kein Code zum Übernehmen: Er ist eine einzelne HTML-Datei ohne Vue und
ohne Rechenkern. Markup und CSS-Regeln dürfen als Orientierung dienen, die Umsetzung baut eigene Komponenten.

## Grundsätze

1. **Gleicher Ort, gleicher Name.** Jedes Bedienelement sitzt dort, wo es in the reference program sitzt, und trägt dieselbe
   englische Beschriftung. Wer das Original kennt, findet alles ohne Suchen.
2. **Das Bild ist die Hauptsache.** Die Oberfläche ist neutral grau ohne Farbstich, damit sie die Farben des Bildes
   nicht verfälscht. Der Bildbereich bleibt in beiden Themen dunkel.
3. **Eine Akzentfarbe.** Bernstein, angelehnt an die orangen Schaltflächen des Originals. Sie markiert den nächsten
   Schritt, gewählte Zustände und Fortschritt, sonst nichts.
4. **Der nächste Schritt ist sichtbar.** Genau eine Schaltfläche der Schrittfolge ist gefüllt. Erledigte Schritte
   tragen einen Haken, noch nicht mögliche sind gesperrt.
5. **Dicht, aber ruhig.** Ein Werkzeug mit vielen Reglern: kleine Schrift, kompakte Felder, Trennung durch Linien
   und Abstand statt durch Rahmen und Kästen.
6. **Dunkel ist Standard.** Das helle Thema ist gleichwertig ausgearbeitet und umschaltbar.

## Tokens

Die Werte stehen in `docs/design/tokens.css` und nur dort. Im Frontend wird die Datei unverändert übernommen;
Komponenten benutzen ausschließlich die Variablen, keine eigenen Farbwerte.

| Gruppe | Variablen | Verwendung |
|---|---|---|
| Flächen | `--bg` `--panel` `--raised` `--raised-h` `--field` | Fenster, Leisten und Panels, Schaltflächen, Hover, Eingabefelder |
| Linien | `--line` `--line-strong` | Trennlinien zwischen Bereichen; Umrisse von Bedienelementen und Reglerspur |
| Text | `--text` `--muted` `--off` | Beschriftung und Werte; Zwischenüberschriften und Statusleiste; Gesperrtes |
| Akzent | `--accent` `--accent-h` `--on-accent` `--accent-ink` `--accent-soft` | Füllung, Hover, Text auf Füllung, Akzent als Text, gewählte Segmente |
| Status | `--ok` `--info` | erledigter Schritt und Punkt „Aligned“; zweite Linie in Diagrammen |
| Histogramm | `--ch-r` `--ch-g` `--ch-b` | Kanalkurven in der Karte Histogram; Bilddaten, keine Oberflächenfarben |
| Bildbereich | `--canvas` `--canvas-chrome` `--canvas-line` `--on-canvas` | Hintergrund um das Bild, Zoom-Leiste darauf |
| Alignpoints | `--ap-point` `--ap-ring` `--ap-shift` | rote Punkte nach Set Alignpoints, gelbe Ringe und grüne Striche nach Limit |
| Schrift | `--font` `--fs` `--fs-s` | Systemschrift, 13 px Grundgröße, 12 px für Nebentext |
| Form | `--r` `--r-s` `--r-l` `--h-btn` `--h-field` `--side` `--side-right` | Radien, Höhen, Panelbreiten |

Das Thema wechselt über `data-theme="light"` am `<html>`-Element. Ohne Attribut gilt dunkel.

Schrift ist die Systemschrift des Betriebssystems, keine eingebettete Schriftdatei. Zahlen stehen überall mit
`font-variant-numeric: tabular-nums`, damit Werte beim Ändern nicht springen. Gewichte: 400 für Text, 500 für
Schaltflächen, 600 für Gruppenüberschriften, aktive Reiter und gefüllte Schaltflächen.

## Fensteraufbau

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│ Werkzeugleiste   Select  MRU  Flat/Dark/Reference  Tools  Settings │ Cancel Pause │ About  CPUs │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Reiterleiste     1 Align › 2 Stack › 3 Wavelet                 Datei · Speicher   │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Aktionszeile     Schritt-Schaltflächen │ Häkchen in zwei Zeilen     Functions »   │
├─────────────┬─────────────────────────────────────────────────┬───────────────────┤
│ Panel links │ Bildbereich                                     │ Panel rechts      │
│ 272 px      │   schwebende Karten oben rechts                 │ 248 px            │
│             │   Zoom-Leiste unten mittig                      │ nur Wavelet       │
├─────────────┴─────────────────────────────────────────────────┴───────────────────┤
│ Frame-Leiste     Status · Goto Frame · Regler über alle Frames · Frame (n): n/N   │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Statusleiste     Fortschritt │ Aktion │ Ergebnis │ Position │ Zeit                │
└───────────────────────────────────────────────────────────────────────────────────┘
```

| Bereich | Höhe | Inhalt und Regeln |
|---|---|---|
| Werkzeugleiste | 44 px | Drei Zonen: links Select und die vier Menüs, in der Mitte Cancel und Pause, rechts Help, About und CPUs. Help gibt es im Original nicht, es öffnet die Doku in der App (RK-012) |
| Reiterleiste | 36 px | Reiter als nummerierte Schritte mit „›“ dazwischen. Rechts Dateiname, Codec, Größe, Frame-Zahl und Speicher in `--muted` |
| Aktionszeile | mind. 62 px | Links die Schritt-Schaltflächen, bündig über dem linken Panel. Daneben die Häkchen spaltenweise in zwei Zeilen, Spaltenfolge wie im Original |
| Arbeitsbereich | Rest | Linkes Panel fest 272 px, Bildbereich füllt den Rest, rechtes Panel 248 px nur im Wavelet-Reiter. Panels scrollen senkrecht, wenn das Fenster zu niedrig ist |
| Frame-Leiste | 40 px | Nur im Align-Reiter |
| Statusleiste | 28 px | Immer sichtbar |

Mindestgröße des Fensters: 1120×600. Darunter wird nichts umgebrochen oder ausgeblendet.

Eine eigene Titelleiste gibt es nicht, das Fenster nutzt die des Betriebssystems. Ihr Text nach dem Laden:
`SidereaPlanets — ‹Dateiname›`, während einer Berechnung zusätzlich Schritt und Prozent (`SidereaPlanets · Align 40%`),
damit der Fortschritt wie im Original in der Taskleiste lesbar ist.

## Komponenten

### Schaltflächen

| Art | Aussehen | Einsatz |
|---|---|---|
| Standard | `--raised`, Umriss `--line-strong`, Höhe `--h-btn`, Radius `--r` | Save image, Process, Load Scheme und alle übrigen |
| Gefüllt | `--accent` mit `--on-accent`, Gewicht 600 | Select; der nächste Schritt der Schrittfolge; Do All; Cancel während einer Berechnung |
| Ohne Rahmen | durchsichtig, Hover `--raised` | Menüs der Werkzeugleiste (mit Pfeil nach unten), Help (mit Icon), About |
| Textlink | `--accent-ink`, ohne Fläche | Aktionen in Gruppenköpfen: Reset Wavelets, Hold, Reset |
| Umschalter | gedrückt: `--accent-soft`, Umriss `--accent`, Text `--accent-ink` | Functions-Raster, Functions », Before |
| Gesperrt | Text `--off`, Umriss `--line`, keine Fläche | jede Art |

Icons sind Linienbilder auf 16×16 px, Strich 1,5 px mit runden Enden, Farbe `--muted` (Hover `--text`), ohne Füllung.
Sie stehen links vom Text. Bisher gibt es nur das Help-Icon, ein aufgeschlagenes Buch (Doku, nicht Info).
Das SVG steht im klickbaren Entwurf, dieselbe Geometrie in `figcraft-build.mjs`.

Pro Zeile ist höchstens eine Schaltfläche gefüllt. Rot kommt nicht vor: „Save image“ ist im Original rot
beschriftet, im Klon eine Standard-Schaltfläche.

### Schrittfolge

Set Alignpoints · Align · Limit (Align-Reiter) und Stack · Save image (Stack-Reiter) stehen als einzelne
Schaltflächen nebeneinander. Der nächste mögliche Schritt ist gefüllt, ein erledigter trägt einen Haken in `--ok`
vor dem Text, ein noch nicht möglicher ist gesperrt. Die genaue Zuordnung steht unter [Zustände](#zustände).

### Reiter

Text mit Nummer im Kreis. Aktiv: Text `--text`, Kreis `--accent`, 2 px Unterstrich in `--accent`. Gesperrt: Text
`--off`, nicht anklickbar, Tooltip nennt den fehlenden Schritt („Run Limit first“, „Run Stack first“).

### Häkchen und Auswahl

Native Checkboxen und Radiobuttons mit `accent-color: var(--accent)`, Beschriftung rechts daneben, Zeilenhöhe
23 px. Gesperrt: Beschriftung in `--off`. Radiogruppen stehen untereinander wie im Original; ihr Name steht als
Zwischenüberschrift (12 px, `--muted`) darüber, ohne Rahmen um die Gruppe.

### Zahlenfeld

Feld in `--field` mit rechtsbündigem Wert und zwei kleinen Pfeiltasten rechts. Höhe `--h-field`, Breite 66 px
(Ausnahmen: 84 px bei vierstelligen Werten, 46–56 px in engen Zeilen). Fokus färbt den Umriss in `--accent`.
Das Feld steht rechts in der Zeile, die Beschriftung links. Gesperrt: 45 % Deckkraft.

Dezimaltrenner ist der Punkt. Nachkommastellen: Denoise 2, Sharpen 3, Preview 1, Faktoren 1, Gamma 2, Verschiebung in RGB Align 2, Clip in
Histogram 1, alles andere ganzzahlig.

### Regler

Spur 4 px hoch in `--line-strong`, Griff 14 px rund in `--accent`. Die Spur wird links vom Griff nicht eingefärbt,
weil mehrere Regler keinen natürlichen Nullpunkt am linken Rand haben (Ebenen: −5.0 … 100.0, Brightness: −512 … 512).
Ein Regler ohne Bereich (Threshold vor Set Alignpoints) steht bei 45 % Deckkraft.

### Segmente

Zusammenhängende Schaltflächen in einem gemeinsamen Umriss. Zwei Einsätze:

- als Auswahl aus zwei Möglichkeiten: Waveletscheme (Dyadic (2^n) · Linear), Wavelet filter (Default · Gaussian).
  Gewählt: `--accent-soft` mit Text in `--accent-ink`.
- als Gruppe verwandter Aktionen: No Align · R.o.Interest · Scan Frames; Copy To · Load to · Difference.

### Menüs

Öffnen unter ihrer Schaltfläche, Fläche `--raised`, Radius `--r-l`, Schatten. Einträge 28 px hoch, Hover
`--accent-soft`. Ein Eintrag ist entweder eine Aktion, ein Häkchen, oder zeigt rechts seinen aktuellen Wert mit
„›“ für ein Untermenü. Zahlen- und Textfelder (Bias subtract, Save to Subdir, Faktor bei Create Darkframe) stehen
rechts im Eintrag. Trennlinien gruppieren wie im Original. Escape und ein Klick daneben schließen.

Dem Menü Tools sind zwei Einträge hinzugefügt (RK-009), getrennt durch eine Linie unter den Einträgen des Originals:
**Open project…** und **Save project…** (Projektdatei, `docs/formats.md`). Das Original hat sie nicht; Ort und Namen sind ein
Vorschlag, den Philipp ändern kann. Save project ist ohne geladene Aufnahme gesperrt.

**Save image** (Stack- und Wavelet-Reiter) ist eine Standard-Schaltfläche, die beim Anklicken ein Menü mit der Auswahl des
Dateityps öffnet (Stack: sieben Einträge, Wavelet: fünf; rechts steht die Endung). Ein Klick auf einen Eintrag öffnet den
Speichern-Dialog des Betriebssystems für genau diesen Typ. Beim Original wählt man den Typ im Dialog; das Dialog-Plugin meldet
den gewählten Filter nicht zurück, deshalb steht die Auswahl davor.

Aus dem Menü Settings entfallen „Check for the reference program Update“ sowie „Layouts“ und „Set Skin“. An die Stelle von „Set Skin“
tritt **Theme** (Dark · Light). Einträge mit „›“ öffnen ihr Untermenü als Flyout rechts neben dem Eintrag (bei Hover oder
Fokus), gleiche Fläche wie das Menü; darin stehen Häkchen, eine Auswahl (Processing Area, Stretch FIT/TIFF, Theme) oder
Zahlenfelder (Luminance Setting mit Reset Luminance).

### Panels und Gruppen

Panels haben keinen eigenen Kopf mit Minimieren/Maximieren. Eine Gruppe beginnt mit ihrer Überschrift (Gewicht 600)
und ist von der nächsten durch eine Linie in `--line` getrennt. Innenabstand des Panels 12 px. Aktionen, die im
Original im Panelkopf stehen (Reset Wavelets, Hold, Reset), stehen als Textlinks rechts in der Gruppenüberschrift.

### Schwebende Karten

Framelist, Registrationgraph und Alignmentpoints sind im Original eigene Fenster. Im Klon liegen sie als Karten
oben rechts über dem Bild, untereinander gestapelt, 236 px breit, mit Titel und Schließen-Kreuz. Sichtbar sind sie,
solange das zugehörige Häkchen gesetzt ist; das Kreuz nimmt das Häkchen heraus. Lange Listen scrollen in der Karte.
Die Karten sind nicht verschiebbar.

- Framelist: Check all · Check none · Empty List, darunter die Frames mit Häkchen, nach Align nach Qualität sortiert
  mit dem Wert rechts.
- Registrationgraph: Liniendiagramm, Quality in `--accent`, Difference in `--info`, Legende darunter. Vor Align steht
  ein Hinweis statt leerer Achsen.

### Bildbereich

Hintergrund `--canvas`, das Bild mittig mit 12 px Rand eingepasst, ein kaum sichtbarer Umriss zeigt die Bildgrenze.
Überlagerungen (Alignpoints, Processing Area als gestricheltes Rechteck in `--accent`) liegen deckungsgleich auf dem
Bild und zoomen mit.

Die Zoom-Leiste unten mittig ersetzt die Scrollbalken des Originals: − · aktueller Maßstab in Prozent der
Bildpixel (Klick setzt 100 %) · + · Fit, im Wavelet-Reiter zusätzlich der Umschalter **Before**, der das
unbearbeitete Summenbild zeigt. Mausrad zoomt, Ziehen verschiebt.

### Frame-Leiste

Von links: Statuspunkt mit Text („Loaded“ blass, „Aligned“ kräftig in `--ok`; er ersetzt das grüne Farbfeld des
Originals), Goto Frame als Zahlenfeld, der Regler über alle Frames in voller Restbreite, rechts „Frame (n): n/N“.

### Statusleiste

Fünf Felder, durch Linien getrennt, Schrift 12 px in `--muted`, Meldungen in `--text`:

1. Fortschrittsbalken (6 px hoch, `--accent`) mit Prozentzahl, 210 px breit
2. aktuelle Aktion, z. B. „Loading frame: 4“
3. Ergebnis, z. B. „Alignment finished.“
4. Position unter dem Mauszeiger im Wavelet-Reiter, z. B. „X=200 Y=201 Stack=10“
5. „Time used … · Time left …“

Die Meldungstexte sind die des Originals (`docs/workflow.md`).

## Zustände

Die Sperrlogik stammt aus `docs/workflow.md`; hier steht, wie sie aussieht.

| Zustand | Set Alignpoints | Align | Limit | Reiter Stack | Stack | Save image | Reiter Wavelet |
|---|---|---|---|---|---|---|---|
| Aufnahme geladen | gefüllt | gesperrt | gesperrt | gesperrt | | | gesperrt |
| Alignpoints gesetzt | aktiv, Haken | gefüllt | gesperrt | gesperrt | | | gesperrt |
| ausgerichtet | gesperrt, Haken | gesperrt, Haken | gefüllt | gesperrt | | | gesperrt |
| begrenzt | gesperrt, Haken | gesperrt, Haken | aktiv, Haken | offen | gefüllt | gesperrt | gesperrt |
| gestackt | gesperrt, Haken | gesperrt, Haken | aktiv, Haken | offen | aktiv, Haken | aktiv | offen |

Nach Limit wechselt die Oberfläche selbst in den Stack-Reiter.

**Während einer Berechnung** (Align, Stack, Process, Do All): alle Schritt-Schaltflächen sind gesperrt, Cancel ist
gefüllt, Pause ist aktiv und heißt nach dem Anhalten „Resume“. Der Balken in der Statusleiste läuft, Feld 2 nennt
den aktuellen Frame, Feld 5 die Zeiten. Außerhalb einer Berechnung sind Cancel und Pause gesperrt.

## Bildschirme

### Align

Linkes Panel in dieser Reihenfolge: Set Alignpoint parameters (mit Intensity selection, Threshold-Regler,
Number of Alignpoints) · Alignment setup · Limit Setup. Über dem Threshold-Regler steht links „weakest“, rechts
„strongest“ und in der Mitte „Threshold“, nach dem Setzen „Alignpoints: 82“.

| Zustand | Soll-Bild |
|---|---|
| Aufnahme geladen | ![Align, geladen](design/screenshots/01-align-loaded.png) |
| Alignpoints gesetzt | ![Align, Punkte gesetzt](design/screenshots/02-align-points.png) |
| Align läuft | ![Align, läuft](design/screenshots/03-align-running.png) |
| ausgerichtet, mit Framelist und Registrationgraph | ![Align, ausgerichtet](design/screenshots/04-align-aligned.png) |

### Stack

Linkes Panel: Stacking options · Create AVI. Resampling und Drizzling zeigen ihre Zusatzfelder (Methode und Faktor
bzw. Faktor) erst, wenn das Häkchen gesetzt ist, und zwar rechts neben dem Häkchen in derselben Zeile.

| Zustand | Soll-Bild |
|---|---|
| nach Limit, Show AlignSections mit der Karte Alignmentpoints | ![Stack, begrenzt](design/screenshots/05-stack-limited.png) |
| gestackt, Resampling und Drizzling eingeschaltet | ![Stack, gestackt](design/screenshots/06-stack-stacked.png) |

### Wavelet

Linkes Panel: Wavelets (Automatic, Hold Wavelet Setting, Waveletscheme, Initial Layer und Step Increment in einer
Zeile, Wavelet filter, Use Linked Wavelets, sechs Ebenen) · Available schemes mit Load Scheme und Save Scheme.

Eine Ebene belegt zwei Zeilen: oben Häkchen mit Ebenennummer, Denoise, Sharpen und die Preview-Schaltfläche mit dem
Reglerwert, darunter der Regler in voller Breite. Über der ersten Ebene stehen die Spaltenköpfe Layer · Denoise ·
Sharpen · Preview.

Rechtes Panel: Functions als Raster aus drei Spalten und fünf Zeilen in der Reihenfolge des Originals, jede
Schaltfläche ein Umschalter für ihr Panel · Contrast/Brightness mit zwei Reglern und zwei Zahlenfeldern · Copy To /
Load to / Difference mit der Auswahl Toggle. **Functions »** rechts in der Aktionszeile blendet das rechte Panel aus
und ein; der Bildbereich nimmt den frei werdenden Platz.

| Zustand | Soll-Bild |
|---|---|
| Summenbild, Standardwerte | ![Wavelet](design/screenshots/07-wavelet.png) |
| Ebenen 1–3 angehoben, Show Processing Area | ![Wavelet, geschärft](design/screenshots/08-wavelet-sharpened.png) |

### Karten hinter den Functions-Schaltflächen

Die Schaltflächen des Functions-Rasters öffnen Karten, die Form ist die der schwebenden Karten (Framelist, siehe
[Schwebende Karten](#schwebende-karten)): oben rechts über dem Bild, 236 px breit, Titel und Schließen-Kreuz, nicht
verschiebbar. Es gibt dazu kein Soll-Bild; die Karten sind aus den vorhandenen Komponenten gebaut.

Dieser Abschnitt legt die Karten fest, die RK-008 braucht: **Histogram**, **Gamma**, **RGB Align** und **RGB Balance**.
Contrast/Brightness bleibt das feste Panel unter dem Raster (Soll-Bild 07). Die übrigen Karten (Colour Mixing, View …,
Flip and Rotate, Resize Image, Denoise/Deringing, Wavelet Filter, Masking, Show Linegraph, Cropping Area) legen ihre
Tickets fest und ergänzen dieses Dokument.

**Gemeinsames Verhalten**

- Jede Functions-Schaltfläche ist ein Umschalter: gedrückt zeigt sie ihre Karte, ein zweiter Klick oder das Kreuz in der
  Karte schließt sie und löst die Schaltfläche. Mehrere Karten sind gleichzeitig offen und liegen in der Reihenfolge des
  Öffnens untereinander; reicht die Höhe nicht, scrollt der Kartenstapel.
- Die Karten stellen Werte ein, sie sind keine Schritte. Das Bild im Bildbereich folgt jeder Änderung sofort; die Werte
  gelten auch für das Ergebnis von Do All und für Save image. Eine geschlossene Karte behält ihre Werte, nur **Reset**
  nimmt sie zurück. Das Original wurde dazu nicht beobachtet (`docs/offene-fragen.md`).
- Die Werte werden immer auf das Ausgangsbild angewendet, nicht aufeinander gestapelt, in dieser Reihenfolge: RGB Align,
  RGB Balance, Histogram, Gamma, Contrast/Brightness. **Before** zeigt das Ausgangsbild ohne sie.
- **Reset** steht als Textlink in der Titelzeile, links vom Kreuz, und setzt die Werte der Karte auf den neutralen
  Zustand: das Bild bleibt unverändert. Weitere Aktionen (Auto) stehen ebenfalls als Textlinks dort.
- Bei einem Graustufenbild sind RGB Align und RGB Balance gesperrt; der Tooltip sagt „Needs a colour image“.
- Zahlenfeld und Regler einer Zeile sind gekoppelt: das eine setzt das andere. Werte außerhalb des Bereichs werden auf
  den Bereich gesetzt.

| Karte | Aufbau von oben nach unten | Standard · Bereich |
|---|---|---|
| Histogram | Graph, 212 × 84 px, Fläche `--field`, Umriss `--line`. Luminanz als gefüllte Fläche in `--line-strong`, darüber Red, Green und Blue als 1-px-Linien in `--ch-r` `--ch-g` `--ch-b`. Waagerecht 0 … 255, senkrecht logarithmisch, weil das Bild meist fast schwarz ist. Unter der Achse zwei Dreiecke für Black und White, ziehbar. Darunter Segmente **Input · Result**, dann die Felder **Black** und **White**, darunter **Auto** mit dem Feld **Clip %** | Black 0 · 0 … 254, White 255 · 1 … 255, Black bleibt unter White; Clip 0.1 · 0 … 49; Segment Input |
| Gamma | Regler mit „darker“ links und „brighter“ rechts (12 px, `--muted`) über der Spur, darunter das Feld **Gamma** rechts in der Zeile | 1.00 · 0.10 … 5.00; Regler in Schritten von 0.01 |
| RGB Align | Spaltenköpfe X und Y (12 px, `--muted`), drei Zeilen **Red**, **Green**, **Blue** mit je zwei Feldern, darunter die Zeile **Max shift** mit Feld, Hinweis „Auto moves Red and Blue onto Green“ (12 px, `--muted`) | Verschiebung 0.00 px · −20.00 … 20.00, Pfeiltasten ±0.1; positives X nach rechts, positives Y nach unten; Max shift 5 · 1 … 20 |
| RGB Balance | Drei Zeilen **Red**, **Green**, **Blue**: Beschriftung links, Regler, Feld rechts (56 px). Die Regler bleiben in der Akzentfarbe, die Kanäle erkennt man an der Beschriftung | 100 % · 0 … 200 |

- **Histogram**: *Input* zeigt das Bild vor dem Stretch, das ist das Bild, auf dem Black und White sitzen; *Result* zeigt
  das Ergebnis nach allen Schritten. **Auto** setzt Black und White so, dass vom dunkelsten und vom hellsten Ende je Clip %
  der Pixel abgeschnitten werden (gemessen an der Luminanz). Black und White wirken gleich auf alle Kanäle, die Farbe
  bleibt also erhalten.
- **Gamma**: über 1.00 wird das Bild in den Mitteltönen heller, darunter dunkler; Schwarz und Weiß bleiben.
- **RGB Align**: **Auto** sucht in den Kanälen Red und Blue die Verschiebung, mit der ihre Kanten auf die von Green fallen
  (Green ist die Bezugsfarbe), und trägt sie in die Felder ein. Danach lässt sich von Hand nachstellen. Die Verschiebung
  hat Nachkommastellen, das Bild behält seine Größe, am Rand wird der letzte Pixel wiederholt.
- **Contrast/Brightness** (festes Panel): 100 und 0 lassen das Bild unverändert. Contrast dreht die Tonwerte um Mittelgrau;
  der volle Brightness-Bereich −512 … 512 entspricht dem ganzen Tonumfang. Das Feld Contrast zeigt die Reglerposition
  geteilt durch 2,5 (250 → 100); dieses Verhältnis ist eine Annahme aus dem einen Wertepaar, nicht belegt.
- **Copy To / Load to / Difference** (festes Panel): *Copy To* merkt sich das Bild, das gerade im Bildbereich steht (mit
  allen Karten und Contrast/Brightness), als *Clipboard Image*. *Current Image* und *Clipboard Image* im Toggle wechseln
  die Anzeige zwischen beiden; *Clipboard Image* ist gesperrt, bis etwas kopiert wurde. *Load to* macht das kopierte Bild
  zum bearbeiteten Bild und setzt alle Karten und Contrast/Brightness zurück, weil sie im Bild schon stecken.
  *Difference* zeigt den Betrag des Unterschieds beider Bilder, vierfach verstärkt, und folgt dem aktuellen Bild;
  die Schaltfläche steht dabei gedrückt. *Load to* und *Difference* sind ohne Kopie gesperrt, *Difference* auch bei
  verschiedener Bildgröße (Meldung in der Statusleiste). Das Original wurde dazu nicht beobachtet; es gilt nur, was
  `docs/ui.md` über den Toggle sagt.
- Eine Functions-Schaltfläche ohne Karte bleibt unten und meldet „‹Name›: not available yet“ in der Statusleiste.

**Colour und LRGB** sitzen als Häkchen im Align- und im Stack-Reiter (`docs/ui.md`). *Colour* verarbeitet in Farbe, ohne
das Häkchen als Graustufen. *LRGB* ist nur mit *Colour* wählbar und schärft nur die Luminanz: Die Wavelets (RK-007)
rechnen auf der Luminanz, Farbton und Sättigung kommen aus dem Summenbild. Beide sind bei Graustufenbildern gesperrt.
Das Ticket nennt sie unter dem Wavelet-Reiter; dort stehen sie im Entwurf nicht, die Stelle ist also noch offen.
Umgesetzt (RK-008): Sie sind bei Graustufenaufnahmen gesperrt und LRGB zusätzlich ohne *Colour*. Umgesetzt (RK-007): Beide
Werte (`settings.colour`, `settings.lrgb`) wirken auf die Live-Vorschau und auf Do All (`colourMode` in `app/src/logic/wavelet.ts`,
Rechnung `wavelet::ColourMode` mit `color::with_luminance`, siehe `docs/wavelets.md`). Ohne *Colour* ist das Ergebnis Graustufen.

### Menüs

| Menü | Soll-Bild |
|---|---|
| Settings | ![Menü Settings](design/screenshots/09-menu-settings.png) |
| Flat/Dark/Reference | ![Menü Flat/Dark/Reference](design/screenshots/10-menu-flat-dark-reference.png) |

MRU zeigt die zuletzt geöffneten Dateien als Einträge, leer den Text „No recent files“ in `--muted`. Tools hat zwei
Einträge: eine Aktion und ein Häkchen.

### Doku

Help in der Werkzeugleiste schaltet zwischen Arbeitsansicht und Doku um. Der Schalter steht in der Doku gedrückt
(`--accent-soft`, Umriss `--accent`, Icon und Text `--accent-ink`); ein zweiter Klick und Escape führen zurück. Die
Arbeitsansicht bleibt dabei unverändert im Speicher (Aufnahme, Reglerwerte, laufende Berechnung). Reiterleiste,
Aktionszeile und Statusleiste entfallen in der Doku, die Werkzeugleiste bleibt.

Zwei Spalten: links die Seitenliste (Breite wie das linke Panel, 272 px, `--panel`, Linie rechts), rechts der Text auf
`--bg`, 720 px breit und waagerecht zentriert, 36 px Abstand oben.

| Teil | Regel |
|---|---|
| Suchfeld | oben in der Seitenliste, wie ein Textfeld (`--field`, Umriss `--line-strong`, Radius `--r-s`, 28 px hoch) |
| Gruppenüberschrift | 12 px, `--muted`; darunter die Seiten als Zeilen, 30 px hoch, Radius `--r` |
| Aktive Seite | `--accent-soft`, Text `--accent-ink`, Gewicht 600 |
| Brotkrumen | 12 px, `--muted`, über der Überschrift |
| Überschrift 1 | 28 px, Gewicht 600; darunter ein Einleitungsabsatz in 15 px, `--muted` |
| Überschrift 2 | 18 px, Gewicht 600 |
| Fließtext | 14 px, Zeilenhöhe 20 px, `--text` |
| Tabelle | Fläche `--panel`, Umriss `--line`, Radius `--r-l`; Kopfzeile `--raised` mit 12 px `--muted`; Zeilen 38 px, getrennt durch `--line`. Noch nicht Vorhandenes in `--off` |
| Hinweis | `--accent-soft`, 3 px Streifen links in `--accent`, Radius `--r` |
| Schritte | Nummer in einem Kreis (20 px, `--raised`, Ziffer 11 px `--muted`), daneben der Text |
| Blättern | unten links „‹ Vorherige“, rechts „Nächste ›“, beides `--accent-ink` |

Doku-Texte sind Anwenderdoku und auf Englisch wie die Oberfläche; sie nennen, was schon geht, und markieren Fehlendes
als „Not available yet“. Quelle der Seiten ist Markdown im Repo (RK-012).

| Zustand | Soll-Bild |
|---|---|
| Doku, dunkel | ![Doku](design/screenshots/13-docs.png) |
| Doku, hell | ![Doku, hell](design/screenshots/14-docs-light.png) |

Die Soll-Bilder 13 und 14 stammen aus `docs/design/siderea-planets.figcraft`, nicht aus dem klickbaren Entwurf; der
Entwurf zeigt die Doku nicht.

### Helles Thema

| Reiter | Soll-Bild |
|---|---|
| Align | ![Align, hell](design/screenshots/11-align-light.png) |
| Wavelet | ![Wavelet, hell](design/screenshots/12-wavelet-light.png) |

## Bewusste Abweichungen vom Original

| Original | Klon | Grund |
|---|---|---|
| Skin „Office 2007 Silver“, hell | neutral dunkel, hell umschaltbar | RK-010; Bildbeurteilung |
| Framelist, Registrationgraph, Alignmentpoints als eigene Fenster | Karten über dem Bild | ein Fenster, nichts geht hinter anderen verloren |
| Panelköpfe mit Minimieren/Maximieren | Gruppenüberschriften | Panels sind fest angeordnet |
| Scrollbalken am Bild | Zoom-Leiste, Mausrad, Ziehen | RK-010 verlangt Zoom, Pan, Vorher/Nachher |
| grünes Farbfeld links unten | Statuspunkt mit Text | Bedeutung des Farbfelds ist ungeklärt, Text sagt sie |
| „File Version“ und Datum in der Reiterleiste | Dateiname, Codec, Größe, Frame-Zahl | Version gehört in About |
| „Save image“ rot beschriftet | Standard-Schaltfläche | Rot bedeutet Gefahr |
| Radiogruppen bei zwei Möglichkeiten | Segmente | weniger Höhe, gleiche Stelle |
| „Intensity_selection“, „Realign_with Processed“ | ohne Unterstrich | Artefakte der Delphi-Beschriftung |
| „Step Increment“ | „Step Incr.“ mit vollem Namen im Tooltip | Platz neben Initial Layer |
| Dezimalkomma je nach Windows-Einstellung | Punkt | einheitlich auf allen Plattformen |
| » klappt das Functions-Panel | „Functions »“ | Beschriftung sagt, was geklappt wird |

## Was im Entwurf nur Platzhalter ist

- Das Jupiter-Bild ist gezeichnet. Im Klon steht dort das Frame bzw. das Summenbild.
- Die Schärfung beim Bewegen der Ebenen-Regler ist ein einfacher Unscharf-Filter, nicht der Wavelet-Algorithmus.
- Qualitätswerte, Punktpositionen, Speicherangaben und Zeiten sind erfunden.
- Die Functions-Schaltflächen schalten nur ihren gedrückten Zustand um.

## Noch nicht festgelegt

Für diese Teile gibt es kein Soll-Bild. Wer sie braucht, baut sie aus den vorhandenen Komponenten und ergänzt
dieses Dokument im selben Commit.

- Zustand vor dem Laden (festgelegt in RK-010): Select ist gefüllt, Schritt-Schaltflächen und linkes Panel sind
  gesperrt, die Frame-Leiste ist leer. Der Bildbereich zeigt mittig „Press Select to open a recording“ in `--muted`.
- Fehlermeldungen (festgelegt in RK-010): Text mit Präfix „Error:“ im Ergebnisfeld der Statusleiste (Feld 3), in
  `--text`; kein Dialog, kein Rot.
- Speicherangabe („Memory used/free/total“) in der Reiterleiste: entfällt vorerst, sie braucht einen Befehl im Backend.
- Elf der fünfzehn Panels hinter den Functions-Schaltflächen. Histogram, Gamma, RGB Align und RGB Balance stehen unter
  [Karten hinter den Functions-Schaltflächen](#karten-hinter-den-functions-schaltflächen); die übrigen sind im Original
  noch nicht erfasst (`docs/offene-fragen.md`). Vorgabe für die Form: schwebende Karten wie Framelist.
- Stackgraph, Stacklist, Aligndata und Prefilter. Vorgabe für die Form: schwebende Karten.
- MACRO/Batch-Fenster und About-Dialog. Ihre Schaltflächen sind angelegt und melden „not available yet“.
- Dateidialoge sind die des Betriebssystems und brauchen kein Design.

## Zuordnung zu den Tickets

| Ticket | Betroffene Teile der Oberfläche | Soll-Bilder |
|---|---|---|
| RK-004 Eingabe | Select, MRU, Angaben zur Datei in der Reiterleiste, Frame-Leiste | 01 |
| RK-005 Ausrichtung | Align-Reiter komplett, Framelist, Registrationgraph, Fortschritt | 01–04, 11 |
| RK-006 Stacken | Stack-Reiter komplett, Karte Alignmentpoints | 05, 06 |
| RK-007 Wavelets | linkes Panel im Wavelet-Reiter, Process, Do All, Before, Processing Area | 07, 08, 12 |
| RK-008 Farbe und Tonwerte | rechtes Panel im Wavelet-Reiter (Functions, Contrast/Brightness), Colour/LRGB | 07, 12 |
| RK-009 Speichern | Save image in Stack und Wavelet, Save Aligned, Save Integrated, Load/Save Scheme | 06, 07 |
| RK-010 Oberfläche | alles in diesem Dokument außer der Doku | 01–12 |
| RK-012 Doku in der App | Help in der Werkzeugleiste, Doku-Ansicht | 13, 14 |

## Zustände im Klon ansehen (Dev-Build)

Im Dev-Build (`pnpm tauri dev` oder `pnpm exec vite`) springt `app/src/preview.ts` mit denselben URL-Parametern wie der
Entwurf in einen Zustand, mit erzeugten Daten statt einer Aufnahme: `step`, `tab`, `theme=light`, `busy`, `show`
(framelist, graph, resampling, drizzling, procarea, alignpoints, nosections), `layers`. Im Release-Build fehlt das Modul.

## Soll-Bilder neu erzeugen

Der Entwurf springt über URL-Parameter in einen Zustand: `step` (loaded, points, aligned, limited, stacked),
`tab` (align, stack, wavelet), `theme=light`, `show` (framelist, graph, resampling, drizzling, procarea,
alignpoints, nosections), `menu` (mru, flat, tools, settings), `layers` (sechs Reglerwerte), `busy` (Prozent),
`chrome=0` (blendet die Leiste des Entwurfs aus). Die Bilder entstehen mit einem Browser ohne Fenster:

```
msedge --headless=new --window-size=1440,900 --hide-scrollbars \
  --screenshot=docs/design/screenshots/04-align-aligned.png \
  "file:///…/docs/design/oberflaeche-entwurf.html?chrome=0&step=aligned&show=framelist,graph"
```

| Bild | Parameter |
|---|---|
| 01-align-loaded | `step=loaded` |
| 02-align-points | `step=points` |
| 03-align-running | `step=points&busy=40` |
| 04-align-aligned | `step=aligned&show=framelist,graph` |
| 05-stack-limited | `step=limited&tab=stack` |
| 06-stack-stacked | `tab=stack&show=resampling,drizzling,nosections` |
| 07-wavelet | `tab=wavelet` |
| 08-wavelet-sharpened | `tab=wavelet&layers=32,22,14,10,10,10&show=procarea` |
| 09-menu-settings | `step=loaded&menu=settings` |
| 10-menu-flat-dark-reference | `step=loaded&menu=flat` |
| 11-align-light | `step=aligned&show=framelist,graph&theme=light` |
| 12-wavelet-light | `tab=wavelet&theme=light` |
