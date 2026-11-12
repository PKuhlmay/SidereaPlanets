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
- [ ] Help-Fenster öffnet getrennt vom Hauptfenster; Suche und Blättern.
- [ ] Menüleiste und ⌘-Kürzel (Open, Save image, Cancel, Help) wirken.
- [ ] CPUs-Feld: Standard sind alle Kerne; mit 1 ist ein langer Align deutlich langsamer.
- [ ] Helles und dunkles Thema, auch beim Umschalten der macOS-Darstellung.

## Windows

### MAC-003 Sitzungslogik im Kern

- [ ] Ganzer Ablauf im Fenster nach dem Umzug der Befehle in `crates/session` (`pnpm tauri dev` am Windows-PC): JUPITER.AVI, Select, Set
  Alignpoints, Align, Limit, Stack, Do All, Save image (TIFF 16 Bit), Datei öffnet. Dazu Align und Stack mit Pause und Cancel (Balken,
  Frame-Zähler, Meldung „cancelled“, danach läuft ein neuer Lauf), Set Alignpoints oder Align erneut (Stack und Wavelet-Bild sind weg,
  Save image meldet „there is no stacked image yet“) und das CPUs-Feld (1 Kern ist bei einem langen Align deutlich langsamer).
