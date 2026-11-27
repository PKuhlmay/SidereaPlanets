# MAC-014 Texte für Einsteiger

- Status: erledigt
- Abhängig von: MAC-013
- Anhang: `MAC-014-anhang-text-check.md` (Text-Check vom 2026-11-12, 165 Befunde: 24 hoch, 64 mittel, 77 niedrig, je mit Datei:Zeile,
  Begründung, Vorschlag, Art und Schwere)

## Ziel

Die Oberfläche soll für Einsteiger in die Planetenfotografie verständlich sein (Philipp, 2026-11-12). Entschieden:

- **Klar benennen, erklären im Tooltip:** Unklare Begriffe bekommen einen verständlichen Namen und einen erklärenden Tooltip.
  Etablierte Fachbegriffe der Planetenfotografie (Alignpoints, Wavelets, Stack, Do All) bleiben und bekommen ebenfalls Tooltips.
- **Kein Hinweis auf das Referenzprogramm** (Philipp, 2026-11-12): Siderea Planets ist ein eigenes Programm. In keinem sichtbaren Text beider Apps
  (Oberfläche, Tooltips, Dateidialoge, Hilfeseiten, Statusmeldungen, Dateiköpfe) steht der Name des Referenzprogramms (mit oder ohne
  Versionsnummer) oder „das Original“.
  Vergleiche mit dem Original gehören nur in die interne Doku (`docs/`). Das Format `.rwv` heißt in Dialogen „Wavelet scheme“.
- **Ohne Wirkung = gesperrt:** Schalter, Felder und Knöpfe, die nichts tun, werden gedimmt mit Tooltip „Not available yet“, wie die
  übrigen noch nicht gebauten Optionen.
- **US-Englisch** überall (Color, Center …).
- **Schritt 4 heißt „Best Frames“** (Philipp, 2026-11-12, nach dem Text-Check): Pille, Panelkopf und Menü Go to Step „Best Frames“, Aktionsknopf
  „Keep Best Frames“ (fertig: „Keep Best Frames Again“; Sperr-Tooltip „Run Keep Best Frames first“), Modi „At least % of the best
  quality“, „Best % of frames“, „Best frames per point“, „Best number of frames“, Status „Keeping N of M frames“. „Limit“ bleibt als
  interner Name (`ViewStep.limit`, Ticket MAC-006, Datei `frame-quality-and-limit.md`) und im Glossar nur als Verweis.
- **Toolbar** (Befund 0.1): „Open“ und „Save“ der Projektdatei heißen „Open Project…“ und „Save Project…“, mit Tooltips.
- **Umfang:** alle hohen und mittleren Befunde, dazu die niedrigen, die Tippfehler, Groß- und Kleinschreibung oder „…“ betreffen;
  neue Hilfeseite „Glossary“.

## Akzeptanzkriterien

- [x] Alle hohen und mittleren Befunde aus dem Anhang umgesetzt oder im Anhang mit Begründung als bewusst verworfen markiert
- [x] Niedrige Befunde zu Tippfehlern, Schreibweise und „…“ umgesetzt
- [x] Bedienelemente ohne Wirkung gesperrt mit „Not available yet“
- [x] Sachfehler behoben: Align-Ergebnis zeigt nicht mehr immer „best quality 100 %“
- [x] Einheitliche Schreibweise (HIG: Title Case für Knöpfe und Menüs, Sentence case für Felder und Tooltips, „…“ bei Dialogen),
      US-Englisch, Einheiten an den Feldern
- [x] Anwenderseiten `app/src/docs/pages/` nutzen die neuen Namen, neue Seite „Glossary“ (Begriff, Name in der App, Erklärung; ohne den Namen des Referenzprogramms, siehe Regel oben)
- [x] `docs/design.md` und `docs/ui.md` (Spalte für den Namen in der App) nachgezogen
- [x] Ein Sammelpunkt in `docs/parity/WINDOWS_PARITY.md` mit der vollständigen Zuordnung alt → neu, den Tooltips und den
      gesperrten Elementen, damit Windows nachziehen kann
- [x] Keine Erwähnung des Referenzprogramms oder von „original“ in sichtbaren Texten beider Apps und in `app/src/docs/pages/`; Suche im Ticket
      dokumentiert; Regel in `CLAUDE.md` (Abschnitt „Name“) festgehalten
- [x] Tests grün (Mac, `pnpm test` für die Doku-Seiten), Texte in Tests angepasst

## Umsetzung (2026-11-12)

- **Alle 24 hohen und 64 mittleren Befunde** sind umgesetzt, die niedrigen zu Tippfehlern, Schreibweise und „…“ ebenso; abweichende und
  bewusst nicht umgesetzte stehen mit Grund am Ende des Anhangs (`MAC-014-anhang-text-check.md`, „Umsetzung und Abweichungen“).
- **Gesperrt (gedimmt, „Not available yet“)**: Show Full Image (3×), Show Align Data, Show Prefilter, Show Stack Graph, Show Stack List
  (im Show-Menü unter dem Abschnitt „Not available yet“), Show alignment, Codec, Compression quality, die drei Frame-size-Radios außer
  Original size, Use prefilters, Clear List, Keep (Contrast/Brightness, früher Hold: ohne Wirkung), die elf Adjustments-Knöpfe ohne Karte,
  Tooltip an Stack Again. Rot-zuerst-Tests: siehe unten.
- **Sachfehler**: Align zeigt „‹n› frames rated“ statt „best quality 100 %“ (die Qualität ist auf den besten Frame normiert).
- **Rot zuerst**: `AlignModelTests.alignRatesTheFramesAndOpensLimit` (Erwartung „10 frames rated“ gegen den alten Text „best quality
  100 %“), `StepLogicTests.saveButtonIsTitleCase` (Label „Save Image“ gegen „Save image“) und `ShowMenuLogicTests` (neue Logik
  `ShowItem.isAvailable` zuerst als Attrappe, die alles freigibt: die Assertionen, dass Show Full Image & Co. gesperrt sind, schlugen
  fehl). Danach implementiert, 240 Tests grün.
- **Windows**: nur gemeinsame Dateien geändert (Anwenderseiten samt Glossary, Rechenkern-Fehlertexte „color“, die Windows-Doku-Tests);
  die Windows-Oberfläche zieht über den Sammelpunkt in `docs/parity/WINDOWS_PARITY.md` nach („Verständliche Texte für Einsteiger“).
- **Suche** (ohne Beachtung der Groß- und Kleinschreibung) nach dem Namen des Referenzprogramms und nach „the original“ in `mac/App`,
  `app/src/docs/pages` und `crates/*/src` am Ende: keine Treffer in sichtbaren
  Texten, in den Anwenderseiten und in `mac/App` nur Code-Kommentare und Bezeichner (`SidereaPlanetsKit`, `PlanetsSession`, …), in `crates/`
  Doc-Kommentare, Bezeichner (`planets_*`) und ein Test. Sichtbar war der Name in der Mac-App im Über-Text, im Dateidialog (Dateityp der
  `.rwv`-Schemata), in den Einstellungen (Schemes) und in `NSHumanReadableCopyright` in `mac/project.yml`; alle ersetzt. Windows:
  `app/src/wavelet.ts:142` (Dialogfilter) bleibt bis zum Nachzug, steht im Sammelpunkt. Regel in `CLAUDE.md`, Abschnitt „Name“.
