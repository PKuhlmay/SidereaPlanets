# RK-004 Video- und Bild-Eingabe

- Status: offen
- Abhängig von: RK-001, RK-003

## Ziel
Aufnahmen laden, wie es astronomische Kameras liefern.

## Akzeptanzkriterien
- SER-Dateien (inkl. Bayer-Muster, 8/16 Bit) werden gelesen
- AVI (unkomprimiert, gängige Codecs) und Einzelbild-Serien (BMP/PNG/TIFF/FITS) werden gelesen
- Frames werden gestreamt, nicht komplett im Speicher gehalten
- Unit-Tests mit kleinen, selbst erzeugten Beispieldateien
