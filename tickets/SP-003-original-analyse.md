# SP-003 Original analysieren und Spezifikation anlegen

- Status: erledigt
- Abhängig von: Installation des Referenzprogramms durch Philipp

## Ziel
Das Original dokumentieren, damit der Klon nachprüfbar gebaut werden kann.

## Akzeptanzkriterien
- [x] `docs/workflow.md`: Ablauf vom Laden bis Speichern, jeder Reiter und jeder Schritt
- [x] `docs/ui.md`: Oberfläche (Screenshots nur in `re-local/`), Reglernamen, Wertebereiche, Standardwerte
- [x] `docs/formats.md`: unterstützte Ein-/Ausgabeformate (AVI, SER, Bilder, Einstellungsdateien)
- [x] Testdatensatz (kurzes Planeten-/Mondvideo) liegt in `re-local/`; erwartete Ergebnisse sind festgehalten
- [x] Offene Fragen und Vermutungen sind getrennt aufgelistet

## Ergebnis
- Spezifikation: `docs/workflow.md`, `docs/ui.md`, `docs/formats.md`, `docs/testdaten.md`, `docs/offene-fragen.md`.
- Wertebereiche: Für die Schieberegler sind sie erfasst. Die Zahlenfelder des Originals begrenzen Eingaben gar nicht;
  ihre Grenzen bleiben offen (`docs/offene-fragen.md`, Punkt 1).
- Referenzfall R1 (Jupiter, Standardwerte) mit Summenbild und zwei Wavelet-Ergebnissen liegt in `re-local/reference/`,
  Screenshots und Steuerelement-Listen in `re-local/screenshots/`.
- Folgepunkte: Die Panels unter „Functions“ gehören zu SP-008. SP-006 erwähnt einen Optimierungsschritt, den
  Version 6 des Referenzprogramms nicht mehr hat (`docs/offene-fragen.md`, Punkt 17).
- Hinweis zur Bedienung per Skript: Das normal gestartete Referenzprogramm lief mit erhöhten Rechten und nahm keine
  simulierten Eingaben an. Gestartet mit `__COMPAT_LAYER=RunAsInvoker` lässt es sich bedienen. Die Einstellungen
  landen dann im VirtualStore des Benutzers.

## Testdaten (lokal, nicht im Repo)
- `re-local/testdata/astropix/` — `CLAVIUS.AVI`, `JUPITER.AVI` (je ~21 MB), `SUNSPOTS.AVI` (~9 MB), Übungsvideos aus dem Buch
  „Guide to Digital Planetary Imaging“ von astropix.com. Nur zum Lernen/Testen, nicht weitergeben und nicht committen.
  Quelle: https://www.astropix.com/books/GDPI/CHAPTERZ/Z04.HTM
- Sind AVI, keine SER: Für den Klon braucht es dafür den AVI-Leser aus SP-004.
