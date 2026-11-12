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
- [ ] Release-Bündel (`scripts/build-mac-release.sh`, `build/release/SidereaPlanets-<Version>-mac.zip`) auf einem anderen Mac: Rechtsklick > Öffnen startet es (ad hoc signiert, nicht notarisiert), Help zeigt die Seiten, Intel-Mac und Apple-Silicon-Mac laufen beide.
- [ ] Helles und dunkles Thema, auch beim Umschalten der macOS-Darstellung.

## Windows

### MAC-003 Sitzungslogik im Kern

- [ ] Ganzer Ablauf im Fenster nach dem Umzug der Befehle in `crates/session` (`pnpm tauri dev` am Windows-PC): JUPITER.AVI, Select, Set
  Alignpoints, Align, Limit, Stack, Do All, Save image (TIFF 16 Bit), Datei öffnet. Dazu Align und Stack mit Pause und Cancel (Balken,
  Frame-Zähler, Meldung „cancelled“, danach läuft ein neuer Lauf), Set Alignpoints oder Align erneut (Stack und Wavelet-Bild sind weg,
  Save image meldet „there is no stacked image yet“) und das CPUs-Feld (1 Kern ist bei einem langen Align deutlich langsamer).
