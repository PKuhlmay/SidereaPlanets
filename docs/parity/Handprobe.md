# Handprobe Mac und Windows

Was der Build nicht prüfen kann: Philipp probiert es in der jeweiligen App aus und trägt das Ergebnis ein.

- `[ ]` noch nicht probiert
- `[x]` klappt
- `[!]` klappt nicht — dahinter kurz, was passiert (`[!] Zoom springt beim Schrittwechsel`)

Ein Punkt mit `[x]` fliegt beim nächsten Aufräumen (`/nachzug`) raus, ein `[!]` wird zum Fix (Ticket oder direkt), und erst der Fix
löscht ihn. Neue Tickets tragen ihre Probe hier ein, wenn der Commit, der sie abschließt, entsteht. Abschnitte nach Plattform und
Ticket, Details im Ticket unter „Umsetzung“.

## Mac

### MAC-001 Mac-App aufbauen (v0.1.0)

- [ ] Ganzer Ablauf mit JUPITER.AVI: Select, Set Alignpoints, Align, Limit, Stack, Do All, Save image (TIFF 16 Bit), Datei öffnet.
- [ ] Align mit Pause und Cancel bei einem langen Lauf; Balken und Frame-Zähler laufen.
- [ ] Zoom per Trackpad und Mausrad, Verschieben, 100 %-Klick, Vorher/Nachher.
- [ ] Open mit SER, mit einer Bildserie (mehrere TIFF/PNG zusammen markieren, Reihenfolge nach Name) und mit FITS; Datei aus dem Finder
  auf das Fenster ziehen; „Open Recent“ (auch „Clear Menu“) und MRU in der Werkzeugleiste.
- [ ] Bildbereich: Pinch auf dem Trackpad zoomt um die Finger, ⌥+Scrollen und ⌘+Scrollen zoomen um den Zeiger, Scrollen verschiebt,
  Zwei-Finger-Doppeltipp wechselt zwischen Fit und 100 %; ⌘+ ⌘- ⌘0 ⌘1 und ⌘[ ⌘] per Tastatur (Menü View).
- [ ] Help (⌘?) öffnet ein eigenes Fenster getrennt vom Hauptfenster (ein zweiter Aufruf holt es nach vorn, Escape schließt es); Seitenliste, Suche (Eingabe in das Feld, Treffer in der Liste, Klick öffnet), Links zwischen Seiten und Blättern unten; Tabellen, Zahlenkreise und Hinweise sehen in hell und dunkel aus wie in `docs/design/screenshots/13-docs.png` und `14-docs-light.png`.
- [ ] Menüleiste und ⌘-Kürzel (Open ⌘O, Cancel ⌘., Help ⌘?, Settings… ⌘,, About im App-Menü, Enter Full Screen im Menü View, Process > Go to Step mit ⌥⌘1 bis ⌥⌘7) wirken; Save image ⌘S, Save project und Open project kommen mit dem Schritt Save.
- [ ] Settings… (⌘,): Fenster mit dem Feld CPUs; Standard sind alle Kerne; Zahl eintippen und mit Return bestätigen, 99 wird zur Kernzahl, 0 zu 1; mit 1 ist ein langer Align deutlich langsamer (Activity Monitor: nur ein Kern ausgelastet).
- [ ] Release-Bündel (`scripts/build-mac-release.sh`, `build/release/Siderea-Planets-<Version>-mac.zip`) auf einem anderen Mac: Rechtsklick > Öffnen startet es (ad hoc signiert, nicht notarisiert), Help zeigt die Seiten, Intel-Mac und Apple-Silicon-Mac laufen beide.
- [ ] Schritt Wavelets (MAC-008): Regler einer Ebene ziehen, die Vorschau in der Processing Area folgt flüssig (Statusleiste „Preview: n ms“); Use Linked
  Wavelets nimmt alle Regler mit; Process; Do All mit Fortschritt, Pause und Cancel (⌘.), danach öffnet Save; Before schaltet auf den Stack zurück; Colour
  und LRGB (LRGB nur mit Colour); Functions » blendet das rechte Panel aus; Histogram (Dreiecke ziehen, Auto), Gamma, RGB Align (Auto) und RGB Balance
  verändern das Bild sofort, Contrast/Brightness, Copy To, Load to, Difference; Load Scheme und Save Scheme (Advanced und Settings…) mit einer `.rwv`-Datei von Windows.
- [ ] Helles und dunkles Thema, auch beim Umschalten der macOS-Darstellung.

### MAC-021 Align by center of gravity

- [ ] Eine Planetenaufnahme mit starker Drift (oder ein Video, in dem der Planet weit springt): Align ohne das Häkchen und mit ihm vergleichen,
  Show Quality Graph und das Stackergebnis ansehen. Brightness threshold ist nur mit dem Häkchen bedienbar; der Wert steht nach Save Project und
  erneutem Öffnen wieder da. Mondaufnahme mit dem Häkchen: Align fällt nicht aus, auch wenn das Ergebnis schlechter sein darf.

## Windows

### MAC-003 Sitzungslogik im Kern

- [ ] Ganzer Ablauf im Fenster nach dem Umzug der Befehle in `crates/session` (`pnpm tauri dev` am Windows-PC): JUPITER.AVI, Select, Set
  Alignpoints, Align, Limit, Stack, Do All, Save image (TIFF 16 Bit), Datei öffnet. Dazu Align und Stack mit Pause und Cancel (Balken,
  Frame-Zähler, Meldung „cancelled“, danach läuft ein neuer Lauf), Set Alignpoints oder Align erneut (Stack und Wavelet-Bild sind weg,
  Save image meldet „there is no stacked image yet“) und das CPUs-Feld (1 Kern ist bei einem langen Align deutlich langsamer).

### RK-016 Texte für Einsteiger (windows-v0.1.0)

- [ ] Ganzer Ablauf mit JUPITER.AVI in `pnpm tauri dev`: jede Pille, jeder Aktionsknopf und jedes Feld zeigt den neuen Namen und einen
  Tooltip (Stichprobe gegen die Mac-App); Statusleiste nach Align „Alignment finished“, nach Keep Best Frames „8 frames will be stacked ·
  weakest kept: 72%“, nach Stack „Stacked 10 frames · 1024×680“ und „Value range 0–255“.
- [ ] Werkzeugleiste bei kleinster Fenstergröße (1120 px): alles sichtbar, CPUs-Feld nicht abgeschnitten; Settings zeigt oben Preview Area,
  Theme, Show Advanced Controls und darunter die gedimmten Einträge.
- [ ] Eine AVI und mehrere TIFFs aus dem Explorer auf das Fenster ziehen: die Aufnahme öffnet; während Align läuft, passiert nichts.
- [ ] Gesperrte Elemente (Show-Menü unten, Adjustments-Knöpfe ohne Karte, Keep, Clear List, Stack Again, Codec, About) sind gedimmt und
  nicht klickbar, Tooltip „Not available yet“.
- [ ] Ein Fehler (z. B. eine Textdatei öffnen) erscheint als „Open failed: …“.

### RK-017 Dateiangaben und Navigation (windows-v0.1.0)

- [ ] JUPITER.AVI und eine SER-Datei öffnen, dann den Schritt Open anklicken: die Liste zeigt Format, Codec, Size, Bit depth, Frames,
  Color, Frame rate (bei SER auch Observer, Instrument, Telescope, wenn die Datei sie hat); Tooltips auf Codec und Bit depth.
- [ ] Frame-Leiste: die Pfeile blättern und sind am ersten bzw. letzten Frame gesperrt; ← und → tun dasselbe, aber nicht, solange das
  Feld Go to frame oder ein Zahlenfeld des Panels den Fokus hat.
- [ ] Alt+1 bis Alt+7 öffnen die Schritte wie ein Klick; ein grauer Schritt öffnet nicht, während Align läuft öffnet keiner; das
  Fenstermenü von Windows erscheint dabei nicht.

### RK-018 Go to Best Frame (windows-v0.1.0)

- [ ] JUPITER.AVI: vor Align ist Go to Best Frame gesperrt (Tooltip „Run Align first“). Nach Align zeigt ein Klick im Panel Best Frames
  und in der Frame List den obersten Frame der Liste, die Frame-Leiste steht auf ihm, Alignpoints öffnet mit dem Hinweis „Best frame: N. …“.
- [ ] Danach Keep Best Frames und Stack sind noch erledigt (nichts verworfen); erst Set Alignpoints Again löscht Hinweis und Ergebnisse.

### RK-019 Auslastung in der Statusleiste (windows-v0.1.0)

- [ ] Links in der Statusleiste steht „CPU (n)“ mit n = Zahl der logischen Kerne und „RAM“; bei einem langen Align mit allen Kernen
  steigt die CPU auf über 90 % und wird fett und amber; der RAM-Wert passt grob zum Working Set im Task-Manager.
- [ ] Settings > Show CPU and Memory Usage aus: das Feld verschwindet; nach einem Neustart bleibt es aus. Minimiertes Fenster: im
  Task-Manager kaum CPU-Last der App.

### RK-020 Color Mixing, Denoise/Deringing und Masking (windows-v0.1.0)

- [ ] Nach Do All: Color Mixing mit dem Preset *Swap Red and Blue* tauscht die Farben sofort, Reset stellt zurück; Preserve brightness
  hält die Helligkeit; Save Image schreibt das gemischte Bild.
- [ ] Denoise/Deringing: Denoise glättet das Rauschen ohne den Planetenrand zu verwischen; Deringing hebt den dunklen Ring am Rand.
- [ ] Masking: Use mask mit Show mask färbt den geschärften Teil blau; Schwelle und Feather verändern ihn; Before oder Remembered image
  zeigen das Bild ohne Maske; Karte schließen nimmt die Maske weg.
- [ ] Save Project und wieder öffnen: Matrix und die Werte beider Karten stehen wieder da (Show mask nicht).

### RK-021 View Compare und Zuschneiden (windows-v0.1.0)

- [ ] Nach Do All: View Compare teilt das Bild, der Griff zieht die Linie (auch mit ← und → nach einem Klick auf den Griff); Zoomen und
  Verschieben halten die Linie fest; Before/After beendet den Vergleich und umgekehrt.
- [ ] Crop (Adjustments oder Crop… im Schritt Save): Rechteck auf 80 % der Mitte; Ziehen innen verschiebt, an Kante und Ecke ändert die
  Größe (Mauszeiger passend), außen zeichnet ein neues; 1:1, 3:2 und 4:3 halten die Form; View Compare ist solange gesperrt.
- [ ] Done, dann Save Image als TIFF: die Datei hat genau die angezeigte Größe; Remove Crop schreibt wieder das ganze Bild; ein neuer Stack
  löscht den Zuschnitt.
