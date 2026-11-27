# Design der Oberfläche

Verbindliche Vorgabe für alles, was im Klon sichtbar ist. Umgesetzt wird sie im Frontend unter `app/src` (SP-010) und in der
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
| `docs/design/vorschlag-gefuehrter-ablauf.html` | Überholt: die drei Vorschläge, aus denen die Schrittleiste (SP-014) gewählt wurde |
| `docs/design/figcraft-build.mjs` | Baut die FigCraft-Datei neu und liest die Farben dabei aus `tokens.css`. Aufruf: `node docs/design/figcraft-build.mjs` |
| `docs/design/icon/` | App-Icon beider Apps: Icon-Composer-Dokumente `AppIcon.icon` und `AppIconDev.icon`, 1024-px-Master `AppIcon-1024.png`, Quellen in `src/` (siehe [App-Icon](#app-icon)) |

Alle Frames, auch die zwei Help-Fenster (SP-012, siehe [Doku](#doku)), entstehen aus dem Skript. Ein Lauf schreibt die
Datei neu und verliert nichts; Änderungen gehören deshalb ins Skript, nicht von Hand in die Datei.

Der Entwurf ist eine Vorlage zum Ansehen, kein Code zum Übernehmen: Er ist eine einzelne HTML-Datei ohne Vue und
ohne Rechenkern. Markup und CSS-Regeln dürfen als Orientierung dienen, die Umsetzung baut eigene Komponenten.

## Grundsätze

1. **Gleicher Aufbau, verständliche Namen.** Jedes Bedienelement entspricht einem Element des Referenzprogramms; sein Name in der App
   folgt seit MAC-014 den Regeln unter [Texte und Schreibweise](#texte-und-schreibweise-mac-014): Wo das Original einen
   unklaren Namen hat, steht in diesem Dokument „Original-Begriff (in der App: Neuer Name)“, im Text der App und in der
   Doku nur der neue Name. Wo es sitzt, bestimmt der Ablauf (SP-014): Der Klon führt in sieben Schritten durch die
   Bedienung, statt sie wie das Original auf Reiter, Aktionszeile und Panel zu verteilen. Wer das Original kennt, findet
   jedes Element über die Spalte „Name in Siderea Planets“ in `docs/ui.md`.
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
│ Werkzeugleiste   Select Recording  Recent Recordings  Flat/Dark/Reference  Tools  Settings  Open Project…  Save Project… │ Cancel Pause │ Help About  CPU cores │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Schrittleiste  ①Open ─ ②Alignpoints ─ ③Align ─ ④Best Frames ─ ⑤Stack ─ ⑥Wavelets ─ ⑦Save   Datei │
├─────────────┬─────────────────────────────────────────────────┬───────────────────┤
│ Panel links │ Bildbereich   Bildname          Adjust › Show ▾ │ Panel rechts      │
│ Kopf        │   schwebende Karten oben rechts                 │ 248 px            │
│ Felder      │                                                 │ nur Wavelets      │
│ Fuß: Aktion │   Zoom-Leiste unten mittig                      │                   │
├─────────────┴─────────────────────────────────────────────────┴───────────────────┤
│ Frame-Leiste     Status · Go to frame · Regler über alle Frames · Frame n of N    │
├───────────────────────────────────────────────────────────────────────────────────┤
│ Statusleiste     Fortschritt │ Aktion │ Ergebnis │ Position │ Zeit                │
└───────────────────────────────────────────────────────────────────────────────────┘
```

| Bereich | Höhe | Inhalt und Regeln |
|---|---|---|
| Werkzeugleiste | 44 px | Drei Zonen: links Select (in der App: Select Recording), die vier Menüs (MRU: in der App Recent Recordings) und die Projekt-Schaltflächen Open und Save (in der App: Open Project… und Save Project…), in der Mitte (im Rest zwischen links und rechts zentriert) Cancel und Pause, rechts Help, About und CPUs (in der App: CPU cores to use; die Zahl der Threads aller Berechnungen: Standard alle logischen Kerne, Bereich 1 bis dahin; das Feld klemmt Eingaben darauf). Help, Open und Save gibt es im Original nicht: Help öffnet die Doku in der App (SP-012), Open und Save öffnen und speichern die Projektdatei (SP-009) |
| Schrittleiste | 40 px | Die sieben Schritte als Pillen (siehe [Schrittleiste](#schrittleiste)). Rechts Dateiname, Größe, Frame-Zahl und Codec in `--muted` („1024×680 · 10 frames · DIB“), wird bei Platzmangel mit „…“ gekürzt, die Pillen nie |
| Arbeitsbereich | Rest | Linkes Panel fest 272 px mit dem offenen Schritt (siehe [Schrittpanel](#schrittpanel)), Bildbereich füllt den Rest, rechtes Panel 248 px nur im Schritt Wavelets. Panels scrollen senkrecht, wenn das Fenster zu niedrig ist; der Fuß des linken Panels mit der Aktion bleibt stehen |
| Frame-Leiste | 40 px | Nur in den Schritten Alignpoints, Align und Limit (in der App: Best Frames) |
| Statusleiste | 28 px | Immer sichtbar |

Mindestgröße des Fensters: 1120×600. Darunter wird nichts umgebrochen oder ausgeblendet.

Eine eigene Titelleiste gibt es nicht, das Fenster nutzt die des Betriebssystems. Ihr Text nach dem Laden:
`Siderea Planets — ‹Dateiname›`, während einer Berechnung zusätzlich Schritt und Prozent (`Siderea Planets · Align 40%`),
damit der Fortschritt wie im Original in der Taskleiste lesbar ist.

## Komponenten

### Schaltflächen

| Art | Aussehen | Einsatz |
|---|---|---|
| Standard | `--raised`, Umriss `--line-strong`, Höhe `--h-btn`, Radius `--r` | Save image (in der App: Save Image), Process (in der App: Update Preview), Load Scheme (in der App: Load Scheme…) und alle übrigen |
| Gefüllt | `--accent` mit `--on-accent`, Gewicht 600 | die Aktion des offenen Schritts im Fuß des Schrittpanels (Select (in der App: Select Recording), Set Alignpoints, Align, Limit (in der App: Keep Best Frames), Stack, Do All, Save image (in der App: Save Image)) oder „Continue: ‹Schritt› ›“; Cancel während einer Berechnung |
| Ohne Rahmen | durchsichtig, Hover `--raised` | Select in der Werkzeugleiste (in der App: Select Recording), Menüs der Werkzeugleiste (mit Pfeil nach unten), Open und Save (in der App: Open Project… und Save Project…, mit Icon), Help (mit Icon), About, Show am Bild |
| Textlink | `--accent-ink`, ohne Fläche | Aktionen in Gruppenköpfen: Reset Wavelets, Hold (in der App: Keep), Reset |
| Umschalter | gedrückt: `--accent-soft`, Umriss `--accent`, Text `--accent-ink` | Functions-Raster (in der App: Adjustments), Functions » (in der App: Adjust ›), Before (in der App: Before/After) |
| Gesperrt | Text `--off`, Umriss `--line`, keine Fläche | jede Art |

Icons sind Linienbilder auf 16×16 px, Strich 1,5 px mit runden Enden, Farbe `--muted` (Hover `--text`), ohne Füllung.
Sie stehen links vom Text. Es gibt drei: Help, ein aufgeschlagenes Buch (Doku, nicht Info); Open, eine Mappe; Save, eine
Diskette. Das Help-SVG steht im klickbaren Entwurf, dieselbe Geometrie in `figcraft-build.mjs`; Open und Save (SP-009) stehen
nur in `app/src/components/Toolbar.vue` (Pfade `M1.5 3.5h4l1.5 1.5h7.5v7.5h-13z` und `M2.5 2.5h8l3 3v8h-11zM5 2.5v3.5h5V2.5M5 13.5v-4h6v4`).

Pro Zeile ist höchstens eine Schaltfläche gefüllt. Rot kommt nicht vor: „Save image“ (in der App: Save Image) ist im Original rot
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
   `--ok`, 12 px, z. B. „✓ Alignpoints · 82 points“.
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
     „Then: ‹nächster Schritt›“ (12 px, `--muted`). Weitere Aktionen des Schritts (Process im Schritt Wavelets, in der App: Update Preview) stehen
     als Standard-Schaltflächen in derselben Zeile links daneben.
   - erledigter, wieder geöffneter Schritt: oben das Ergebnis in `--ok`, darunter „‹Aktion› Again“ als
     Standard-Schaltfläche in voller Breite, darunter in 12 px `--muted` „Running it again discards the results of the
     later steps.“ (nur, wenn es Schritte danach gibt, die etwas verwerfen würden), zuletzt gefüllt „Continue: ‹nächster offener
     Schritt› ›“.
   - noch nicht möglicher Schritt kommt nicht vor, er lässt sich nicht öffnen.

Während einer Berechnung ist die Aktion im Fuß gesperrt; gefüllt ist dann nur Cancel in der Werkzeugleiste.

Nach jedem erfolgreichen Lauf öffnet sich der nächste Schritt von selbst (nach Select also Alignpoints, nach Do All
Save). Nach Save image (in der App: Save Image) bleibt Save offen und zeigt das Ergebnis; „Continue“ entfällt dort, an seiner Stelle steht
gefüllt „Select Another Recording…“ (bis MAC-014 „Open another recording“; öffnet wie Select Recording).

Die Ergebnistexte:

| Schritt | Ergebnis (Ergebniszeile, Tooltip der Pille) |
|---|---|
| Open | ‹Dateiname› · ‹N› frames |
| Alignpoints | ‹n› points (bis MAC-014 „‹n› set“) |
| Align | ‹N› frames rated (bis MAC-014 „best quality ‹q› %“, ein Sachfehler: die Qualität ist auf den besten Frame normiert, der beste hat immer 100 %) |
| Limit (in der App: Best Frames) | ‹k› of ‹N› frames kept |
| Stack | ‹n› frames · ‹B›×‹H› |
| Wavelets | Sharpening applied |
| Save | ‹Dateiname› |

### Show am Bild

Die Häkchen der früheren Aktionszeile (Show Full Image, Show ROI, Show Alignpoints, Show Aligndata, Show Framelist,
Show Registrationgraph, Show Prefilter; Show AlignSections, Show Stackgraph, Show Stacklist; Show AlignPoints, Show
Processing Area; in der App: Show Full Image, Show Region, Show Alignpoints, Show Align Data, Show Frame List, Show Quality
Graph, Show Prefilter; Show Alignpoints, Show Stack Graph, Show Stack List; Show Alignpoints, Show Preview Area) liegen in einem Menü **Show ▾** oben rechts im Bildbereich, 10 px vom Rand, als Schaltfläche ohne
Rahmen auf `--canvas-chrome` mit Text `--on-canvas`. Das Menü enthält die Häkchen, die das Original im zugehörigen
Reiter hat: in Alignpoints, Align und Limit (in der App: Best Frames) die des Align-Reiters, in Stack die des Stack-Reiters, in Wavelets und Save
die des Wavelet-Reiters. Im Schritt Open gibt es kein Show. Die Häkchen behalten ihre Wirkung, die schwebenden Karten
verhalten sich wie bisher und beginnen unter der Zeile mit Show.

Links oben im Bildbereich steht in derselben Art, aber nicht anklickbar, was das Bild zeigt: „Frame ‹n› of ‹N›“,
„Stack of ‹n› frames“ oder „Sharpened stack“. Im Schritt Wavelets steht links neben Show der Umschalter **Functions »** (in der App: **Adjust ›**)
(gleiche Form, gedrückt mit Umriss und Text in `--accent`).

Text auf dem Bildbereich steht in `--on-canvas` (Hinweise wie „Select a recording or drop a video or images here“, bis MAC-014 „Press Select Recording to open a video or a series of images“, mit 60 % Deckkraft),
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

Dezimaltrenner ist der Punkt. Nachkommastellen: Denoise 2, Sharpen 3, Preview (der Sliderwert; in der App als Text rechts neben dem Regler) 1, Faktoren 1, Gamma 2, Verschiebung in RGB Align 2, Clip in
Histogram 1, alles andere ganzzahlig.

### Regler

Spur 4 px hoch in `--line-strong`, Griff 14 px rund in `--accent`. Die Spur wird links vom Griff nicht eingefärbt,
weil mehrere Regler keinen natürlichen Nullpunkt am linken Rand haben (Ebenen: −5.0 … 100.0, Brightness: −512 … 512).
Ein Regler ohne Bereich (Threshold vor Set Alignpoints) steht bei 45 % Deckkraft.

### Segmente

Zusammenhängende Schaltflächen in einem gemeinsamen Umriss. Zwei Einsätze:

- als Auswahl aus zwei Möglichkeiten: Waveletscheme (in der App: Layer spacing; Dyadic (2^n) · Linear), Wavelet filter (Default · Gaussian).
  Gewählt: `--accent-soft` mit Text in `--accent-ink`.
- als Gruppe verwandter Aktionen: No Align · R.o.Interest (in der App: Draw Region) · Scan Frames; Copy To · Load to · Difference (in der App: Remember · Restore · Difference).

### Menüs

Öffnen unter ihrer Schaltfläche, Fläche `--raised`, Radius `--r-l`, Schatten. Einträge 28 px hoch, Hover
`--accent-soft`. Ein Eintrag ist entweder eine Aktion, ein Häkchen, oder zeigt rechts seinen aktuellen Wert mit
„›“ für ein Untermenü. Zahlen- und Textfelder (Bias subtract, Save to Subdir, Faktor bei Create Darkframe) stehen
rechts im Eintrag. Trennlinien gruppieren wie im Original. Escape und ein Klick daneben schließen.

**Open** und **Save** (SP-009) sind eigene Schaltflächen ohne Rahmen in der Werkzeugleiste, rechts neben dem Menü Settings
(Projektdatei, `docs/formats.md`). Das Original hat sie nicht. In der App heißen sie **Open Project…** und **Save Project…** (bis
MAC-014 „Open“ und „Save“, weil die Werkzeugleiste bei der Mindestbreite von 1120 px sonst überläuft; der Tooltip und das Label
für Screenreader hießen *Open project* und *Save project*). Beide sind während einer Berechnung gesperrt, Save zusätzlich ohne geladene Aufnahme. Tools enthält damit
wieder nur die Einträge des Originals.

**Save image** (in der App: **Save Image**) gibt es zweimal: gefüllt als Aktion des Schritts Save (das geschärfte Bild, fünf Dateitypen) und als
Standard-Schaltfläche im erledigten Schritt Stack unter dem Ergebnis (das Summenbild, sieben Dateitypen). Beide tragen den Pfeil nach unten und öffnen
beim Anklicken ein Menü mit der Auswahl des Dateityps (rechts steht die Endung). Ein Klick auf einen Eintrag öffnet den
Speichern-Dialog des Betriebssystems für genau diesen Typ. Beim Original wählt man den Typ im Dialog; das Dialog-Plugin meldet
den gewählten Filter nicht zurück, deshalb steht die Auswahl davor.

Aus dem Menü Settings entfallen die Update-Prüfung („Check for … Update“) sowie „Layouts“ und „Set Skin“. An die Stelle von „Set Skin“
tritt **Theme** (Dark · Light). Einträge mit „›“ öffnen ihr Untermenü als Flyout rechts neben dem Eintrag (bei Hover oder
Fokus), gleiche Fläche wie das Menü; darin stehen Häkchen, eine Auswahl (Processing Area (in der App: Preview area), Stretch FIT/TIFF, Theme) oder
Zahlenfelder (Luminance Setting mit Reset Luminance).

### Panels und Gruppen

Panels haben keinen eigenen Kopf mit Minimieren/Maximieren. Eine Gruppe beginnt mit ihrer Überschrift (Gewicht 600)
und ist von der nächsten durch eine Linie in `--line` getrennt. Innenabstand des Panels 12 px. Aktionen, die im
Original im Panelkopf stehen (Reset Wavelets, Hold (in der App: Keep), Reset), stehen als Textlinks rechts in der Gruppenüberschrift.

Scrollbalken (Panels, Seitenliste der Doku, Text) sind schmal: 10 px breit, ohne Pfeilknöpfe, Spur durchsichtig, Griff in
`--line-strong` mit Radius 5 px, beim Überfahren `--off`. Der Standardbalken von Windows kommt nirgends vor; er würde
außerdem 17 px der 272 px des Panels kosten und Beschriftungen wie „Minimum distance between“ umbrechen.

### Schwebende Karten

Framelist, Registrationgraph und Alignmentpoints (in der App: Frame List, Quality Graph und Alignpoints Used for Stacking) sind im Original eigene Fenster. Im Klon liegen sie als Karten
oben rechts über dem Bild, untereinander gestapelt, 236 px breit, mit Titel und Schließen-Kreuz. Sichtbar sind sie,
solange das zugehörige Häkchen gesetzt ist; das Kreuz nimmt das Häkchen heraus. Lange Listen scrollen in der Karte.
Die Karten sind nicht verschiebbar.

- Framelist (in der App: Frame List): Check all · Check none · Empty List (in der App: Check All · Check None · Clear List, gesperrt), darunter die Frames („Frame 10“ statt „Frame00010“) mit Häkchen, nach Align nach Qualität sortiert
  mit dem Wert rechts.
- Go to best frame (in der App: Go to Best Frame; neu gegenüber dem Original, MAC-011): zweite Zeile der Karte unter den drei Schaltflächen, kompakte
  Standardschaltfläche über die volle Kartenbreite, bis Align gelaufen ist gesperrt; Wirkung siehe „4 Limit“.
- Registrationgraph (in der App: Quality Graph): Liniendiagramm, Quality in `--accent`, Difference in `--info`, Legende darunter. Vor Align steht
  ein Hinweis statt leerer Achsen.

### Bildbereich

Hintergrund `--canvas`, das Bild mittig mit 12 px Rand eingepasst, ein kaum sichtbarer Umriss zeigt die Bildgrenze.
Überlagerungen (Alignpoints, Processing Area als gestricheltes Rechteck in `--accent`) liegen deckungsgleich auf dem
Bild und zoomen mit.

Die Zoom-Leiste unten mittig ersetzt die Scrollbalken des Originals: − · aktueller Maßstab in Prozent der
Bildpixel (Klick setzt 100 %) · + · Fit, in den Schritten Wavelets und Save zusätzlich der Umschalter **Before** (in der App: **Before/After**), der das
unbearbeitete Summenbild zeigt. Mausrad zoomt, Ziehen verschiebt. Das Bild folgt der Größe der Fläche, bis der Nutzer zoomt oder
verschiebt (ein Klick ohne Bewegung zählt nicht); bei jedem Schrittwechsel wird es neu eingepasst, weil sich die Fläche mit
Frame-Leiste und rechtem Panel ändert (SP-010-Prüflauf: Zoom blieb nach Wavelets bei 73 %).

**View Compare** (MAC-019, die Schaltfläche des Adjustments-Panels; im Original nicht erfasst): teilt das Bild im Schritt Wavelets
an einer senkrechten **Trennlinie** in zwei Hälften: links das Summenbild ohne alles (**Before**), rechts das Bild mit Schärfung und
Karten (**After**, das, was der Bildbereich sonst zeigt). Beide Hälften haben denselben Zoom und dieselbe Lage, es ist ein Bild in zwei
Ausschnitten. Die Linie ist 2 px breit, `--on-canvas` mit 90 % Deckkraft und Schatten, in der Mitte der Griff: eine schwarze Kapsel
(75 %, 22 × 28 px) mit `--accent`-Rand und zwei Pfeilen in `--accent`. Der Griff und ein 30 px breiter Streifen um die Linie sind
ziehbar (Cursor ↔); der Rest des Bildbereichs verschiebt weiter das Bild. Die Linie sitzt bei einem Anteil der Breite des Bildbereichs
(Start 0,5, Grenzen 0,02 … 0,98), bleibt also beim Zoomen und Verschieben stehen; VoiceOver stellt sie in 5-%-Schritten ein.
Die Beschriftungen **Before** und **After** stehen unten links und unten rechts im Bildbereich (Kapsel wie die Beschriftung oben
links, 24 px hoch, 12 px Rand); eine Beschriftung verschwindet, wenn zwischen Linie und Rand weniger als 84 px bleiben. Solange die
Teilung läuft, entfällt die Beschriftung oben links („Sharpened stack“). **Before/After** in der Zoom-Leiste zeigt weiterhin eines der
beiden Bilder ganz: es beendet die Teilung, und die Schaltfläche View Compare beendet *Before*. Die beiden laufen nie zugleich. Nur
im Schritt Wavelets; die Schaltfläche steht wie alle gedrückten Adjustments-Schaltflächen in `wavelet.functions` der Projektdatei, die
Lage der Linie nicht. Die Seite-an-Seite-Variante (zwei Bildflächen) gibt es nicht.

### Frame-Leiste

Von links: Statuspunkt mit Text („Loaded“ blass, „Aligned“ kräftig in `--ok`; er ersetzt das grüne Farbfeld des
Originals), Goto Frame (in der App: Go to frame) als Zahlenfeld, der Regler über alle Frames in voller Restbreite, rechts „Frame (n): n/N“ (in der App: „Frame 5 of 10“).

### Statusleiste

Sechs Felder, durch Linien getrennt, Schrift 12 px in `--muted`, Meldungen in `--text`:

1. Fortschrittsbalken (6 px hoch, `--accent`) mit Prozentzahl, 210 px breit
2. aktuelle Aktion, z. B. „Aligning frame: 4“ (bis MAC-014 „Loading frame: 4“)
3. Ergebnis, z. B. „Alignment finished“ (bis MAC-014 mit Punkt)
4. Position unter dem Mauszeiger in den Schritten Wavelets und Save, z. B. „X=200 Y=201 Stack=10“: das Pixel des angezeigten Bilds
   (gedreht und gespiegelt, wie es zu sehen ist), leer außerhalb des Bilds und in den anderen Schritten
5. „Time used … · Time left …“
6. wie in Siderea Version · Commit · Bauzeit, z. B. `0.1.0 · a4e826b · 2026-11-13 13:38` (Ortszeit auf die Minute, Unbekanntes fällt
   weg), Tooltip „Version of Siderea Planets and the time this build was made.“ Am Mac kommt der Commit aus `SP_BUILD_COMMIT`, das
   `scripts/deploy-mac.sh` und `scripts/build-mac-release.sh` setzen (ein Bau aus Xcode zeigt ihn nicht), die Bauzeit ist das
   Änderungsdatum der ausführbaren Datei

Die Meldungstexte waren die des Originals (`docs/workflow.md`); seit MAC-014 gelten die Texte aus `docs/parity/WINDOWS_PARITY.md`.

Ganz links, vor dem Fortschrittsbalken, steht die **Auslastung** der App (wie in Siderea): „CPU (12)“ mit der Last als Anteil aller
12 Kerne („ 50 %“, ab 90 % `--accent` und fett) und „RAM“ mit dem belegten Speicher („1.5 GB“), Werte in `--text`, Beschriftung
`--muted`, ein Strich statt des Werts ohne Messung, Tooltips erklären die Zahl. Bei Platzmangel fällt zuerst der Speicher weg, nie
bricht die Zeile um. Gemessen wird nur bei sichtbarem Fenster; ein Schalter in den Settings blendet das Feld aus.

Reihenfolge des Wegfallens am Mac: zuerst der Speicher, dann die ganze Auslastung. Die Felder 4 bis 6 (Position, Zeit, Version) fallen
nicht weg und behalten ihre Breite; Aktion und Ergebnis (Felder 2 und 3) werden mit „…“ gekürzt und zeigen den ganzen Text im Tooltip.

## Zustände

Die Sperrlogik stammt aus `docs/workflow.md`; hier steht, wie sie aussieht.

Pillen der Schrittleiste je Zustand (✓ erledigt, ● nächster offener Schritt, ○ möglich, – gesperrt). Angezeigt ist
nach jedem Lauf der nächste offene Schritt (●); jede Pille mit ✓ oder ○ lässt sich öffnen.

| Zustand | Open | Alignpoints | Align | Limit (in der App: Best Frames) | Stack | Wavelets | Save |
|---|---|---|---|---|---|---|---|
| leer | ● | – | – | – | – | – | – |
| Aufnahme geladen | ✓ | ● | – | – | – | – | – |
| Alignpoints gesetzt | ✓ | ✓ | ● | – | – | – | – |
| ausgerichtet | ✓ | ✓ | ✓ | ● | – | – | – |
| begrenzt | ✓ | ✓ | ✓ | ✓ | ● | – | – |
| gestackt | ✓ | ✓ | ✓ | ✓ | ✓ | ● | ○ |
| geschärft (Do All gelaufen) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ● |
| gespeichert | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

- Save ist schon nach Stack möglich, weil Save image (in der App: Save Image) ein noch nicht angewendetes Do All selbst nachholt (SP-009).
- Wavelets gilt als erledigt, solange das Ergebnis von Do All zu den aktuellen Wavelet-Werten passt; ändert man danach
  eine Ebene oder eine Einstellung des Schritts, verliert die Pille den Haken und Wavelets ist wieder der nächste
  offene Schritt. Save verliert dabei ebenfalls den Haken.
- Open bleibt immer anklickbar; Select (in der App: Select Recording) dort (oder in der Werkzeugleiste) öffnet eine neue Aufnahme und setzt alles auf
  „Aufnahme geladen“.

**Erneutes Ausrichten** (Entscheidung von Philipp, SP-005): Set Alignpoints und Align bleiben nach ihrem ersten Lauf
bedienbar, auch nach Limit, Stack und Wavelet. Im Original sind beide ab Align gesperrt (`docs/workflow.md`,
Abschnitt 2.2); der Klon weicht bewusst davon ab, damit man andere Punkte, eine andere Region oder eine andere
Boxgröße ausprobieren kann, ohne die Aufnahme neu zu öffnen. Ein neuer Lauf verwirft alles, was auf der alten
Ausrichtung aufbaut: Qualität und Frame-Auswahl (alle Frames wieder angehakt), Verschiebungsstriche der Punkte, Stack,
geschärftes Bild und Farbanpassung, in der Oberfläche wie im Rechenkern (`StackState`, `WaveletState`, `ColorState`,
`ExportState`, `AlignState`). Der Zustand fällt auf „Alignpoints gesetzt“ zurück, die Pillen ab Limit sind wieder
gesperrt und nach dem Lauf öffnet sich der nächste offene Schritt (nach Set Alignpoints Align, nach Align Limit). Im
wieder geöffneten Schritt steht der Hinweis „Running it again discards the results of the later steps.“ unter „‹Aktion› Again“.
Align verwirft beim Start, nicht erst am Ende: Bricht man
ab, bleibt kein altes Ergebnis stehen, und Save image kann kein Bild des alten Stacks schreiben. Auch ein erneutes
Limit nach Stack verwirft den Stack. Regler, Wavelet-Einstellungen und das Bild von Copy To (in der App: das gemerkte Bild, Remember) bleiben erhalten.

**Während einer Berechnung** (Align, Stack, Process (in der App: Update Preview), Do All): die Aktion im Fuß des Schrittpanels ist gesperrt, keine
Pille ist anklickbar, Cancel ist
gefüllt, Pause ist aktiv und heißt nach dem Anhalten „Resume“. Der Balken in der Statusleiste läuft, Feld 2 nennt
den aktuellen Frame, Feld 5 die Zeiten. Außerhalb einer Berechnung sind Cancel und Pause gesperrt und der Balken steht auf
0 % (auch nach einem Abbruch oder Fehler; die Soll-Bilder der fertigen Schritte zeigen noch 100 %, SP-010-Prüflauf). Das Ergebnis
von Limit steht im Ergebnisfeld als „8 frames will be stacked · weakest kept: 72%“ (Qualität des schwächsten behaltenen Frames; bis MAC-014 „Stacksize: 8  Low quality: 87.20%“).

## Bildschirme

Je Schritt: der Satz im Kopf (Englisch wie die Oberfläche; Philipp nimmt die Sätze noch ab, SP-014), die wichtigen
Einstellungen, was unter Advanced liegt, und die Aktion im Fuß. Die Gruppen und Felder sind die der früheren Reiter
(`docs/ui.md`); die Einteilung in wichtig und Advanced ist die Vorgabe dieses Dokuments.

### 1 Open

*„Choose a recording: a SER or AVI video or a series of images. You can also drop it onto the window.“*

- Wichtig: **Recent**, die zuletzt geöffneten Dateien als Zeilen (Name, darunter der Ordner in 12 px `--muted`), ein
  Klick öffnet die Datei; leer „No recent recordings“ in `--muted` (bis MAC-014 „Recent“ und „No recent files“). Nach dem Laden steht darüber die geladene Datei mit
  Codec, Größe und Frame-Zahl.
- Advanced: entfällt.
- Aktion: **Select Recording…** (bis 2026-11-12 „Select“ wie im Original; umbenannt, weil der Knopf Videos und Bildserien öffnet). Select Recording… und Recent Recordings in der Werkzeugleiste bleiben und tun dasselbe.
- Bildbereich vor dem Laden: mittig „Select a recording or drop a video or images here“ in `--muted` (bis MAC-014 „Press Select Recording to open a video or a series of images“).

### 2 Alignpoints

*„Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer.“*

- Wichtig: der Threshold-Regler mit „weakest“ links, „strongest“ rechts und „Threshold“ in der Mitte, nach dem Setzen
  „Alignpoints: 82“ (in der App: „more points“ · „Threshold“ · „fewer points“, danach „Threshold · 82 points“); Minimum distance between
  (in der App: Min. distance between points (px)); Min distance from edge (in der App: Min. distance from image edge (px));
  **R.o.Interest** (in der App: **Draw Region**) als Umschalter in voller Breite
  (aus dem Segment der Gruppe Alignment setup hierher gezogen, weil Set Alignpoints die Region braucht; zeichnen lässt
  sie sich wie bisher vor und nach Set Alignpoints, bis Align gelaufen ist).
- Advanced: der Rest der Gruppe Set Alignpoint parameters (in der App: Point Placement): Intensity selection (in der App:
  Brightness measure) mit Lo/Hi (in der App: Min. brightness (0–255) und Max. brightness (0–255), je eine Zeile), Number of Alignpoints,
  Keep Alignpoints inside ScanFrame (in der App: Keep points inside the scan frame, gesperrt).
- Aktion: **Set Alignpoints**. Frame-Leiste sichtbar.

### 3 Align

*„Lines up every frame on the alignpoints and rates how sharp each one is.“*

- Wichtig: Alignmentbox size (in der App: Box size per point (px)); Max Alignpoint movement (in der App: Max. point shift (px)).
  Die Segmente No Align · Scan Frames (beide noch ohne Funktion: gesperrt, Tooltip und Hinweis „Not available yet“) stehen seit
  MAC-014 unter Advanced, Beschriftung „Alignment mode“.
- Advanced: der Rest der Gruppe Alignment setup, dazu die Segmente No Align · Scan Frames; Show Alignment und Estimate Rotation sind
  gesperrt. **Align by center of gravity** (Original: Align by Centre of gravity) ist seit MAC-021 frei: Häkchen, darunter eingerückt
  (20 px) **Brightness threshold** (0–255, Standard 20), Feld und Beschriftung nur mit dem Häkchen aktiv, sonst auf 45 % gedimmt und
  Tooltip „Turn on Align by center of gravity to use it.“ Tooltip des Häkchens: „Follows the planet by the centre of its brightness,
  however far it jumps between frames. For a whole planet against a dark sky; leave it off for the Moon or the Sun in close-up.“,
  des Felds: „Pixels up to this brightness (0–255) do not count towards the centre: the sky and its noise. Raise it if a moon or a
  bright edge pulls the planet off.“
- Aktion: **Align**. Frame-Leiste sichtbar.

### 4 Limit (in der App: Best Frames)

Seit MAC-014 heißt der Schritt in der App **Best Frames** (Pille, Panelkopf, Menü Go to Step), seine Aktion **Keep Best Frames**
(erledigt „Keep Best Frames Again“, gesperrt „Run Keep Best Frames first“). Statustext während des Laufs: „Keeping N of M frames“
(bis MAC-014 „Processing limited to N frames“).

*„Keeps the sharpest frames and drops the rest. Only the kept frames go into the stack.“*

- Wichtig: die Gruppe Limit Setup vollständig (in der App: At least % of the best quality, Best % of frames, Best frames per point, Best number of frames; Original: Lowest Quality (%), Best Frames (%), Frames/Apoint, Best Frames), darunter das **Qualitätsdiagramm**: ein Balken je Frame, nach Qualität
  sortiert, bestes Frame links; Fläche `--field`, Umriss `--line`, volle Panelbreite, 76 px hoch. Behaltene Frames in
  `--accent`, verworfene in `--line-strong`. Die Grenze ist eine gestrichelte Linie in `--text` mit dem Wert rechts
  (10 px, `--muted`): bei Lowest Quality (%) (in der App: At least % of the best quality) waagerecht auf der Höhe des Werts, bei den übrigen Arten senkrecht nach
  dem letzten behaltenen Frame. Unten „best frame“ links und „weakest“ rechts (10 px, `--muted`; in der App „sharpest“ und „blurriest“). Bei mehr Frames
  als Pixeln fasst ein Balken mehrere Frames zusammen. Das Diagramm folgt jeder Änderung des Werts sofort und hat eine
  Textalternative („Quality of 10 frames, best first. 8 are kept.“). Ausnahme Frames/Apoint (in der App: Best frames per point): Welche Frames je Punkt
  bleiben, rechnet erst der Rechenkern; das Diagramm zeigt in dieser Art die Auswahl des letzten Limit (davor alle
  Frames) und folgt dem Wert nicht sofort.
- Go to best frame (in der App: Go to Best Frame; neu gegenüber dem Original, MAC-011): Standardschaltfläche über die volle Panelbreite unter dem Diagramm, bis Align
  gelaufen ist gesperrt. Sie zeigt den ersten Platz der Rangliste im Bildbereich und in der Frame-Leiste und öffnet den Schritt
  Alignpoints, dort steht oben ein Hinweis (Fläche `--accent-soft`, Streifen links in `--accent`): „Best frame: 295. Run Set Alignpoints
  and Align again to line all frames up on this one.“ (Nummer ab 1; bis MAC-014 „… run Set Alignpoints and Align again to use it as reference.“). Die Ergebnisse bleiben, bis Set Alignpoints läuft.
- Advanced: entfällt.
- Aktion: **Limit** (in der App: **Keep Best Frames**). Frame-Leiste sichtbar.

### 5 Stack

*„Adds the kept frames into one image with far less noise.“*

- Wichtig: StackSize (in der App: Output size, Default · Smallest common area · Largest area · Region); Use Nearestby Alignpoints (in der App:
  Use nearest points); Correct geometry.
- Advanced: die übrigen Stacking options; Resampling und Drizzling (aus der früheren Aktionszeile); ihre Zusatzfelder
  (Methode und Faktor bzw. Faktor) erscheinen erst mit dem Häkchen, rechts daneben in derselben Zeile. Beide sind noch nicht
  umgesetzt: die Häkchen sind gesperrt (Tooltip und Hinweis „Not available yet“ rechts in der Zeile), die Zusatzfelder
  bleiben deshalb ausgeblendet (SP-006-Prüflauf: eingeschaltet wirkten sie nicht); die Gruppe Create AVI vollständig mit Save Aligned und Save Integrated (in der App: Save Aligned Video… und Save Averaged Video…; Compress Qual: Compression quality, Framerate: Frame rate (fps), die Radios unter der Überschrift Frame size).
- Ebenso gesperrt, weil das Backend sie nicht auswertet: StackSize Minimized, Maximized und R.of Interest (in der App: Smallest common area, Largest area, Region; Default bleibt
  wählbar) und Stretch histogram after stacking. Gleiche Behandlung wie Resampling: ausgegraut, Tooltip und
  „Not available yet“ rechts in der Zeile (bei langem Namen darunter, rechtsbündig), Wert bleibt im Datenmodell. **De-rotate images** ist frei
  (Mac seit MAC-020, Windows seit SP-023; Abschnitt „Derotation (MAC-020)“). Es wirken:
  Use Nearestby Alignpoints, Correct geometry, Normalization of frame intensity (in der App: Even out frame brightness) und Do not stack n pixels at the frameedge (in der App: Ignore edge pixels (px)).
  Seit MAC-014 sind auch Codec, Compression quality, die Frame-size-Radios außer Original size und Use prefilters gesperrt.
  Entwurf und Soll-Bilder zeigen diese Optionen noch bedienbar; sie bleiben das Ziel, bis die Funktionen gebaut sind.
- Aktion: **Stack**. Im erledigten Schritt steht unter dem Ergebnis **Save image** (in der App: Save Image) als Standard-Schaltfläche für das
  Summenbild.

### 6 Wavelets

*„Sharpens the stack. Raise the first layers for fine detail, then press Do All to apply them to the whole image.“*

- Wichtig: die Häkchen **Colour** und **LRGB** (in der App: **Color** und **Sharpen brightness only**; Sperren wie bisher); die sechs Ebenen mit **Reset Wavelets** als
  Textlink in der Überschrift. Eine Ebene belegt zwei Zeilen: oben Häkchen mit Ebenennummer, Denoise, Sharpen und die
  Preview-Schaltfläche mit dem Reglerwert, darunter der Regler in voller Breite. Über der ersten Ebene stehen die
  Spaltenköpfe Layer · Denoise · Sharpen · Preview. Die Preview-Schaltfläche ist noch ohne Funktion und gesperrt (Tooltip
  „Not available yet“, der Reglerwert bleibt lesbar). Seit MAC-014 steht der Sliderwert als Text rechts neben dem Regler
  (z. B. „10.0“) und der gesperrte Preview-Knopf trägt ein Auge-Symbol.
- Advanced (Gruppe Wavelet Settings): Automatic, Hold Wavelet Setting, Waveletscheme, Initial Layer und Step Incr. in einer Zeile, Wavelet
  filter, Use Linked Wavelets (in der App: Run Do All automatically, Keep layer values for a new stack, Layer spacing, First layer size und
  Size step, Wavelet filter, Move all sliders together); Available schemes (in der App: Saved Schemes, gesperrt) mit Load Scheme und Save Scheme (in der App: Load Scheme… und Save Scheme…); Realign with Processed und Stack Again. Beide sind gesperrt; Realign
  trägt wie die gesperrten Stack-Optionen Tooltip und den Hinweis „Not available yet“ (darunter), Stack Again ist wie im Original gesperrt.
- Aktion: **Do All**, links daneben **Process** (in der App: **Update Preview**) als Standard-Schaltfläche in derselben Zeile.
- Rechtes Panel: Functions (in der App: Adjustments; in der Mac-App sind alle bis auf View Zoomed, View Stack Size, Wavelet Filter und Show Line Graph aktiv, diese vier gesperrt; die Windows-App hat nur Histogram, Gamma, Color Mixing, RGB Align, RGB Balance und View Compare aktiv) als Raster aus drei Spalten und fünf Zeilen in der Reihenfolge des Originals, jede
  Schaltfläche ein Umschalter für ihr Panel · Contrast/Brightness mit zwei Reglern und zwei Zahlenfeldern (in der App je Beschriftung und Feld über dem Regler; Hold heißt Keep und ist gesperrt) · Copy To /
  Load to / Difference mit der Auswahl Toggle (in der App: Remember · Restore · Difference mit „Show:“ Current image · Remembered image). **Functions »** (in der App: **Adjust ›**) oben im Bildbereich links neben Show blendet das rechte
  Panel aus und ein; der Bildbereich nimmt den frei werdenden Platz.

### 7 Save

*„Writes the finished image to a file.“*

- Wichtig: ein Satz in 12 px `--muted`, was gespeichert wird: „Saves the sharpened stack with your color and tone settings.“
  Passt das Ergebnis von Do All nicht zu den aktuellen Werten, darunter: „Your latest changes are applied to the whole image first (Do All).“
- Advanced: entfällt.
- Mac (MAC-016, MAC-017): unter dem Hinweis die Gruppe **Crop** (Text, **Crop…**, **Remove Crop**) und darunter die Gruppe **Size** („The picture is saved at 300 × 400 px.“, bei gedrehtem oder gespiegeltem
  Bild „It is turned 15.0° clockwise.“, dazu ein Hinweis, wo man Flip and Rotate und Resize Image findet); beide Gruppen mit einer Linie getrennt.
- Aktion: **Save image** (in der App: **Save Image**) mit dem Menü der fünf Dateitypen („TIFF 16-bit (.tif)…“). Im erledigten Schritt steht der Dateiname als Ergebnis.

### Soll-Bilder der Schritte

| Zustand | Soll-Bild |
|---|---|
| leer, Schritt Open | ![Open](design/screenshots/01-open.png) |
| Aufnahme geladen, Schritt Alignpoints | ![Alignpoints](design/screenshots/02-alignpoints.png) |
| Alignpoints gesetzt, Schritt Align | ![Align](design/screenshots/03-align.png) |
| Align läuft | ![Align, läuft](design/screenshots/04-align-running.png) |
| ausgerichtet, Schritt Limit, mit Framelist und Registrationgraph (in der App: Frame List und Quality Graph) | ![Limit](design/screenshots/05-limit.png) |
| begrenzt, Schritt Stack, Show AlignSections mit der Karte Alignmentpoints (in der App: Show Alignpoints, Karte Alignpoints Used for Stacking) | ![Stack](design/screenshots/06-stack.png) |
| gestackt, Stack wieder geöffnet, Advanced offen, Resampling und Drizzling eingeschaltet | ![Stack, erledigt](design/screenshots/07-stack-done.png) |
| gestackt, Schritt Wavelets, Standardwerte | ![Wavelets](design/screenshots/08-wavelets.png) |
| Ebenen 1–3 angehoben, Show Processing Area (in der App: Show Preview Area) | ![Wavelets, geschärft](design/screenshots/09-wavelets-sharpened.png) |
| geschärft, Schritt Save | ![Save](design/screenshots/10-save.png) |

### Karten hinter den Functions-Schaltflächen

(In der App: Karten hinter den Schaltflächen des Panels **Adjustments**.) Die Schaltflächen des Functions-Rasters öffnen Karten, die Form ist die der schwebenden Karten (Framelist, siehe
[Schwebende Karten](#schwebende-karten)): oben rechts über dem Bild, 236 px breit, Titel und Schließen-Kreuz, nicht
verschiebbar. Es gibt dazu kein Soll-Bild; die Karten sind aus den vorhandenen Komponenten gebaut.

Dieser Abschnitt legt die Karten fest, die SP-008 braucht: **Histogram**, **Gamma**, **RGB Align** und **RGB Balance**,
dazu (MAC-019) **Color Mixing**; **View Compare** ist keine Karte, sondern eine Ansicht (Abschnitt am Ende dieses Teils).
Contrast/Brightness bleibt das feste Panel unter dem Raster (Soll-Bild 07). **Denoise/Deringing** und **Masking** (MAC-018, Mac)
folgen unten in Tabelle und Liste; die Verfahren dahinter stehen in `docs/wavelets.md`. Die übrigen Karten (View …,
Wavelet Filter, Show Linegraph; in der App View Zoomed, View Stack Size, Wavelet Filter, Show Line Graph; diese vier Schaltflächen sind gesperrt,
„Not available yet“) legen ihre Tickets fest und ergänzen dieses Dokument. **Flip and Rotate** und **Resize Image** (MAC-017) stehen im Abschnitt
[Drehen und Größe](#drehen-und-größe-mac-017). **Cropping Area** heißt in der Mac-App **Crop** und hat ein eigenes Werkzeug, siehe
[Zuschneiden](#zuschneiden-mac-016).

**Gemeinsames Verhalten**

- Jede Functions-Schaltfläche ist ein Umschalter: gedrückt zeigt sie ihre Karte, ein zweiter Klick oder das Kreuz in der
  Karte schließt sie und löst die Schaltfläche. Mehrere Karten sind gleichzeitig offen und liegen in der Reihenfolge des
  Öffnens untereinander; reicht die Höhe nicht, scrollt der Kartenstapel.
- Die Karten stellen Werte ein, sie sind keine Schritte. Das Bild im Bildbereich folgt jeder Änderung sofort; die Werte
  gelten auch für das Ergebnis von Do All und für Save image. Eine geschlossene Karte behält ihre Werte, nur **Reset**
  nimmt sie zurück. Das Original wurde dazu nicht beobachtet (`docs/offene-fragen.md`).
- Die Werte werden immer auf das Ausgangsbild angewendet, nicht aufeinander gestapelt, in dieser Reihenfolge: RGB Align,
  Color Mixing, RGB Balance, Histogram, Gamma, Contrast/Brightness. **Before** zeigt das Ausgangsbild ohne sie.
- **Reset** steht als Textlink in der Titelzeile, links vom Kreuz, und setzt die Werte der Karte auf den neutralen
  Zustand: das Bild bleibt unverändert. Weitere Aktionen (Auto) stehen ebenfalls als Textlinks dort.
- Bei einem Graustufenbild sind RGB Align und RGB Balance gesperrt (Color Mixing nicht: dort sind die drei Eingangskanäle gleich, jede
  Ausgabe bekommt ihre eigene Summe, das Bild lässt sich also einfärben); der Tooltip sagt „Needs a color image“ (bis MAC-014 „colour“).
- Zahlenfeld und Regler einer Zeile sind gekoppelt: das eine setzt das andere. Werte außerhalb des Bereichs werden auf
  den Bereich gesetzt.

| Karte | Aufbau von oben nach unten | Standard · Bereich |
|---|---|---|
| Histogram | Graph, 212 × 84 px, Fläche `--field`, Umriss `--line`. Luminanz als gefüllte Fläche in `--line-strong`, darüber Red, Green und Blue als 1-px-Linien in `--ch-r` `--ch-g` `--ch-b`. Waagerecht 0 … 255, senkrecht logarithmisch, weil das Bild meist fast schwarz ist. Unter der Achse zwei Dreiecke für Black und White, ziehbar. Darunter Segmente **Input · Result**, dann die Felder **Black** und **White** (in der App: **Black point** und **White point**), darunter **Auto** mit dem Feld **Clip %** (in der App: **Clip (%)**) | Black 0 · 0 … 254, White 255 · 1 … 255, Black bleibt unter White; Clip 0.1 · 0 … 49; Segment Input |
| Gamma | Regler mit „darker“ links und „brighter“ rechts (12 px, `--muted`) über der Spur, darunter das Feld **Gamma** rechts in der Zeile | 1.00 · 0.10 … 5.00; Regler in Schritten von 0.01 |
| RGB Align | Spaltenköpfe X und Y (12 px, `--muted`), drei Zeilen **Red**, **Green**, **Blue** mit je zwei Feldern, darunter die Zeile **Max shift** mit Feld (in der App: Felder **X (px)**, **Y (px)**, **Max shift (px)**), Hinweis „Auto moves Red and Blue onto Green“ (12 px, `--muted`) | Verschiebung 0.00 px · −20.00 … 20.00, Pfeiltasten ±0.1; positives X nach rechts, positives Y nach unten; Max shift 5 · 1 … 20 |
| Color Mixing | Zeile **Preset** (Beschriftung links, Menü-Schaltfläche **Choose** rechts: Neutral · Swap Red and Blue · Luminance Gray · Red to All Channels). Darunter drei Gruppen **Red output**, **Green output**, **Blue output** (Gruppenkopf 12 px, `--muted`), jede mit drei Zeilen **Red**, **Green**, **Blue** (Eingangskanal): Beschriftung links (40 px), Regler, Feld (36 px) mit „%“ dahinter, wie bei RGB Balance. Zuletzt das Häkchen **Preserve brightness**. In der Titelzeile **Reset** (gesperrt, solange die Matrix neutral ist) | Diagonale 100 %, sonst 0 % · je −200 … 200; Preserve brightness aus |
| RGB Balance | Drei Zeilen **Red**, **Green**, **Blue**: Beschriftung links, Regler, Feld rechts (56 px). Die Regler bleiben in der Akzentfarbe, die Kanäle erkennt man an der Beschriftung | 100 % · 0 … 200 (in der App „%“ hinter dem Feld) |
| Denoise/Deringing | Drei Zeilen aus Beschriftung links, Feld rechts (40 px) und Regler darunter: **Denoise (%)**, **Deringing (%)**, **Radius (px)**; darunter ein Hinweis (12 px, `--muted`) | Denoise 0 · 0 … 100, Deringing 0 · 0 … 100, Radius 4 · 1 … 10; Reset setzt alles auf 0 und 4 |
| Masking | Häkchen **Use mask**, zwei Zeilen wie oben: **Threshold**, **Feather (px)**, darunter in einer Zeile die Häkchen **Invert** und **Show mask** | aus · Threshold 20 · 0 … 255, Feather 4 · 0 … 50; Invert aus, Show mask aus |

- **Histogram**: *Input* zeigt das Bild vor dem Stretch, das ist das Bild, auf dem Black und White sitzen; *Result* zeigt
  das Ergebnis nach allen Schritten. **Auto** setzt Black und White so, dass vom dunkelsten und vom hellsten Ende je Clip %
  der Pixel abgeschnitten werden (gemessen an der Luminanz). Black und White wirken gleich auf alle Kanäle, die Farbe
  bleibt also erhalten.
- **Gamma**: über 1.00 wird das Bild in den Mitteltönen heller, darunter dunkler; Schwarz und Weiß bleiben.
- **Color Mixing** (MAC-019): jede Ausgabe ist die Summe der drei Eingangskanäle mal ihrem Prozentwert (3 × 3 Matrix, Zeile = Ausgabe).
  Negative Summen werden 0, Summen über Weiß abgeschnitten. **Preserve brightness** skaliert jedes Pixel nach dem Mischen so, dass
  es die Luminanz (0,299 R + 0,587 G + 0,114 B) vor dem Mischen behält; ein Pixel, das zu Schwarz wird, wird grau in dieser Luminanz.
  Die Neutral-Matrix lässt das Bild bitgenau unverändert. Der Kern rechnet es zwischen RGB Align und RGB Balance
  (`crates/core/src/color.rs`, `mix_channels`); die Projektdatei hat dafür `tone.mix` (`matrix`: neun Prozentwerte zeilenweise,
  `preserveBrightness`), optional, Windows ignoriert den Schlüssel.
- **RGB Align**: **Auto** sucht in den Kanälen Red und Blue die Verschiebung, mit der ihre Kanten auf die von Green fallen
  (Green ist die Bezugsfarbe), und trägt sie in die Felder ein. Danach lässt sich von Hand nachstellen. Die Verschiebung
  hat Nachkommastellen, das Bild behält seine Größe, am Rand wird der letzte Pixel wiederholt.
- **Denoise/Deringing** und **Masking** (MAC-018; kein Soll-Bild, nur Mac): wirken auf das geschärfte Bild und das ungeschärfte Summenbild, von dem
  es kommt, und stehen in der Reihenfolge **vor** allen anderen Karten: Deringing, Denoise, Maske. 0 und „Use mask“ aus lassen das Bild
  unverändert. **Show mask** ersetzt im Bildbereich das Bild durch das Bild mit blau eingefärbter Maske (der Teil, der geschärft wird); es ist nur
  eine Ansicht, wird nicht gespeichert und nicht in Do All oder Save image gerechnet. *Before* zeigt weiter das Summenbild. Schließt man die Karte
  Masking, ist auch die Maskenansicht weg. Die Werte stehen im Abschnitt `refine` der Projektdatei.
- **Contrast/Brightness** (festes Panel): 100 und 0 lassen das Bild unverändert. Contrast dreht die Tonwerte um Mittelgrau;
  der volle Brightness-Bereich −512 … 512 entspricht dem ganzen Tonumfang. Das Feld Contrast zeigt die Reglerposition
  geteilt durch 2,5 (250 → 100); dieses Verhältnis ist eine Annahme aus dem einen Wertepaar, nicht belegt.
- **Copy To / Load to / Difference** (festes Panel; in der App Remember / Restore / Difference, *Clipboard Image* heißt *Remembered image*): *Copy To* merkt sich das Bild, das gerade im Bildbereich steht (mit
  allen Karten und Contrast/Brightness), als *Clipboard Image*. *Current Image* und *Clipboard Image* im Toggle wechseln
  die Anzeige zwischen beiden; *Clipboard Image* ist gesperrt, bis etwas kopiert wurde. *Load to* macht das kopierte Bild
  zum bearbeiteten Bild und setzt alle Karten und Contrast/Brightness zurück, weil sie im Bild schon stecken.
  *Difference* zeigt den Betrag des Unterschieds beider Bilder, vierfach verstärkt, und folgt dem aktuellen Bild;
  die Schaltfläche steht dabei gedrückt. *Load to* und *Difference* sind ohne Kopie gesperrt, *Difference* auch bei
  verschiedener Bildgröße (Meldung in der Statusleiste). Das Original wurde dazu nicht beobachtet; es gilt nur, was
  `docs/ui.md` über den Toggle sagt.
- Eine Functions-Schaltfläche ohne Karte bleibt unten und meldet „‹Name›: not available yet“ in der Statusleiste. Seit MAC-014 ist sie stattdessen gesperrt (gedimmt, nicht klickbar, Tooltip „Not available yet“).

**Colour und LRGB** (in der App: **Color** und **Sharpen brightness only**) sitzen im Original als Häkchen im Align- und im Stack-Reiter (`docs/ui.md`), im Klon seit SP-014
im Schritt Wavelets über den Ebenen. *Colour* verarbeitet in Farbe, ohne
das Häkchen als Graustufen. *LRGB* ist nur mit *Colour* wählbar und schärft nur die Luminanz: Die Wavelets (SP-007)
rechnen auf der Luminanz, Farbton und Sättigung kommen aus dem Summenbild. Beide sind bei Graustufenbildern gesperrt.
Umgesetzt (SP-008): Sie sind bei Graustufenaufnahmen gesperrt und LRGB zusätzlich ohne *Colour*. Umgesetzt (SP-007): Beide
Werte (`settings.colour`, `settings.lrgb`) wirken auf die Live-Vorschau und auf Do All (`colourMode` in `app/src/logic/wavelet.ts`,
Rechnung `wavelet::ColourMode` mit `color::with_luminance`, siehe `docs/wavelets.md`). Ohne *Colour* ist das Ergebnis Graustufen.

### Menüs

| Menü | Soll-Bild |
|---|---|
| Settings | ![Menü Settings](design/screenshots/11-menu-settings.png) |
| Flat/Dark/Reference | ![Menü Flat/Dark/Reference](design/screenshots/12-menu-flat-dark-reference.png) |

Recent Recordings (im Original „MRU“, umbenannt 2026-11-12) zeigt die zuletzt geöffneten Dateien als Einträge, leer den Text „No recent recordings“ in `--muted` (bis MAC-014 „No recent files“). Tools hat zwei
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
als „Not available yet“. Quelle der Seiten ist Markdown im Repo (SP-012).

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

## Zuschneiden (Mac-016)

Das Werkzeug **Crop** legt fest, welchen Ausschnitt **Save Image** schreibt (nur Schritte Wavelets und Save, nur das geschärfte Bild).
Aussehen und Verhalten wie im Zuschneiden von Siderea, übernommen mit den Maßen von dort.

- **Aufruf:** Schaltfläche **Crop** im Panel Adjustments (Schalter wie die anderen, an solange das Werkzeug offen ist) und **Crop…** im
  Schritt Save, in der Gruppe **Crop** über Save Image (Text „The whole picture is saved.“ / „Only 800 × 600 px is saved.“, dazu der
  Link **Remove Crop**). Beides öffnet dasselbe Werkzeug.
- **Rechteck über dem Bild** (`CropOverlay`, Bildpunkte wie die anderen Marken, Linien in Bildschirmpunkten): Außenseite abgedunkelt
  (Schwarz 55 %, nach Done 40 %), Rand 1 pt Weiß über 3 pt Schwarz (60 %), Drittel-Raster (0,5 pt Weiß 55 % über 2 pt Schwarz 35 %), kleines
  **Fadenkreuz in der Mitte** (Arme 10 pt, höchstens ein Viertel der kürzeren Seite, Rand-Stil), acht quadratische Griffe (8 pt, weiß, Rand
  Schwarz 70 %) und die Größe „Breite × Höhe“ auf einer Platte (11 pt, Schwarz 60 %, Radius 3, Abstand 10 pt von der linken oberen Ecke).
  Die Bildfläche ist in beiden Erscheinungsbildern dunkel, die Farben sind fest. Nach **Done** zeigt das Bild nur Außenseite, Rand und
  Größe.
- **Karte „Crop“** (wie die Karten der Adjustments, oben rechts, immer die oberste): Titel, Link **Reset**, Kreuz (= Cancel), Segmentleiste
  **Free · 1:1 · 3:2 · 4:3**, Zeile *Size* mit „W × H px“, ein Hinweis („Drag on the image to draw the part to keep.“ / „Drag the rectangle
  to move it and its handles to resize it. Save Image writes only what is inside.“) und die Knöpfe **Cancel** und **Done** (gefüllt).
- **Bedienung:** Ziehen im Rechteck verschiebt, Ziehen an Kante oder Ecke (Reichweite 7 pt, höchstens ein Drittel der kürzeren Seite)
  ändert die Größe, Ziehen außerhalb zeichnet ein neues. Ein festes Format bleibt beim Ziehen erhalten und liegt oder steht, wie das
  Rechteck gerade ist (kein Umschalter); ein anderes Format formt das Rechteck um (gleiche Mitte und Fläche). Mindestgröße 8 × 8 px, nie
  außerhalb des Bildes. Zeiger: Fadenkreuz über dem Bild, Hand im Rechteck, Pfeile an Kanten und Ecken. Zoomen und Scrollen gehen weiter,
  Verschieben des Bildes ist während des Werkzeugs aus.
- **View Compare:** beim Öffnen des Werkzeugs geht die geteilte Ansicht aus (Schaltfläche kommt hoch), solange es offen ist, ist View Compare
  gesperrt (Statuszeile „View Compare: finish or cancel the crop first“); danach schaltet man es wieder ein.
- **Anfang:** das Werkzeug öffnet auf dem angewendeten Rechteck, sonst auf der Mitte (80 % jeder Seite, in der Form des Formats).
  **Reset** setzt das ganze Bild und Format Free; ein Rechteck, das das ganze Bild deckt, ist kein Zuschnitt, dann zeigt das Bild keins und
  der nächste Zug zeichnet ein neues.

## Drehen und Größe (MAC-017)

Die Karten **Flip and Rotate** und **Resize Image** (Panel Adjustments, Mac) sind Geometrie-Schritte am Ende von Save Image. Sie sind Karten wie die der
Farben (oben rechts, 236 px, Reset als Textlink, Kreuz), gelten nur für das geschärfte Bild (Schritte Wavelets und Save), nie für das Stack-Bild des
Schritts Stack, und wie die Farbwerte nicht für die Quelle der Karten: *Remember*, *Restore* und *Difference* arbeiten auf dem Bild vor dem Drehen.

**Reihenfolge, festgelegt:** Refine (Denoise/Deringing, Masking) → Farb- und Tonwerte → **Flip and Rotate** → **Crop** → **Resize Image**
(`crates/core/src/geometry.rs`, `crates/session/src/export.rs`). Das Bild im Bildbereich zeigt das gedrehte Bild (Schritte Wavelets und Save, auch die
linke Hälfte von View Compare und Before). Das Zuschneiden arbeitet auf genau diesem Bild: sein Rechteck liegt in den Pixeln des gedrehten Bildes,
also ist es das, was man sieht. Resize Image kommt zuletzt und skaliert, was nach dem Zuschnitt übrig ist; der Bildbereich zeigt es nicht (er bliebe
beim Zoomen auf „Fit“ ohnehin gleich groß), die Karte und die Gruppe Size im Schritt Save nennen die Größe der Datei. Wer erst zuschneidet und dann dreht,
bekäme ein Rechteck, das auf einem Bild liegt, das der Nutzer nicht sieht. Ändert man Spiegelung, Winkel oder Crop to fit, **nachdem** das Zuschneiden
mit Done bestätigt war, wird das Rechteck entfernt (Statuszeile „Crop removed: the picture was flipped or turned“); solange das Werkzeug offen ist,
sind die Bedienelemente der Karte gesperrt („Finish or cancel the crop first“). Die Interpolation ändert keine Größe und lässt den Zuschnitt stehen.
Die Marken des Stacks (Alignpoints, Preview-Bereich) liegen in den Pixeln des Stacks und werden auf einem gedrehten Bild nicht gezeichnet.

**Flip and Rotate** (von oben nach unten): Beschriftung *Flip* mit der Segmentleiste **Horizontal · Vertical** (Schalter, an solange gespiegelt),
Beschriftung *Rotate* mit **Left 90° · Right 90° · 180°**, Zeile **Angle (°)** mit Dezimalfeld (1 Stelle, Schritt 0,1) über einem Regler, Zeile
**Interpolation** mit den Segmenten **Bilinear · Bicubic**, das Häkchen **Crop to fit**, die Zeile *Picture* mit „W × H px“ (Größe nach dem Drehen).
Standard: nichts gespiegelt, 0°, Bicubic, Crop to fit aus · Winkel −180 … 180 in Zehnteln, positiv = im Uhrzeigersinn.

- Die Rechnung im Kern spiegelt zuerst im Rahmen des Bildes und dreht dann im Uhrzeigersinn. Die Schaltflächen wirken aber auf das Bild auf dem
  Bildschirm: ein Spiegeln kehrt das Vorzeichen des Winkels um (`OrientationValues.flipHorizontally`), sonst würde ein gedrehtes Bild an der falschen
  Achse gespiegelt. **Left 90°**, **Right 90°** und **180°** addieren −90, 90 und 180 zum Winkel (Bereich −180 … 180, 180 bleibt 180).
- Vielfache von 90° (und Spiegeln) versetzen nur Pixel, nichts wird gerechnet; 0° und kein Spiegeln lassen das Bild bitgenau stehen. Jeder andere
  Winkel zerlegt sich in ganze Vierteldrehungen und einen Rest von höchstens 45°, der mit Bilinear oder Bicubic (Catmull-Rom) gerechnet wird.
- Der Raum eines frei gedrehten Bildes wächst auf das umschließende Rechteck (Hintergrund schwarz); seine Seiten behalten die Parität der Quelle, damit
  die Mitte des Bildes auf der Mitte eines Pixels bleibt. **Crop to fit** nimmt statt dessen das größte Rechteck in der Form des (viertelgedrehten)
  Bildes, das ganz im gedrehten Bild liegt.
- Das gedrehte Bild für den Bildbereich kommt aus dem Kern (`Session::orient_preview`, 8-Bit-RGBA); gerechnet wird in voller Größe, ein verkleinertes
  Vorschaubild gäbe dem Zuschneiden falsche Koordinaten. Eine Anfrage läuft zugleich, der neueste Wert gewinnt; das vorige gedrehte Bild bleibt stehen,
  bis das neue da ist.

**Resize Image** (von oben nach unten): Segmente **Percent · Pixels**, bei Percent die Zeile **Size (%)** mit Feld (44 px) über einem Regler, bei Pixels
die Zeilen **Width (px)** und **Height (px)** mit Feld (52 px) und das Häkchen **Lock aspect ratio**, die Zeile **Method** (Menü: Lanczos3 · Bicubic · Bilinear ·
Nearest Neighbor), die Zeile *Result* mit „W × H px“ und ein Hinweis, dass skaliert wird, wenn gespeichert wird.
Standard: Percent, 100% (keine Änderung), Lock an, Lanczos3 · Prozent 10 … 400, Seiten 1 … 16384 Pixel.

- Mit Lock bestimmt die zuletzt getippte Seite die andere (der Kern bekommt nur diese Seite und rechnet die andere nach der Form, gerundet, mindestens 1);
  ohne Lock sind beide Seiten fest, das Ausschalten hält die Größen fest, die die Felder gerade zeigen. Die Felder zeigen immer die Größe, die herauskommt.
- Die Ausgangsgröße ist der Zuschnitt (der, den man gerade zieht, sonst der angewendete) oder das ganze gedrehte Bild. Ein Wert, der die Größe nicht
  ändert (100%, oder Pixel gleich der Größe), wird nicht an den Kern gegeben.
- Skaliert wird getrennt in beide Richtungen; beim Verkleinern wird der Kern der Methode um den Faktor gestreckt, damit kein Pixel ausgelassen wird.
  Bicubic und Lanczos3 können überschwingen, das Ergebnis wird auf 0 … 1 geklemmt; ein konstantes Bild bleibt konstant, gleiche Größe lässt das Bild bitgenau.

**Dauer:** die Werte beider Karten sind Einstellungen wie die Farbwerte: sie bleiben bei einem neuen Stack und einer anderen Aufnahme (anders als der
Zuschnitt, der auf einem bestimmten Bild liegt) und stehen als Abschnitt `geometry` in der Projektdatei (nur Mac, Windows ignoriert ihn).

## Derotation (MAC-020)

**De-rotate images** (Advanced im Schritt Stack, Mac seit MAC-020, Windows seit SP-023; im Original nur als Häkchen bekannt) dreht jeden Frame vor dem Stacken auf die Mitte der
Aufnahme zurück (`docs/stacking.md`, „Derotation“). Das Häkchen ist nicht mehr gesperrt, Tooltip: „Turns every frame back to the middle of the
recording by the rotation of the planet, so that details at the edge of the disc stay sharp. Needs the date and time of the recording.“ Eingeschaltet
erscheinen darunter, **18 px eingerückt**, in dieser Reihenfolge (alle 6 px Abstand):

1. Beschriftung **Planet** (12 px, `--muted`) und die Segmentleiste **Jupiter · Saturn · Mars** (Segmente wie Limit Setup; das gewählte `--accent-soft`).
   Beim Öffnen einer Aufnahme steht der Planet vor, den der Dateiname nennt („jup“, „sat“, „mar“, jede Schreibweise); das Häkchen schaltet der Vorschlag nie ein.
2. **Start time** (Beschriftung links, rechts ein `DatePicker` im Feldstil mit Datum und Uhrzeit in der Zeitzone des Macs, Minuten genau) **nur wenn die
   Datei keine Zeit je Frame hat** (SER mit Zeitstempeln: das Feld entfällt, darunter steht „The file has a time for every frame.“). Darunter (12 px, `--muted`)
   dieselbe Zeit in UTC („2026-11-12 08:38 UTC“), bei einem Vorschlag aus dem Dateidatum dazu „ · taken from the file date, check it“; ohne Zeit steht dort
   „Enter the time of the first frame“. Intern ist es der UTC-Zeitpunkt (Sekunden seit 1970).
3. **Camera angle (°)**: Dezimalfeld mit Pfeilen, −360 … 360, eine Nachkommastelle, Standard 0 (Norden oben, Osten links); positiv gegen den Uhrzeigersinn
   auf dem Bildschirm. Tooltip nennt das.
4. Häkchen **Mirrored image** (Spiegelbild, Osten rechts bei Norden oben, etwa bei einem einzelnen Spiegel im Strahlengang). Nicht in der Vorlage des Tickets,
   aber nötig: ein Spiegelbild würde sonst in die falsche Richtung gedreht.
5. Zeile **„Span 3 min 12 s · rotation 5.8°“** (12 px, `--text`): Zeit vom ersten bis zum letzten der behaltenen Frames (`45 s`, `3 min 12 s`, `1 h 05 min`) und
   wie weit sich der Planet in ihr dreht. Dreht er sich um **weniger als 1°**, steht darunter (`--muted`) **„Too short to matter“**. Fehlt etwas (keine
   Startzeit, kein Planet im Bezugsframe, keine Bildrate), steht stattdessen der Satz des Kerns in `--accent-ink`.

Die Zahlen kommen aus dem Kern (`Session::derotation_info`), sobald Limit gelaufen ist und das Häkchen an ist; sie werden nach jeder Änderung der
Einstellungen (Planet, Camera angle, Mirrored image, Start time) und der Frame-Auswahl neu gebildet, 120 ms verzögert, damit Tippen nicht je Taste misst.

**Overlay** im Schritt Stack (vor und nach dem Stacken), solange das Häkchen an ist: die gemessene Scheibe als Ellipse in `--accent` (1,5 px), für Saturn
zusätzlich die Außenkante des Rings gestrichelt (1 px, 70 % Deckkraft), und ein kurzer Strich mit **N** über dem Nordpol des Planeten. Es liegt in Bildpixeln und
zoomt mit. Die Scheibe ist im Bezugsframe gemessen; sobald die Zahlen da sind, zeigt der Schritt Stack deshalb dieses Frame, nicht das zuletzt betrachtete.

**Projektdatei:** optionaler Abschnitt `derotation` (`enabled`, `planet`, `cameraAngle`, `mirrored`, `startTimeUtc` in Sekunden seit 1970 UTC); fehlt er oder ein
Schlüssel, bleiben die Werte; beide Apps schreiben und lesen ihn (Windows seit SP-023). Der Abschnitt `stack` trägt die Werte der Derotation nicht.

## Texte und Schreibweise (MAC-014)

Die Oberfläche soll für Einsteiger verständlich sein (Philipp, 2026-11-12). Die Zuordnung alt → neu aller Namen, die Tooltips und die
gesperrten Elemente stehen in `docs/parity/WINDOWS_PARITY.md` (Abschnitt „Verständliche Texte für Einsteiger“), die Namen je Element
in der Spalte „Name in Siderea Planets“ in `docs/ui.md`. Dieses Dokument nennt die Regeln:

- **Schreibweise nach HIG.** *Title Case* für Schaltflächen, Menüeinträge und Gruppen- oder Kartenüberschriften („Set Alignpoints“,
  „Save Project…“, „Alignpoints Used for Stacking“); *Sentence case* für Feldbeschriftungen, Häkchen, Radios, Hinweise und Tooltips
  („Min. distance between points (px)“, „Use nearest points“).
- **„…“ nur an Aktionen, die einen Dialog öffnen** (Select Recording…, Open Project…, Save Image-Dateityp „TIFF 16-bit (.tif)…“,
  Load Scheme…). Das Zeichnen einer Region (Draw Region) ist kein Dialog und trägt kein „…“.
- **Einheiten am Feld:** „(px)“, „(%)“, „(fps)“ in der Beschriftung oder dahinter; Maße mit „×“ („1024×680“, bei Feldern mit
  Leerzeichen „512 × 512 px“), Prozent ohne Leerzeichen („72%“, nicht „72 %“).
- **Statustexte:** Großbuchstabe am Anfang, kein Punkt am Ende, ein kurzer Satz („Alignment finished“); Fehler als „‹Schritt› failed:
  ‹Grund›“.
- **Ohne Wirkung = gesperrt:** ein Schalter, ein Feld oder ein Knopf, der nichts tut, ist gedimmt (45 %, wie alle gesperrten Elemente),
  nicht klickbar, sein Tooltip beginnt mit „Not available yet“; bei Optionen im Panel steht der Text zusätzlich rechts in der Zeile.
  Ein Knopf darf nicht klickbar sein, nur um „not available yet“ in der Statusleiste zu melden.
- **Kein Name des Referenzprogramms und kein „das Original“ in sichtbaren Texten** (Oberfläche, Tooltips, Dateidialoge, Statusmeldungen, Datei- und
  Projektköpfe, Hilfeseiten, About). Tooltips tragen keinen Zusatz mit dem dortigen Begriff. Vergleiche mit dem Original gehören in die
  interne Doku (`docs/`), also auch in dieses Dokument.
- **US-Englisch** überall (Color, Center, gray).
- **Tooltips auf Fachbegriffen:** Schritte, Aktionsknöpfe und Felder mit Fachwort (Alignpoints, Threshold, Limit, Wavelets, Layer,
  Denoise, Do All, Correct geometry, …) erklären sich in einem Satz; Etabliertes bleibt im Namen, die Erklärung kommt in den Tooltip.
  Die Texte stehen in Tabelle 2 der Zuordnung in `docs/parity/WINDOWS_PARITY.md`.
- **Hilfeseiten** (`app/src/docs/pages/`) nutzen dieselben Namen und haben die Seite *Glossary* für die Fachwörter.

## Bewusste Abweichungen vom Original

| Original | Klon | Grund |
|---|---|---|
| Skin „Office 2007 Silver“, hell | neutral dunkel, hell umschaltbar | SP-010; Bildbeurteilung |
| Framelist, Registrationgraph, Alignmentpoints als eigene Fenster | Karten über dem Bild | ein Fenster, nichts geht hinter anderen verloren |
| Panelköpfe mit Minimieren/Maximieren | Gruppenüberschriften | Panels sind fest angeordnet |
| Scrollbalken am Bild | Zoom-Leiste, Mausrad, Ziehen | SP-010 verlangt Zoom, Pan, Vorher/Nachher |
| grünes Farbfeld links unten | Statuspunkt mit Text | Bedeutung des Farbfelds ist ungeklärt, Text sagt sie |
| „File Version“ und Datum in der Reiterleiste | Dateiname, Codec, Größe, Frame-Zahl rechts in der Schrittleiste | Version gehört in About |
| drei Reiter Align · Stack · Wavelet, Schritt-Schaltflächen in der Aktionszeile, alle Einstellungen eines Reiters im Panel | sieben Schritte in der Schrittleiste, das Panel zeigt nur den offenen Schritt mit seiner Aktion, Seltenes unter Advanced | SP-014; Philipp wollte, dass die App durch den Ablauf führt (Vorbild Siderea) |
| Häkchen „Show …“ in der Aktionszeile | Menü Show oben rechts am Bild | SP-014; die Aktionszeile entfällt |
| Colour und LRGB im Align- und Stack-Reiter | im Schritt Wavelets (Color und Sharpen brightness only) | SP-014; sie wirken auf die Schärfung (SP-007) |
| „Save image“ rot beschriftet | Standard-Schaltfläche | Rot bedeutet Gefahr |
| Radiogruppen bei zwei Möglichkeiten | Segmente | weniger Höhe, gleiche Stelle |
| „Intensity_selection“, „Realign_with Processed“ | ohne Unterstrich | Artefakte der Delphi-Beschriftung |
| „Step Increment“, „Initial Layer“ | Windows-Entwurf: „Step Incr.“ mit vollem Namen im Tooltip; in der App seit MAC-014 „Size step“ und „First layer size“ | Platz in einer Zeile; verständlichere Namen |
| Set Alignpoints und Align ab Align gesperrt | bleiben bedienbar, ein neuer Lauf verwirft Limit, Stack und Wavelet-Ergebnis | SP-005; Philipp wollte ohne erneutes Öffnen neu ausrichten können, siehe [Zustände](#zustände) |
| Dezimalkomma je nach Windows-Einstellung | Punkt | einheitlich auf allen Plattformen |
| CPUs: Standard 1 | Standard = Zahl der logischen Kerne, Bereich 1 bis dahin; die Zahl wirkt auf alle Berechnungen (eigener Thread-Pool im Backend) und wird nicht in der Projektdatei gespeichert | SP-010; Philipp (2026-11-12): die Auswahl soll wirklich wirken und der Rechner standardmäßig ausgelastet sein; die Kernzahl gehört zum Rechner, nicht zum Projekt |
| » klappt das Functions-Panel | „Functions »“ oben im Bildbereich (in der App seit MAC-014: „Adjust ›“) | Beschriftung sagt, was geklappt wird |

## Mac-App: bewusste Abweichungen

Die Mac-App (`mac/App`, SwiftUI) übernimmt Tokens, Fensteraufbau, Schrittleiste, Panels, Bildbereich, Frame- und Statusleiste dieses
Dokuments. Bedienelemente und Eingaben folgen dort, wo es eine Mac-Konvention gibt, dieser statt dem Entwurf:

| Entwurf (Windows) | Mac-App | Grund |
|---|---|---|
| Werkzeugleiste als eigene Leiste, 44 px, Fenstertitel nur der des Systems | Native Werkzeugleiste im Fenster (Select Recording, Recent Recordings, in der Mitte Cancel und Pause); Titel `Siderea Planets — ‹Datei›` in der Titelleiste | Mac-Fensteraufbau; Open und Save (Projekt) stehen links neben Cancel und Pause, Help und CPUs kommen mit MAC-010 |
| Select und MRU nur in der Werkzeugleiste | zusätzlich **⌘O** „Select Recording…“ (bis MAC-014 „Open…“) und **Datei > Open Recent** (mit „Clear Menu“) | Menüleiste und ⌘-Kürzel (Mac-Konvention) |
| Dateidialog des Betriebssystems | `NSOpenPanel`, Mehrfachauswahl; die Dateien einer Bildserie gehen nach Namen sortiert an den Kern | nativ |
| Mausrad zoomt, Ziehen verschiebt | Pinch auf dem Trackpad zoomt um die Finger, **⌥ oder ⌘ + Scrollen** zoomt um den Zeiger, einfaches Scrollen und Ziehen verschieben, Zwei-Finger-Doppeltipp schaltet zwischen Fit und 100 % | Mac-Gesten; ein Mausrad ohne Taste zoomt nicht |
| Zoom-Leiste − · Prozent · + · Fit | gleich, dazu **⌘+ ⌘- ⌘0** (Fit) und **⌘1** (100 %) im Menü View (auf einem deutschen Mac „Darstellung“) | Mac-Kürzel |
| Frame-Leiste: Goto Frame und Regler | zusätzlich Schaltflächen zum vorigen und nächsten Frame sowie **⌘[** und **⌘]** | schnelles Blättern ohne Regler |
| Cancel und Pause nur als Schaltflächen der Werkzeugleiste | zusätzlich Menü **Run** (bis MAC-014 „Process“) mit **Cancel (⌘.)** und **Pause/Resume**; Cancel ist in der Werkzeugleiste gefüllt (`--accent`), solange Align, Stack, Process, Do All oder Create AVI läuft; beide wirken nur auf diese Läufe (Set Alignpoints und Limit sind in einem Augenblick fertig) | Mac-Kürzel ⌘. für „abbrechen“ |
| Show ▾ mit Häkchen als eigenes Menü, Radiobuttons und Zahlenfelder mit eigenen Pfeiltasten | natives Menü (Häkchen links), Radiobuttons als eigene Zeilen in `--accent`, Zahlenfeld mit nativem `Stepper`; der Threshold-Regler färbt die Spur links vom Griff in `--accent` | Mac-Steuerelemente; ein `Slider` ohne Einfärbung gibt es in SwiftUI nicht |
| Thema in Settings > Theme umschaltbar | folgt der Darstellung von macOS (hell und dunkel), kein Theme-Eintrag | Systemeinstellung gilt |
| Felder, Regler, Stepper, Menüs der Oberfläche | Systemschrift (SF) in 13 und 12 pt, native Steuerelemente (`Stepper`, `Slider`, `TextField`) mit Akzent `--accent`; eigene Gestaltung nur dort, wo dieses Dokument sie festlegt (Pillen, Schaltflächen, Zoom-Leiste, Statusleiste) | Mac-Steuerelemente, Fokus und Bedienungshilfen kommen vom System |
| – | Eine Datei aus dem Finder auf das Fenster ziehen öffnet sie | Mac-Konvention |
| Save image nur als Schaltfläche mit Menü der Dateitypen; Open und Save (Projekt) nur in der Werkzeugleiste | zusätzlich im Menü File: **Save Image… (⌘S)** im zuletzt gewählten Dateityp (Start: TIFF 16 bit; bietet der Schritt ihn nicht an, TIFF), **Save Project…** und **Open Project…**; Speichern-Dialog `NSSavePanel` mit dem gewählten Typ, Öffnen-Dialog `NSOpenPanel` für `.sidereaplanets`; Save Aligned Video… und Save Averaged Video… (Original: Save Aligned und Save Integrated) laufen wie Stack mit Fortschritt, Cancel (⌘.) und Pause | Mac-Konvention ⌘S und native Dateidialoge |
| Help in der Werkzeugleiste, Doku-Ansicht mit Seitenliste und Suche | Menü **Help > Siderea Planets Help (⌘?)** öffnet dasselbe Help-Fenster (1000 × 720 pt, mindestens 640 × 400, zentriert, `Window`-Szene: ein zweiter Aufruf holt es nach vorn, Escape oder Schließen beendet es). Die Seiten sind dieselben Markdown-Dateien (`app/src/docs/pages/`, beim Bau ins Bündel kopiert); die Ansicht ist eine `WKWebView` mit dem Stil aus „Doku“, die Seitenliste mit Suchfeld ist SwiftUI. Keine Schaltfläche Help in der Werkzeugleiste | Mac-Menü; die Doku bleibt gemeinsam |
| About in der Werkzeugleiste (im Original nicht vorhanden, in Windows noch „Not available yet“) | **About Siderea Planets** im App-Menü (Standard-Infofenster mit Version aus `MARKETING_VERSION`) | Mac-Konvention |
| CPUs-Feld in der Werkzeugleiste, Settings als Menü der Werkzeugleiste | **Siderea Planets > Settings… (⌘,)**: Fenster mit dem Feld CPUs (Zahlenfeld mit `Stepper`, 1 bis zu den logischen Kernen, Standard alle, nicht gespeichert; die Änderung gilt für die nächste Berechnung). Dazu der Abschnitt Wavelets mit **Preview area** (Original: Processing Area; 2048 bis 128, Standard 512, „512 × 512 px“) und **Load Scheme… / Save Scheme…** (`.rwv`, dasselbe Dateiformat wie Windows; die Schaltflächen im Advanced des Schritts Wavelets tun dasselbe). Die übrigen Einträge des Windows-Menüs Settings (Automatic Processing, Luminance, Stretch FIT/TIFF, …) kommen mit dem Schritt, der sie braucht; Theme entfällt (macOS-Darstellung) | Mac-Konvention |
| Regler der Ebenen, Contrast/Brightness, Gamma und RGB Balance: Spur ohne Einfärbung, Zahlenfelder mit eigenen Pfeiltasten | native `Slider` mit `Stepper`; die Spur links vom Griff ist in `--accent` gefärbt (wie beim Threshold), die Felder haben 2, 3, 1 oder 0 Nachkommastellen wie im Entwurf, der Punkt ist immer der Dezimaltrenner | Mac-Steuerelemente |
| Vollbild gibt es nicht | **View > Enter Full Screen (⌃⌘F)** und die grüne Fenstertaste | Mac-Konvention |
| Schritte nur über die Schrittleiste | zusätzlich **Run > Go to Step** (bis MAC-014 „Process > Go to Step“) mit **⌥⌘1** bis **⌥⌘7** (gesperrte Schritte und eine laufende Berechnung sperren den Eintrag wie die Schrittleiste) | Tastaturbedienung; ⌘1 ist Actual Size |

Die Dateiangaben im Schritt Open zeigen auf dem Mac zusätzlich zu Dateiname, Codec, Größe und Frame-Zahl auch Format, Bittiefe,
Farbe, Bildrate und (wo die Datei sie hat) Beobachter, Instrument und Teleskop; Windows zieht das nach
(`docs/parity/WINDOWS_PARITY.md`).

## App-Icon

Das Icon gehört zur Familie mit Siderea (gleicher Hintergrund, gleiche drei gestapelten Rahmen mit Orange-Kontur) und zeigt
das Thema des Programms: ein Planet im vorderen Rahmen, **links unscharf, rechts scharf mit Bändern**, dazwischen eine dünne helle
Trennlinie, dahinter ein oranger Schein. Mac und Windows tragen dasselbe Bild.

| Ebene (`docs/design/icon/AppIcon.icon/Assets`) | Inhalt |
|---|---|
| `frame-back.svg`, `frame-middle.svg` | hinterer und mittlerer Rahmen, wie bei Siderea |
| `frame-front.svg` | vorderer Rahmen mit vier blassen Sternen |
| `planet-glow.svg` | oranger Schein hinter dem Planeten |
| `planet-blur.png` | der unscharfe Planet. Icon Composer ignoriert SVG-Filter, deshalb wird die Unschärfe aus `src/planet-blur.svg` in ein PNG gerechnet |
| `planet-sharp.svg` | rechte Planetenhälfte mit Bändern und die Trennlinie |

Alle SVG-Ebenen liegen auf dem 824-Einheiten-Raster des Mac-Icons (`transform="scale(1.242718) translate(-100 -100)"`).
`AppIconDev.icon` ist dieselbe Ebenenfolge mit einem orangen **DEV-Abzeichen**: Der Debug-Build der Mac-App trägt es
(`ASSETCATALOG_COMPILER_APPICON_NAME[config=Debug]` in `mac/project.yml`), damit er im Dock nicht mit der installierten App
verwechselt wird. Die Windows-App hat kein Abzeichen.

**Neu erzeugen** nach einer Änderung an den Ebenen: `scripts/make-icons.sh`. Es rechnet das PNG der Unschärfe, gleicht
`AppIconDev.icon` an, setzt `AppIcon-1024.png` zusammen (Quadrat mit abgerundeten Ecken, 40 px Rand) und schreibt über
`pnpm tauri icon` den Windows-Satz nach `app/src-tauri/icons/`. Die Größen 16, 24 und 32 px der `icon.ico` (und `32x32.png`)
kommen aus der vereinfachten `src/AppIcon-small.svg`: nur der vordere Rahmen, dicker Rand, größerer Planet, kräftigere Trennlinie.
Die Mac-App braucht keinen Schritt, `xcodegen` und `xcodebuild` bauen die `.icon`-Ordner selbst.

## Was im Entwurf nur Platzhalter ist

- Das Jupiter-Bild ist gezeichnet. Im Klon steht dort das Frame bzw. das Summenbild.
- Die Schärfung beim Bewegen der Ebenen-Regler ist ein einfacher Unscharf-Filter, nicht der Wavelet-Algorithmus.
- Qualitätswerte, Punktpositionen, Speicherangaben und Zeiten sind erfunden.
- Die Functions-Schaltflächen schalten nur ihren gedrückten Zustand um.

## Noch nicht festgelegt

Für diese Teile gibt es kein Soll-Bild. Wer sie braucht, baut sie aus den vorhandenen Komponenten und ergänzt
dieses Dokument im selben Commit.

- Zustand vor dem Laden (festgelegt in SP-010, angepasst in SP-014): Schritt Open ist offen, Select im Fuß des Panels
  gefüllt, alle anderen Pillen gesperrt, keine Frame-Leiste. Der Bildbereich zeigt mittig „Press Select Recording to open a video or a series of images“ in `--on-canvas` mit 60 % Deckkraft.
- Fehlermeldungen (festgelegt in SP-010): Text mit Präfix „Error:“ im Ergebnisfeld der Statusleiste (Feld 3), in
  `--text`; kein Dialog, kein Rot.
- Speicherangabe („Memory used/free/total“) in der Schrittleiste: entfällt vorerst, sie braucht einen Befehl im Backend.
- Neun der fünfzehn Panels hinter den Functions-Schaltflächen (in der App: Adjustments; die Schaltflächen sind gesperrt). Histogram, Gamma, Color Mixing, RGB Align und RGB Balance stehen unter
  [Karten hinter den Functions-Schaltflächen](#karten-hinter-den-functions-schaltflächen); die übrigen sind im Original
  noch nicht erfasst (`docs/offene-fragen.md`). Vorgabe für die Form: schwebende Karten wie Framelist.
- Stackgraph, Stacklist, Aligndata und Prefilter (in der App: Stack Graph, Stack List, Align Data, Prefilter; im Menü Show gesperrt). Vorgabe für die Form: schwebende Karten.
- MACRO/Batch-Fenster und About-Dialog. Ihre Schaltflächen sind angelegt und melden „not available yet“.
- Dateidialoge sind die des Betriebssystems und brauchen kein Design.

## Zuordnung zu den Tickets

| Ticket | Betroffene Teile der Oberfläche | Soll-Bilder |
|---|---|---|
| SP-004 Eingabe | Select, MRU, Angaben zur Datei in der Reiterleiste, Frame-Leiste | 01 |
| SP-005 Ausrichtung | Align-Reiter komplett, Framelist, Registrationgraph, Fortschritt | 01–04, 11 |
| SP-006 Stacken | Stack-Reiter komplett, Karte Alignmentpoints | 05, 06 |
| SP-007 Wavelets | linkes Panel im Wavelet-Reiter, Process, Do All, Before, Processing Area | 07, 08, 12 |
| SP-008 Farbe und Tonwerte | rechtes Panel im Wavelet-Reiter (Functions, Contrast/Brightness), Colour/LRGB | 07, 12 |
| SP-009 Speichern | Save image in Stack und Wavelet, Save Aligned, Save Integrated, Load/Save Scheme | 06, 07 |
| SP-010 Oberfläche | alles in diesem Dokument außer der Doku | 01–12 |
| SP-012 Doku in der App | Help in der Werkzeugleiste, Doku-Ansicht | 13, 14 |
| MAC-017 Drehen und Größe | Karten Flip and Rotate und Resize Image, Gruppe Size im Schritt Save | – |
| MAC-020 Derotation | De-rotate images mit Planet, Start time, Camera angle, Mirrored image im Schritt Stack, Overlay der gemessenen Scheibe | – |
| SP-014 Geführter Ablauf | Schrittleiste, Schrittpanel, Show am Bild, Qualitätsdiagramm in Limit; ersetzt die Reiter in den Zeilen darüber (Bildnummern dort sind die alten) | 01–12, 15, 16 |

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
