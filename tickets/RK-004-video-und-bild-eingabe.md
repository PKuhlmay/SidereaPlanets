# RK-004 Video- und Bild-Eingabe

- Status: erledigt
- Abhängig von: RK-001, RK-003

## Ziel
Aufnahmen laden, wie es astronomische Kameras liefern.

## Akzeptanzkriterien
- [x] SER-Dateien (8–16 Bit, Mono, RGB/BGR, Bayer als Rohdaten mit Muster-Kennung) werden gelesen
- [x] AVI unkomprimiert (8 Bit mit Palette, 24/32 Bit, OpenDML-Segmente) wird gelesen
- [x] AVI mit komprimierten Codecs (z. B. MJPEG, HuffYUV, Lagarith) wird gelesen
- [x] Einzelbild-Serien (BMP/PNG/TIFF/FITS) werden gelesen
- [x] Frames werden gestreamt, nicht komplett im Speicher gehalten
- [x] Unit-Tests mit kleinen, selbst erzeugten Beispieldateien

## Ergebnis
- Eigene Dekoder für `MJPG`, `HFYU`/`FFVH` und `LAGS` unter `crates/io/src/codec/`; Bildserien in `images.rs`, FITS in `fits.rs`.
  Was genau unterstützt ist und was abgelehnt wird, steht in `docs/formats.md`.
- Tests: Codec-Fixtures aus ffmpeg in `crates/io/tests/data/` (Erzeugung in `generate.sh`); Lagarith gegen die
  FFmpeg-FATE-Beispiele, die nur lokal in `re-local/testdata/codecs/lagarith/` liegen (Pfad über `PLANETS_TESTDATA`).
- Die App öffnet Aufnahmen über den Dateidialog des Systems (Select), Mehrfachauswahl wird zur Bildserie.
- Nicht enthalten: MPEG-Eingabe, interlaced MJPEG, HuffYUV 1.x ohne Tabellen, Lagarith „reduced resolution“.
- **Offen: Anwenderdoku in der App.** Die Doku-Ansicht gibt es bisher nur in einem anderen Worktree (RK-012), noch
  nicht in `main`. Sobald sie gemergt ist, die Seite „Aufnahme wählen“ für RK-004 nachtragen (Formate, Bildserien,
  Codecs; Vorlage: `docs/formats.md`, Abschnitt „Was der Klon liest“).

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: **Select**, **MRU**, Angaben zur Datei in der Reiterleiste, Frame-Leiste (`docs/design.md`, Abschnitte Fensteraufbau und Frame-Leiste)
- Soll-Bild: `docs/design/screenshots/01-align-loaded.png`
- Der Zustand vor dem Laden und die Anzeige von Fehlern sind dort noch nicht festgelegt (Abschnitt „Noch nicht festgelegt“)
