# RK-004 Video- und Bild-Eingabe

- Status: in Arbeit (SER und unkomprimiertes AVI fertig; komprimierte AVI-Codecs, Bildserien und FITS offen)
- Abhängig von: RK-001, RK-003

## Ziel
Aufnahmen laden, wie es astronomische Kameras liefern.

## Akzeptanzkriterien
- SER-Dateien (inkl. Bayer-Muster, 8/16 Bit) werden gelesen
- AVI unkomprimiert (8 Bit mit Palette, 24/32 Bit, OpenDML-Segmente) ist fertig; komprimierte Codecs (z. B. MJPEG, HuffYUV, Lagarith) offen, erst wenn echte Aufnahmen sie brauchen
- AVI (gängige Codecs) und Einzelbild-Serien (BMP/PNG/TIFF/FITS) werden gelesen
- Frames werden gestreamt, nicht komplett im Speicher gehalten
- Unit-Tests mit kleinen, selbst erzeugten Beispieldateien
