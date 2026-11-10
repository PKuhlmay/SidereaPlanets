# RK-003 Original analysieren und Spezifikation anlegen

- Status: offen
- Abhängig von: Installation von the reference program durch Philipp

## Ziel
Das Original dokumentieren, damit der Klon nachprüfbar gebaut werden kann.

## Akzeptanzkriterien
- [ ] `docs/workflow.md`: Ablauf vom Laden bis Speichern, jeder Reiter und jeder Schritt
- [ ] `docs/ui.md`: Oberfläche (Screenshots nur in `re-local/`), Reglernamen, Wertebereiche, Standardwerte
- [ ] `docs/formats.md`: unterstützte Ein-/Ausgabeformate (AVI, SER, Bilder, Einstellungsdateien)
- [ ] Testdatensatz (kurzes Planeten-/Mondvideo) liegt in `re-local/`; erwartete Ergebnisse sind festgehalten
- [ ] Offene Fragen und Vermutungen sind getrennt aufgelistet

## Testdaten (lokal, nicht im Repo)
- `re-local/testdata/astropix/` — `CLAVIUS.AVI`, `JUPITER.AVI` (je ~21 MB), `SUNSPOTS.AVI` (~9 MB), Übungsvideos aus dem Buch
  „Guide to Digital Planetary Imaging“ von astropix.com. Nur zum Lernen/Testen, nicht weitergeben und nicht committen.
  Quelle: https://www.astropix.com/books/GDPI/CHAPTERZ/Z04.HTM
- Sind AVI, keine SER: Für den Klon braucht es dafür den AVI-Leser aus RK-004.
