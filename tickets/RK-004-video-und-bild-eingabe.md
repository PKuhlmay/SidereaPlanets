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

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: **Select**, **MRU**, Angaben zur Datei in der Reiterleiste, Frame-Leiste (`docs/design.md`, Abschnitte Fensteraufbau und Frame-Leiste)
- Soll-Bild: `docs/design/screenshots/01-align-loaded.png`
- Der Zustand vor dem Laden und die Anzeige von Fehlern sind dort noch nicht festgelegt (Abschnitt „Noch nicht festgelegt“)
