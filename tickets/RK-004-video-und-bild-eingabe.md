# RK-004 Video- und Bild-Eingabe

- Status: in Arbeit (SER und unkomprimiertes AVI fertig; komprimierte AVI-Codecs, Bildserien und FITS offen)
- Abhängig von: RK-001, RK-003

## Ziel
Aufnahmen laden, wie es astronomische Kameras liefern.

## Akzeptanzkriterien
- [x] SER-Dateien (8–16 Bit, Mono, RGB/BGR, Bayer als Rohdaten mit Muster-Kennung) werden gelesen
- [x] AVI unkomprimiert (8 Bit mit Palette, 24/32 Bit, OpenDML-Segmente) wird gelesen
- [ ] AVI mit komprimierten Codecs (z. B. MJPEG, HuffYUV, Lagarith) wird gelesen
- [ ] Einzelbild-Serien (BMP/PNG/TIFF/FITS) werden gelesen
- [x] Frames werden gestreamt, nicht komplett im Speicher gehalten
- [x] Unit-Tests mit kleinen, selbst erzeugten Beispieldateien
