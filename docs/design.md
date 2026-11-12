# Design der Oberfläche

Verbindliche Vorgabe für alles, was im Klon sichtbar ist. Umgesetzt wird sie im Frontend unter `app/src` (RK-010) und in der
Mac-App unter `mac/App` (MAC-005, SwiftUI; ihre bewussten Abweichungen stehen in [Mac-App](#mac-app-bewusste-abweichungen));
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
| `mac/App/Design/Tokens.swift` | Dieselben Werte als Swift-Farben und -Maße für die Mac-App, je Wert mit hellem und dunklem Zweig. Ein Wert ändert sich zuerst in `tokens.css`, dann hier |
| `docs/design/oberflaeche-entwurf.html` | Klickbarer Entwurf aller sieben Schritte, im Browser öffnen |
| `docs/design/screenshots/` | Soll-Bilder der Zustände bei 1440×900, erzeugt aus dem Entwurf |
| `docs/design/siderea-planets.figcraft` | Die sieben Schritte (Open, Alignpoints, Align, Limit, Stack, Wavelets, Save) als FigCraft-Datei, je dunkel und hell, dazu Limit mit geöffnetem Menü Show, Stack im erledigten Zustand und das Help-Fenster: 20 Frames, 18 davon 1440×900, die zwei Help-Fenster 1000×720. Farben sind Variablen der Collection „Theme“ mit den Modi Dark und Light |
| `docs/design/vorschlag-gefuehrter-ablauf.html` | Überholt: die drei Vorschläge, aus denen die Schrittleiste (RK-014) gewählt wurde |
| `docs/design/figcraft-build.mjs` | Baut die FigCraft-Datei neu und liest die Farben dabei aus `tokens.css`. Aufruf: `node docs/design/figcraft-build.mjs` |

Alle Frames, auch die zwei Help-Fenster (RK-012, siehe [Doku](#doku)), entstehen aus dem Skript. Ein Lauf schreibt die
Datei neu und verliert nichts; Änderungen gehören deshalb ins Skript, nicht von Hand in die Datei.

Der Entwurf ist eine Vorlage zum Ansehen, kein Code zum Übernehmen: Er ist eine einzelne HTML-Datei ohne Vue und
ohne Rechenkern. Markup und CSS-Regeln dürfen als Orientierung dienen, die Umsetzung baut eigene Komponenten.

## Grundsätze

1. **Gleicher Name.** Jedes Bedienelement trägt dieselbe englische Beschriftung wie in the reference program. Wo es sitzt,
   bestimmt der Ablauf (RK-014): Der Klon führt in sieben Schritten durch die Bedienung, statt sie wie das Original auf
   Reiter, Aktionszeile und Panel zu verteilen. Wer das Original kennt, findet jedes Element unter seinem Namen im
   Schritt, zu dem es gehört.
2. **Das Bild ist die Hauptsache.** Die Oberfläche ist neutral grau ohne Farbstich, damit sie die Farben des Bildes
   nicht verfälscht. Der Bildbereich bleibt in beiden Themen dunkel.
3. **Eine Akzentfarbe.** Bernstein, angelehnt an die orangen Schaltflächen des Originals. Sie markiert den nächsten
   Schritt, gewählte Zustände und Fortschritt, sonst nichts.
4. **Der nächste Schritt ist sichtbar.** Außerhalb einer Berechnung ist im ganzen Fenster genau eine Schaltfläche
   gefüllt: die Aktion des offenen Schritts unten im linken Panel. Erledigte Schritte tragen einen Haken, noch nicht
   mögliche sind gesperrt.
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
Schaltflächen, 600 für Gruppenüberschriften, die angezeigte Pille der Schrittleiste und gefüllte Schaltflächen.

## Fensteraufbau

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│ Werkzeugleiste   Select  MRU  Flat/Dark/Reference  Tools  Settings  Open  Save │ Cancel Pause │ Help About  CPUs │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Schrittleiste  ①Open ─ ②Alignpoints ─ ③Align ─ ④Limit ─ ⑤Stack ─ ⑥Wavelets ─ ⑦Save   Datei │
├─────────────┬─────────────────────────────────────────────────┬───────────────────┤
│ Panel links │ Bildbereich   Bildname          Functions » Show ▾ │ Panel rechts      │
│ Kopf        │   schwebende Karten oben rechts                 │ 248 px            │
│ Felder      │                                                 │ nur Wavelets      │
│ Fuß: Aktion │   Zoom-Leiste unten mittig                      │                   │
├─────────────┴─────────────────────────────────────────────────┴───────────────────┤
│ Frame-Leiste     Status · Goto Frame · Regler über alle Frames · Frame (n): n/N   │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Statusleiste     Fortschritt │ Aktion │ Ergebnis │ Position │ Zeit                │
└───────────────────────────────────────────────────────────────────────────────────┘
```

| Bereich | Höhe | Inhalt und Regeln |
|---|---|---|
| Werkzeugleiste | 44 px | Drei Zonen: links Select, die vier Menüs und die Projekt-Schaltflächen Open und Save, in der Mitte (im Rest zwischen links und rechts zentriert) Cancel und Pause, rechts Help, About und CPUs (die Zahl der Threads aller Berechnungen: Standard alle logischen Kerne, Bereich 1 bis dahin; das Feld klemmt Eingaben darauf). Help, Open und Save gibt es im Original nicht: Help öffnet die Doku in der App (RK-012), Open und Save öffnen und speichern die Projektdatei (RK-009) |
| Schrittleiste | 40 px | Die sieben Schritte als Pillen (siehe [Schrittleiste](#schrittleiste)). Rechts Dateiname, Codec, Größe und Frame-Zahl in `--muted`, wird bei Platzmangel mit „…“ gekürzt, die Pillen nie |
| Arbeitsbereich | Rest | Linkes Panel fest 272 px mit dem offenen Schritt (siehe [Schrittpanel](#schrittpanel)), Bildbereich füllt den Rest, rechtes Panel 248 px nur im Schritt Wavelets. Panels scrollen senkrecht, wenn das Fenster zu niedrig ist; der Fuß des linken Panels mit der Aktion bleibt stehen |
| Frame-Leiste | 40 px | Nur in den Schritten Alignpoints, Align und Limit |
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
| Gefüllt | `--accent` mit `--on-accent`, Gewicht 600 | die Aktion des offenen Schritts im Fuß des Schrittpanels (Select, Set Alignpoints, Align, Limit, Stack, Do All, Save image) oder „Continue: ‹Schritt› ›“; Cancel während einer Berechnung |
| Ohne Rahmen | durchsichtig, Hover `--raised` | Select in der Werkzeugleiste, Menüs der Werkzeugleiste (mit Pfeil nach unten), Open und Save (mit Icon), Help (mit Icon), About, Show am Bild |
| Textlink | `--accent-ink`, ohne Fläche | Aktionen in Gruppenköpfen: Reset Wavelets, Hold, Reset |
| Umschalter | gedrückt: `--accent-soft`, Umriss `--accent`, Text `--accent-ink` | Functions-Raster, Functions », Before |
| Gesperrt | Text `--off`, Umriss `--line`, keine Fläche | jede Art |

Icons sind Linienbilder auf 16×16 px, Strich 1,5 px mit runden Enden, Farbe `--muted` (Hover `--text`), ohne Füllung.
Sie stehen links vom Text. Es gibt drei: Help, ein aufgeschlagenes Buch (Doku, nicht Info); Open, eine Mappe; Save, eine
Diskette. Das Help-SVG steht im klickbaren Entwurf, dieselbe Geometrie in `figcraft-build.mjs`; Open und Save (RK-009) stehen
nur in `app/src/components/Toolbar.vue` (Pfade `M1.5 3.5h4l1.5 1.5h7.5v7.5h-13z` und `M2.5 2.5h8l3 3v8h-11zM5 2.5v3.5h5V2.5M5 13.5v-4h6v4`).

Pro Zeile ist höchstens eine Schaltfläche gefüllt. Rot kommt nicht vor: „Save image“ ist im Original rot
beschriftet, im Klon eine Standard-Schaltfläche.

### Schrittleiste

Eine Zeile (40 px, `--panel`, Linie unten in `--line`, Innenabstand 12 px) mit den sieben Schritten **Open ·
Alignpoints · Align · Limit · Stack · Wavelets · Save**. Jeder Schritt ist eine Pille (Höhe 28 px, Radius 14 px,
Innenabstand 4 px links und 10 px rechts, Text 13 px) mit seiner Nummer in einem Kreis (18 px, Ziffer 11 px, Gewicht
600). Zwischen zwei Pillen steht eine waagerechte Linie (14 px breit, 1 px, `--line-strong`; nach einem erledigten
Schritt `--ok`).

| Zustand der Pille | Aussehen | Verhalten |
|---|---|---|
| angezeigt (der Schritt im Panel) | Fläche `--accent-soft`, Text `--accent-ink`, Gewicht 600, Kreis `--accent` mit Ziffer `--on-accent` | — |
| erledigt | ohne Fläche, Text `--text`, Kreis `--ok` mit Haken statt Ziffer | anklickbar, öffnet den Schritt; Tooltip nennt Schritt und Ergebnis („Alignpoints · 82 set“) |
| möglich, noch nicht erledigt | ohne Fläche, Text `--text`, Kreis mit Umriss `--line-strong` und Ziffer `--muted` | anklickbar |
| noch nicht möglich | Text `--off`, Kreis mit Umriss `--line`, Ziffer `--off` | nicht anklickbar, Tooltip nennt die Aktion des nächsten offenen Schritts („Run Align first“), ohne Aufnahme „Open a recording first“ |

Hover auf einer anklickbaren Pille: Fläche `--raised-h`. Erledigt und angezeigt zugleich: Fläche wie angezeigt, Kreis in
`--ok` mit Haken. Während einer Berechnung ist keine Pille anklickbar; ihr Aussehen bleibt. Die Leiste passt bei der
Mindestbreite von 1120 px ohne Umbruch und ohne Kürzung der Pillen hinein; gekürzt wird nur die Dateiangabe rechts.

Welcher Schritt erledigt, möglich oder gesperrt ist, steht unter [Zustände](#zustände).

### Schrittpanel

Das linke Panel zeigt nur den angezeigten Schritt, von oben nach unten:

1. **Ergebniszeile des vorigen Schritts**, nur wenn der angezeigte Schritt der nächste offene ist: Haken und Text in
   `--ok`, 12 px, z. B. „✓ Alignpoints · 82 set“.
2. **Kopf**: „Step n of 7“ (12 px, `--muted`), darunter der Name des Schritts (15 px, Gewicht 600), darunter ein Satz,
   was der Schritt tut (12 px, `--muted`, Zeilenhöhe 17 px). Die Sätze stehen unter [Bildschirme](#bildschirme).
3. **Wichtige Einstellungen** des Schritts, gebaut aus den üblichen Komponenten (Zeilen, Zahlenfelder, Regler,
   Häkchen, Segmente), ohne eigene Gruppenüberschrift, wenn es nur eine Gruppe ist.
4. **Advanced**: eine Zeile „▸ Advanced“ (Gewicht 500) mit „n more settings“ rechts in `--muted`; ein Klick klappt die
   übrigen Einstellungen des Schritts auf („▾ Advanced“). Dort stehen sie vollständig, mit ihren Gruppenüberschriften
   aus dem Original. Kein Element aus `docs/ui.md` geht verloren. Aufgeklappt bleibt aufgeklappt, bis man es zuklappt
   (je Schritt im Browser-Speicher der App gemerkt, auch nach einem Neustart; nicht in der Projektdatei).
   Das Häkchen **Settings > Show Advanced Controls** klappt beim Setzen die Advanced-Bereiche aller Schritte auf; danach
   lässt sich jeder Bereich einzeln zu- und wieder aufklappen. Entfernt man das Häkchen, bleiben die Bereiche, wie sie sind.
5. **Fuß** (unten im Panel, scrollt nicht mit, Linie oben in `--line`, Innenabstand 12 px):
   - offener Schritt: die Aktion als gefüllte Schaltfläche in voller Breite, Beschriftung des Originals; darunter
     „Then: ‹nächster Schritt›“ (12 px, `--muted`). Weitere Aktionen des Schritts (Process im Schritt Wavelets) stehen
     als Standard-Schaltflächen in derselben Zeile links daneben.
   - erledigter, wieder geöffneter Schritt: oben das Ergebnis in `--ok`, darunter „‹Aktion› again“ als
     Standard-Schaltfläche in voller Breite, darunter in 12 px `--muted` „Running it again discards the steps after
     it.“ (nur, wenn es Schritte danach gibt, die etwas verwerfen würden), zuletzt gefüllt „Continue: ‹nächster offener
     Schritt› ›“.
   - noch nicht möglicher Schritt kommt nicht vor, er lässt sich nicht öffnen.

Während einer Berechnung ist die Aktion im Fuß gesperrt; gefüllt ist dann nur Cancel in der Werkzeugleiste.

Nach jedem erfolgreichen Lauf öffnet sich der nächste Schritt von selbst (nach Select also Alignpoints, nach Do All
Save). Nach Save image bleibt Save offen und zeigt das Ergebnis; „Continue“ entfällt dort, an seiner Stelle steht
gefüllt „Open another recording“ (öffnet wie Select).

Die Ergebnistexte:

| Schritt | Ergebnis (Ergebniszeile, Tooltip der Pille) |
|---|---|
| Open | ‹Dateiname› · ‹N› frames |
| Alignpoints | ‹n› set |
| Align | best quality ‹q› % |
| Limit | ‹k› of ‹N› kept |
| Stack | ‹n› frames · ‹B›×‹H› |
| Wavelets | applied to the whole image |
| Save | ‹Dateiname› |

### Show am Bild

Die Häkchen der früheren Aktionszeile (Show Full Image, Show ROI, Show Alignpoints, Show Aligndata, Show Framelist,
Show Registrationgraph, Show Prefilter; Show AlignSections, Show Stackgraph, Show Stacklist; Show AlignPoints, Show
Processing Area) liegen in einem Menü **Show ▾** oben rechts im Bildbereich, 10 px vom Rand, als Schaltfläche ohne
Rahmen auf `--canvas-chrome` mit Text `--on-canvas`. Das Menü enthält die Häkchen, die das Original im zugehörigen
Reiter hat: in Alignpoints, Align und Limit die des Align-Reiters, in Stack die des Stack-Reiters, in Wavelets und Save
die des Wavelet-Reiters. Im Schritt Open gibt es kein Show. Die Häkchen behalten ihre Wirkung, die schwebenden Karten
verhalten sich wie bisher und beginnen unter der Zeile mit Show.

Links oben im Bildbereich steht in derselben Art, aber nicht anklickbar, was das Bild zeigt: „Frame ‹n› of ‹N›“,
„Stack of ‹n› frames“ oder „Sharpened stack“. Im Schritt Wavelets steht links neben Show der Umschalter **Functions »**
(gleiche Form, gedrückt mit Umriss und Text in `--accent`).

Text auf dem Bildbereich steht in `--on-canvas` (Hinweise wie „Press Select to open a recording“ mit 60 % Deckkraft),
nie in `--muted` oder `--accent-ink`: Der Bildbereich ist in beiden Themen dunkel, diese Töne wären im hellen Thema
unlesbar. Ohne geladene Aufnahme fehlen Bildname, Show und Zoom-Leiste; es steht nur der Hinweis.

### Häkchen und Auswahl

Native Checkboxen und Radiobuttons mit `accent-color: var(--accent)`, Beschriftung rechts daneben, Zeilenhöhe
23 px. Gesperrt: Beschriftung in `--off`. Radiogruppen stehen untereinander wie im Original; ihr Name steht als
Zwischenüberschrift (12 px, `--muted`) darüber, ohne Rahmen um die Gruppe.

### Zahlenfeld

Feld in `--field` mit rechtsbündigem Wert und zwei kleinen Pfeiltasten rechts. Höhe `--h-field`, Breite 66 px
(Ausnahmen: 84 px bei vierstelligen Werten, 46–56 px in engen Zeilen). Fokus färbt den Umriss in `--accent`.
Das Feld steht rechts in der Zeile, die Beschriftung links. Gesperrt: 45 % Deckkraft und Wert in `--off`, auch
außerhalb einer Zeile (Limit Setup) und wenn nur das Panel darum gesperrt ist.

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

**Open** und **Save** (RK-009) sind eigene Schaltflächen ohne Rahmen in der Werkzeugleiste, rechts neben dem Menü Settings
(Projektdatei, `docs/formats.md`). Das Original hat sie nicht. Die Beschriftung ist kurz gehalten, weil die Werkzeugleiste
bei der Mindestbreite von 1120 px sonst überläuft; der Tooltip und das Label für Screenreader heißen *Open project* und
*Save project*. Beide sind während einer Berechnung gesperrt, Save zusätzlich ohne geladene Aufnahme. Tools enthält damit
wieder nur die Einträge des Originals.

**Save image** gibt es zweimal: gefüllt als Aktion des Schritts Save (das geschärfte Bild, fünf Dateitypen) und als
Standard-Schaltfläche im erledigten Schritt Stack unter dem Ergebnis (das Summenbild, sieben Dateitypen). Beide tragen den Pfeil nach unten und öffnen
beim Anklicken ein Menü mit der Auswahl des Dateityps (rechts steht die Endung). Ein Klick auf einen Eintrag öffnet den
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

Scrollbalken (Panels, Seitenliste der Doku, Text) sind schmal: 10 px breit, ohne Pfeilknöpfe, Spur durchsichtig, Griff in
`--line-strong` mit Radius 5 px, beim Überfahren `--off`. Der Standardbalken von Windows kommt nirgends vor; er würde
außerdem 17 px der 272 px des Panels kosten und Beschriftungen wie „Minimum distance between“ umbrechen.

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
Bildpixel (Klick setzt 100 %) · + · Fit, in den Schritten Wavelets und Save zusätzlich der Umschalter **Before**, der das
unbearbeitete Summenbild zeigt. Mausrad zoomt, Ziehen verschiebt. Das Bild folgt der Größe der Fläche, bis der Nutzer zoomt oder
verschiebt (ein Klick ohne Bewegung zählt nicht); bei jedem Schrittwechsel wird es neu eingepasst, weil sich die Fläche mit
Frame-Leiste und rechtem Panel ändert (RK-010-Prüflauf: Zoom blieb nach Wavelets bei 73 %).

### Frame-Leiste

Von links: Statuspunkt mit Text („Loaded“ blass, „Aligned“ kräftig in `--ok`; er ersetzt das grüne Farbfeld des
Originals), Goto Frame als Zahlenfeld, der Regler über alle Frames in voller Restbreite, rechts „Frame (n): n/N“.

### Statusleiste

Fünf Felder, durch Linien getrennt, Schrift 12 px in `--muted`, Meldungen in `--text`:

1. Fortschrittsbalken (6 px hoch, `--accent`) mit Prozentzahl, 210 px breit
2. aktuelle Aktion, z. B. „Loading frame: 4“
3. Ergebnis, z. B. „Alignment finished.“
4. Position unter dem Mauszeiger in den Schritten Wavelets und Save, z. B. „X=200 Y=201 Stack=10“
5. „Time used … · Time left …“

Die Meldungstexte sind die des Originals (`docs/workflow.md`).

## Zustände

Die Sperrlogik stammt aus `docs/workflow.md`; hier steht, wie sie aussieht.

Pillen der Schrittleiste je Zustand (✓ erledigt, ● nächster offener Schritt, ○ möglich, – gesperrt). Angezeigt ist
nach jedem Lauf der nächste offene Schritt (●); jede Pille mit ✓ oder ○ lässt sich öffnen.

| Zustand | Open | Alignpoints | Align | Limit | Stack | Wavelets | Save |
|---|---|---|---|---|---|---|---|
| leer | ● | – | – | – | – | – | – |
| Aufnahme geladen | ✓ | ● | – | – | – | – | – |
| Alignpoints gesetzt | ✓ | ✓ | ● | – | – | – | – |
| ausgerichtet | ✓ | ✓ | ✓ | ● | – | – | – |
| begrenzt | ✓ | ✓ | ✓ | ✓ | ● | – | – |
| gestackt | ✓ | ✓ | ✓ | ✓ | ✓ | ● | ○ |
| geschärft (Do All gelaufen) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ● |
| gespeichert | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

- Save ist schon nach Stack möglich, weil Save image ein noch nicht angewendetes Do All selbst nachholt (RK-009).
- Wavelets gilt als erledigt, solange das Ergebnis von Do All zu den aktuellen Wavelet-Werten passt; ändert man danach
  eine Ebene oder eine Einstellung des Schritts, verliert die Pille den Haken und Wavelets ist wieder der nächste
  offene Schritt. Save verliert dabei ebenfalls den Haken.
- Open bleibt immer anklickbar; Select dort (oder in der Werkzeugleiste) öffnet eine neue Aufnahme und setzt alles auf
  „Aufnahme geladen“.

**Erneutes Ausrichten** (Entscheidung von Philipp, RK-005): Set Alignpoints und Align bleiben nach ihrem ersten Lauf
bedienbar, auch nach Limit, Stack und Wavelet. Im Original sind beide ab Align gesperrt (`docs/workflow.md`,
Abschnitt 2.2); der Klon weicht bewusst davon ab, damit man andere Punkte, eine andere Region oder eine andere
Boxgröße ausprobieren kann, ohne die Aufnahme neu zu öffnen. Ein neuer Lauf verwirft alles, was auf der alten
Ausrichtung aufbaut: Qualität und Frame-Auswahl (alle Frames wieder angehakt), Verschiebungsstriche der Punkte, Stack,
geschärftes Bild und Farbanpassung, in der Oberfläche wie im Rechenkern (`StackState`, `WaveletState`, `ColorState`,
`ExportState`, `AlignState`). Der Zustand fällt auf „Alignpoints gesetzt“ zurück, die Pillen ab Limit sind wieder
gesperrt und nach dem Lauf öffnet sich der nächste offene Schritt (nach Set Alignpoints Align, nach Align Limit). Im
wieder geöffneten Schritt steht der Hinweis „Running it again discards the steps after it.“ unter „‹Aktion› again“.
Align verwirft beim Start, nicht erst am Ende: Bricht man
ab, bleibt kein altes Ergebnis stehen, und Save image kann kein Bild des alten Stacks schreiben. Auch ein erneutes
Limit nach Stack verwirft den Stack. Regler, Wavelet-Einstellungen und das Bild von Copy To bleiben erhalten.

**Während einer Berechnung** (Align, Stack, Process, Do All): die Aktion im Fuß des Schrittpanels ist gesperrt, keine
Pille ist anklickbar, Cancel ist
gefüllt, Pause ist aktiv und heißt nach dem Anhalten „Resume“. Der Balken in der Statusleiste läuft, Feld 2 nennt
den aktuellen Frame, Feld 5 die Zeiten. Außerhalb einer Berechnung sind Cancel und Pause gesperrt und der Balken steht auf
0 % (auch nach einem Abbruch oder Fehler; die Soll-Bilder der fertigen Schritte zeigen noch 100 %, RK-010-Prüflauf). Das Ergebnis
von Limit steht im Ergebnisfeld als „Stacksize: 8  Low quality: 87.20%“ (Qualität des schwächsten behaltenen Frames).

## Bildschirme

Je Schritt: der Satz im Kopf (Englisch wie die Oberfläche; Philipp nimmt die Sätze noch ab, RK-014), die wichtigen
Einstellungen, was unter Advanced liegt, und die Aktion im Fuß. Die Gruppen und Felder sind die der früheren Reiter
(`docs/ui.md`); die Einteilung in wichtig und Advanced ist die Vorgabe dieses Dokuments.

### 1 Open

*„Choose a recording: a SER or AVI video or a series of images.“*

- Wichtig: **Recent**, die zuletzt geöffneten Dateien als Zeilen (Name, darunter der Ordner in 12 px `--muted`), ein
  Klick öffnet die Datei; leer „No recent files“ in `--muted`. Nach dem Laden steht darüber die geladene Datei mit
  Codec, Größe und Frame-Zahl.
- Advanced: entfällt.
- Aktion: **Select**. Select und MRU in der Werkzeugleiste bleiben und tun dasselbe.
- Bildbereich vor dem Laden: mittig „Press Select to open a recording“ in `--muted`.

### 2 Alignpoints

*„Marks the details the frames are lined up on. A lower threshold gives more points.“*

- Wichtig: der Threshold-Regler mit „weakest“ links, „strongest“ rechts und „Threshold“ in der Mitte, nach dem Setzen
  „Alignpoints: 82“; Minimum distance between; Min distance from edge; **R.o.Interest** als Umschalter in voller Breite
  (aus dem Segment der Gruppe Alignment setup hierher gezogen, weil Set Alignpoints die Region braucht; zeichnen lässt
  sie sich wie bisher vor und nach Set Alignpoints, bis Align gelaufen ist).
- Advanced: der Rest der Gruppe Set Alignpoint parameters (Intensity selection mit Lo/Hi, Number of Alignpoints, Keep
  Alignpoints inside ScanFrame).
- Aktion: **Set Alignpoints**. Frame-Leiste sichtbar.

### 3 Align

*„Lines up every frame on the alignpoints and rates how sharp each one is.“*

- Wichtig: Segmente No Align · Scan Frames (beide noch ohne Funktion: gesperrt, Tooltip und Hinweis „Not available yet“
  darunter); Alignmentbox size; Max Alignpoint movement.
- Advanced: der Rest der Gruppe Alignment setup.
- Aktion: **Align**. Frame-Leiste sichtbar.

### 4 Limit

*„Keeps the sharpest frames and drops the rest. Only the kept frames go into the stack.“*

- Wichtig: die Gruppe Limit Setup vollständig, darunter das **Qualitätsdiagramm**: ein Balken je Frame, nach Qualität
  sortiert, bestes Frame links; Fläche `--field`, Umriss `--line`, volle Panelbreite, 76 px hoch. Behaltene Frames in
  `--accent`, verworfene in `--line-strong`. Die Grenze ist eine gestrichelte Linie in `--text` mit dem Wert rechts
  (10 px, `--muted`): bei Lowest Quality (%) waagerecht auf der Höhe des Werts, bei den übrigen Arten senkrecht nach
  dem letzten behaltenen Frame. Unten „best frame“ links und „weakest“ rechts (10 px, `--muted`). Bei mehr Frames
  als Pixeln fasst ein Balken mehrere Frames zusammen. Das Diagramm folgt jeder Änderung des Werts sofort und hat eine
  Textalternative („Quality of 10 frames, best first. 8 are kept.“). Ausnahme Frames/Apoint: Welche Frames je Punkt
  bleiben, rechnet erst der Rechenkern; das Diagramm zeigt in dieser Art die Auswahl des letzten Limit (davor alle
  Frames) und folgt dem Wert nicht sofort.
- Advanced: entfällt.
- Aktion: **Limit**. Frame-Leiste sichtbar.

### 5 Stack

*„Adds the kept frames into one image with far less noise.“*

- Wichtig: StackSize; Use Nearestby Alignpoints; Correct geometry.
- Advanced: die übrigen Stacking options; Resampling und Drizzling (aus der früheren Aktionszeile); ihre Zusatzfelder
  (Methode und Faktor bzw. Faktor) erscheinen erst mit dem Häkchen, rechts daneben in derselben Zeile. Beide sind noch nicht
  umgesetzt: die Häkchen sind gesperrt (Tooltip und Hinweis „Not available yet“ rechts in der Zeile), die Zusatzfelder
  bleiben deshalb ausgeblendet (RK-006-Prüflauf: eingeschaltet wirkten sie nicht); die Gruppe Create AVI vollständig mit Save Aligned und Save Integrated.
- Ebenso gesperrt, weil das Backend sie nicht auswertet: StackSize Minimized, Maximized und R.of Interest (Default bleibt
  wählbar), Stretch histogram after stacking und De-rotate images. Gleiche Behandlung wie Resampling: ausgegraut, Tooltip und
  „Not available yet“ rechts in der Zeile (bei langem Namen darunter, rechtsbündig), Wert bleibt im Datenmodell. Es wirken:
  Use Nearestby Alignpoints, Correct geometry, Normalization of frame intensity und Do not stack n pixels at the frameedge.
  Entwurf und Soll-Bilder zeigen diese Optionen noch bedienbar; sie bleiben das Ziel, bis die Funktionen gebaut sind.
- Aktion: **Stack**. Im erledigten Schritt steht unter dem Ergebnis **Save image** als Standard-Schaltfläche für das
  Summenbild.

### 6 Wavelets

*„Sharpens the stack. Raise the first layers for fine detail, then apply them to the whole image.“*

- Wichtig: die Häkchen **Colour** und **LRGB** (Sperren wie bisher); die sechs Ebenen mit **Reset Wavelets** als
  Textlink in der Überschrift. Eine Ebene belegt zwei Zeilen: oben Häkchen mit Ebenennummer, Denoise, Sharpen und die
  Preview-Schaltfläche mit dem Reglerwert, darunter der Regler in voller Breite. Über der ersten Ebene stehen die
  Spaltenköpfe Layer · Denoise · Sharpen · Preview. Die Preview-Schaltfläche ist noch ohne Funktion und gesperrt (Tooltip
  „Not available yet“, der Reglerwert bleibt lesbar).
- Advanced: Automatic, Hold Wavelet Setting, Waveletscheme, Initial Layer und Step Incr. in einer Zeile, Wavelet
  filter, Use Linked Wavelets; Available schemes mit Load Scheme und Save Scheme; Realign with Processed und Stack Again. Beide sind gesperrt; Realign
  trägt wie die gesperrten Stack-Optionen Tooltip und den Hinweis „Not available yet“ (darunter), Stack Again ist wie im Original gesperrt.
- Aktion: **Do All**, links daneben **Process** als Standard-Schaltfläche in derselben Zeile.
- Rechtes Panel: Functions als Raster aus drei Spalten und fünf Zeilen in der Reihenfolge des Originals, jede
  Schaltfläche ein Umschalter für ihr Panel · Contrast/Brightness mit zwei Reglern und zwei Zahlenfeldern · Copy To /
  Load to / Difference mit der Auswahl Toggle. **Functions »** oben im Bildbereich links neben Show blendet das rechte
  Panel aus und ein; der Bildbereich nimmt den frei werdenden Platz.

### 7 Save

*„Writes the finished image to a file.“*

- Wichtig: ein Satz in 12 px `--muted`, was gespeichert wird: „The sharpened stack with the colour and tone settings.“
  Passt das Ergebnis von Do All nicht zu den aktuellen Werten, darunter: „Do All runs first with the current values.“
- Advanced: entfällt.
- Aktion: **Save image** mit dem Menü der fünf Dateitypen. Im erledigten Schritt steht der Dateiname als Ergebnis.

### Soll-Bilder der Schritte

| Zustand | Soll-Bild |
|---|---|
| leer, Schritt Open | ![Open](design/screenshots/01-open.png) |
| Aufnahme geladen, Schritt Alignpoints | ![Alignpoints](design/screenshots/02-alignpoints.png) |
| Alignpoints gesetzt, Schritt Align | ![Align](design/screenshots/03-align.png) |
| Align läuft | ![Align, läuft](design/screenshots/04-align-running.png) |
| ausgerichtet, Schritt Limit, mit Framelist und Registrationgraph | ![Limit](design/screenshots/05-limit.png) |
| begrenzt, Schritt Stack, Show AlignSections mit der Karte Alignmentpoints | ![Stack](design/screenshots/06-stack.png) |
| gestackt, Stack wieder geöffnet, Advanced offen, Resampling und Drizzling eingeschaltet | ![Stack, erledigt](design/screenshots/07-stack-done.png) |
| gestackt, Schritt Wavelets, Standardwerte | ![Wavelets](design/screenshots/08-wavelets.png) |
| Ebenen 1–3 angehoben, Show Processing Area | ![Wavelets, geschärft](design/screenshots/09-wavelets-sharpened.png) |
| geschärft, Schritt Save | ![Save](design/screenshots/10-save.png) |

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

**Colour und LRGB** sitzen im Original als Häkchen im Align- und im Stack-Reiter (`docs/ui.md`), im Klon seit RK-014
im Schritt Wavelets über den Ebenen. *Colour* verarbeitet in Farbe, ohne
das Häkchen als Graustufen. *LRGB* ist nur mit *Colour* wählbar und schärft nur die Luminanz: Die Wavelets (RK-007)
rechnen auf der Luminanz, Farbton und Sättigung kommen aus dem Summenbild. Beide sind bei Graustufenbildern gesperrt.
Umgesetzt (RK-008): Sie sind bei Graustufenaufnahmen gesperrt und LRGB zusätzlich ohne *Colour*. Umgesetzt (RK-007): Beide
Werte (`settings.colour`, `settings.lrgb`) wirken auf die Live-Vorschau und auf Do All (`colourMode` in `app/src/logic/wavelet.ts`,
Rechnung `wavelet::ColourMode` mit `color::with_luminance`, siehe `docs/wavelets.md`). Ohne *Colour* ist das Ergebnis Graustufen.

### Menüs

| Menü | Soll-Bild |
|---|---|
| Settings | ![Menü Settings](design/screenshots/11-menu-settings.png) |
| Flat/Dark/Reference | ![Menü Flat/Dark/Reference](design/screenshots/12-menu-flat-dark-reference.png) |

MRU zeigt die zuletzt geöffneten Dateien als Einträge, leer den Text „No recent files“ in `--muted`. Tools hat zwei
Einträge: eine Aktion und ein Häkchen (Open und Save stehen nicht mehr dort, sondern als Schaltflächen daneben).

### Doku

Help in der Werkzeugleiste öffnet die Doku in einem eigenen Fenster (1000 × 720 px, mindestens 640 × 400), zentriert auf
dem Bildschirm und vor dem Hauptfenster,
nicht in der App selbst. Ein zweiter Klick holt das offene Fenster nach vorn; Escape oder Schließen beendet es. Die
Arbeitsansicht bleibt unberührt und benutzbar. Das Help-Fenster zeigt nur die Doku, ohne Werkzeugleiste, Schrittleiste
und Statusleiste; die Schaltfläche Help bleibt im Hauptfenster ungedrückt wie jede Schaltfläche ohne Zustand.

Zwei Spalten: links die Seitenliste (Breite wie das linke Panel, 272 px, `--panel`, Linie rechts), rechts der Text auf
`--bg`, 720 px breit und waagerecht zentriert, 36 px Abstand oben. In einem schmaleren Fenster schrumpft die Spalte samt
Seitenrand (48 px, bis 16 px) mit; es gibt keinen waagerechten Balken, auch nicht bei 640 px Fensterbreite.

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

Die Soll-Bilder 13 und 14 (Help-Fenster, 1000×720 px) stammen aus `docs/design/siderea-planets.figcraft` (Frames „Docs ·
dark“ und „Docs · light“), nicht aus dem klickbaren Entwurf; der Entwurf zeigt die Doku nicht.

### Helles Thema

| Schritt | Soll-Bild |
|---|---|
| Limit | ![Limit, hell](design/screenshots/15-limit-light.png) |
| Wavelets | ![Wavelets, hell](design/screenshots/16-wavelets-light.png) |

## Bewusste Abweichungen vom Original

| Original | Klon | Grund |
|---|---|---|
| Skin „Office 2007 Silver“, hell | neutral dunkel, hell umschaltbar | RK-010; Bildbeurteilung |
| Framelist, Registrationgraph, Alignmentpoints als eigene Fenster | Karten über dem Bild | ein Fenster, nichts geht hinter anderen verloren |
| Panelköpfe mit Minimieren/Maximieren | Gruppenüberschriften | Panels sind fest angeordnet |
| Scrollbalken am Bild | Zoom-Leiste, Mausrad, Ziehen | RK-010 verlangt Zoom, Pan, Vorher/Nachher |
| grünes Farbfeld links unten | Statuspunkt mit Text | Bedeutung des Farbfelds ist ungeklärt, Text sagt sie |
| „File Version“ und Datum in der Reiterleiste | Dateiname, Codec, Größe, Frame-Zahl rechts in der Schrittleiste | Version gehört in About |
| drei Reiter Align · Stack · Wavelet, Schritt-Schaltflächen in der Aktionszeile, alle Einstellungen eines Reiters im Panel | sieben Schritte in der Schrittleiste, das Panel zeigt nur den offenen Schritt mit seiner Aktion, Seltenes unter Advanced | RK-014; Philipp wollte, dass die App durch den Ablauf führt (Vorbild Siderea) |
| Häkchen „Show …“ in der Aktionszeile | Menü Show oben rechts am Bild | RK-014; die Aktionszeile entfällt |
| Colour und LRGB im Align- und Stack-Reiter | im Schritt Wavelets | RK-014; sie wirken auf die Schärfung (RK-007) |
| „Save image“ rot beschriftet | Standard-Schaltfläche | Rot bedeutet Gefahr |
| Radiogruppen bei zwei Möglichkeiten | Segmente | weniger Höhe, gleiche Stelle |
| „Intensity_selection“, „Realign_with Processed“ | ohne Unterstrich | Artefakte der Delphi-Beschriftung |
| „Step Increment“ | „Step Incr.“ mit vollem Namen im Tooltip | Platz neben Initial Layer |
| Set Alignpoints und Align ab Align gesperrt | bleiben bedienbar, ein neuer Lauf verwirft Limit, Stack und Wavelet-Ergebnis | RK-005; Philipp wollte ohne erneutes Öffnen neu ausrichten können, siehe [Zustände](#zustände) |
| Dezimalkomma je nach Windows-Einstellung | Punkt | einheitlich auf allen Plattformen |
| CPUs: Standard 1 | Standard = Zahl der logischen Kerne, Bereich 1 bis dahin; die Zahl wirkt auf alle Berechnungen (eigener Thread-Pool im Backend) und wird nicht in der Projektdatei gespeichert | RK-010; Philipp (2026-11-12): die Auswahl soll wirklich wirken und der Rechner standardmäßig ausgelastet sein; die Kernzahl gehört zum Rechner, nicht zum Projekt |
| » klappt das Functions-Panel | „Functions »“ oben im Bildbereich | Beschriftung sagt, was geklappt wird |

## Mac-App: bewusste Abweichungen

Die Mac-App (`mac/App`, SwiftUI) übernimmt Tokens, Fensteraufbau, Schrittleiste, Panels, Bildbereich, Frame- und Statusleiste dieses
Dokuments. Bedienelemente und Eingaben folgen dort, wo es eine Mac-Konvention gibt, dieser statt dem Entwurf:

| Entwurf (Windows) | Mac-App | Grund |
|---|---|---|
| Werkzeugleiste als eigene Leiste, 44 px, Fenstertitel nur der des Systems | Native Werkzeugleiste im Fenster (Select, MRU, in der Mitte Cancel und Pause); Titel `SidereaPlanets — ‹Datei›` in der Titelleiste | Mac-Fensteraufbau; Help, CPUs und die Projekt-Schaltflächen kommen mit MAC-009 und MAC-010 |
| Select und MRU nur in der Werkzeugleiste | zusätzlich **⌘O** „Open…“ und **Datei > Open Recent** (mit „Clear Menu“) | Menüleiste und ⌘-Kürzel (Mac-Konvention) |
| Dateidialog des Betriebssystems | `NSOpenPanel`, Mehrfachauswahl; die Dateien einer Bildserie gehen nach Namen sortiert an den Kern | nativ |
| Mausrad zoomt, Ziehen verschiebt | Pinch auf dem Trackpad zoomt um die Finger, **⌥ oder ⌘ + Scrollen** zoomt um den Zeiger, einfaches Scrollen und Ziehen verschieben, Zwei-Finger-Doppeltipp schaltet zwischen Fit und 100 % | Mac-Gesten; ein Mausrad ohne Taste zoomt nicht |
| Zoom-Leiste − · Prozent · + · Fit | gleich, dazu **⌘+ ⌘- ⌘0** (Fit) und **⌘1** (100 %) im Menü View (auf einem deutschen Mac „Darstellung“) | Mac-Kürzel |
| Frame-Leiste: Goto Frame und Regler | zusätzlich Schaltflächen zum vorigen und nächsten Frame sowie **⌘[** und **⌘]** | schnelles Blättern ohne Regler |
| Cancel und Pause nur als Schaltflächen der Werkzeugleiste | zusätzlich Menü **Process** mit **Cancel (⌘.)** und **Pause/Resume**; Cancel ist in der Werkzeugleiste gefüllt (`--accent`), solange Align läuft; beide wirken nur auf Align (Set Alignpoints und Limit sind in einem Augenblick fertig) | Mac-Kürzel ⌘. für „abbrechen“ |
| Show ▾ mit Häkchen als eigenes Menü, Radiobuttons und Zahlenfelder mit eigenen Pfeiltasten | natives Menü (Häkchen links), Radiobuttons als eigene Zeilen in `--accent`, Zahlenfeld mit nativem `Stepper`; der Threshold-Regler färbt die Spur links vom Griff in `--accent` | Mac-Steuerelemente; ein `Slider` ohne Einfärbung gibt es in SwiftUI nicht |
| Thema in Settings > Theme umschaltbar | folgt der Darstellung von macOS (hell und dunkel), kein Theme-Eintrag | Systemeinstellung gilt |
| Felder, Regler, Stepper, Menüs der Oberfläche | Systemschrift (SF) in 13 und 12 pt, native Steuerelemente (`Stepper`, `Slider`, `TextField`) mit Akzent `--accent`; eigene Gestaltung nur dort, wo dieses Dokument sie festlegt (Pillen, Schaltflächen, Zoom-Leiste, Statusleiste) | Mac-Steuerelemente, Fokus und Bedienungshilfen kommen vom System |
| – | Eine Datei aus dem Finder auf das Fenster ziehen öffnet sie | Mac-Konvention |
| Ansicht der Doku im Help-Fenster | Help-Fenster, About im App-Menü, Einstellungen im Menü (⌘,) und Vollbild nach Mac-Art | folgt mit MAC-010 |

Die Dateiangaben im Schritt Open zeigen auf dem Mac zusätzlich zu Dateiname, Codec, Größe und Frame-Zahl auch Format, Bittiefe,
Farbe, Bildrate und (wo die Datei sie hat) Beobachter, Instrument und Teleskop; Windows zieht das nach
(`docs/parity/WINDOWS_PARITY.md`).

## Was im Entwurf nur Platzhalter ist

- Das Jupiter-Bild ist gezeichnet. Im Klon steht dort das Frame bzw. das Summenbild.
- Die Schärfung beim Bewegen der Ebenen-Regler ist ein einfacher Unscharf-Filter, nicht der Wavelet-Algorithmus.
- Qualitätswerte, Punktpositionen, Speicherangaben und Zeiten sind erfunden.
- Die Functions-Schaltflächen schalten nur ihren gedrückten Zustand um.

## Noch nicht festgelegt

Für diese Teile gibt es kein Soll-Bild. Wer sie braucht, baut sie aus den vorhandenen Komponenten und ergänzt
dieses Dokument im selben Commit.

- Zustand vor dem Laden (festgelegt in RK-010, angepasst in RK-014): Schritt Open ist offen, Select im Fuß des Panels
  gefüllt, alle anderen Pillen gesperrt, keine Frame-Leiste. Der Bildbereich zeigt mittig „Press Select to open a recording“ in `--on-canvas` mit 60 % Deckkraft.
- Fehlermeldungen (festgelegt in RK-010): Text mit Präfix „Error:“ im Ergebnisfeld der Statusleiste (Feld 3), in
  `--text`; kein Dialog, kein Rot.
- Speicherangabe („Memory used/free/total“) in der Schrittleiste: entfällt vorerst, sie braucht einen Befehl im Backend.
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
| RK-014 Geführter Ablauf | Schrittleiste, Schrittpanel, Show am Bild, Qualitätsdiagramm in Limit; ersetzt die Reiter in den Zeilen darüber (Bildnummern dort sind die alten) | 01–12, 15, 16 |

## Zustände im Klon ansehen (Dev-Build)

Im Dev-Build (`pnpm tauri dev` oder `pnpm exec vite`) springt `app/src/preview.ts` mit denselben URL-Parametern wie der
Entwurf in einen Zustand, mit erzeugten Daten statt einer Aufnahme: `step`, `view`, `theme=light`, `busy`, `show`
(framelist, graph, resampling, drizzling, procarea, alignpoints, nosections), `layers`. Im Release-Build fehlt das Modul.

## Soll-Bilder neu erzeugen

Der Entwurf springt über URL-Parameter in einen Zustand: `step` (empty, loaded, points, aligned, limited, stacked,
processed, saved), `view` (open, alignpoints, align, limit, stack, wavelets, save; ohne Angabe der nächste offene
Schritt), `advanced=1` (Advanced aufgeklappt), `theme=light`, `show` (framelist, graph, resampling, drizzling, procarea,
alignpoints, sections), `menu` (mru, flat, tools, settings, show), `layers` (sechs Reglerwerte), `busy` (Prozent),
`chrome=0` (blendet die Leiste des Entwurfs aus). Die Bilder entstehen mit einem Browser ohne Fenster:

```
msedge --headless=new --window-size=1440,900 --hide-scrollbars   --screenshot=docs/design/screenshots/05-limit.png   "file:///…/docs/design/oberflaeche-entwurf.html?chrome=0&step=aligned&show=framelist,graph"
```

| Bild | Parameter |
|---|---|
| 01-open | `step=empty` |
| 02-alignpoints | `step=loaded` |
| 03-align | `step=points` |
| 04-align-running | `step=points&busy=40` |
| 05-limit | `step=aligned&show=framelist,graph` |
| 06-stack | `step=limited&show=sections` |
| 07-stack-done | `step=stacked&view=stack&advanced=1&show=resampling,drizzling` |
| 08-wavelets | `step=stacked` |
| 09-wavelets-sharpened | `step=stacked&layers=32,22,14,10,10,10&show=procarea` |
| 10-save | `step=processed` |
| 11-menu-settings | `step=loaded&menu=settings` |
| 12-menu-flat-dark-reference | `step=loaded&menu=flat` |
| 15-limit-light | `step=aligned&show=framelist,graph&theme=light` |
| 16-wavelets-light | `step=stacked&theme=light` |

13 und 14 (Doku) kommen aus der FigCraft-Datei: Frame in FigCraft auswählen und als PNG exportieren (Maßstab 1).
