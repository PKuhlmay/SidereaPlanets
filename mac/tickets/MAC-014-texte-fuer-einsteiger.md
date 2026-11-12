# MAC-014 Texte für Einsteiger

- Status: offen
- Abhängig von: MAC-013
- Anhang: `MAC-014-anhang-text-check.md` (Text-Check vom 2026-11-12, 165 Befunde: 24 hoch, 64 mittel, 77 niedrig, je mit Datei:Zeile,
  Begründung, Vorschlag, Art und Schwere)

## Ziel

Die Oberfläche soll für Einsteiger in die Planetenfotografie verständlich sein (Philipp, 2026-11-12). Entschieden:

- **Klar benennen, erklären im Tooltip:** Unklare Begriffe bekommen einen verständlichen Namen und einen erklärenden Tooltip.
  Etablierte Fachbegriffe der Planetenfotografie (Alignpoints, Wavelets, Stack, Do All, Limit) bleiben und bekommen ebenfalls Tooltips.
- **Kein Hinweis auf the reference program** (Philipp, 2026-11-12): Siderea Planets ist ein eigenes Programm. In keinem sichtbaren Text beider Apps
  (Oberfläche, Tooltips, Dateidialoge, Hilfeseiten, Statusmeldungen, Dateiköpfe) steht „the reference program“, „the reference program“ oder „das Original“.
  Vergleiche mit dem Original gehören nur in die interne Doku (`docs/`). Das Format `.rwv` heißt in Dialogen „Wavelet scheme“.
- **Ohne Wirkung = gesperrt:** Schalter, Felder und Knöpfe, die nichts tun, werden gedimmt mit Tooltip „Not available yet“, wie die
  übrigen noch nicht gebauten Optionen.
- **US-Englisch** überall (Color, Center …).
- **Umfang:** alle hohen und mittleren Befunde, dazu die niedrigen, die Tippfehler, Groß- und Kleinschreibung oder „…“ betreffen;
  neue Hilfeseite „Glossary“.

## Akzeptanzkriterien

- [ ] Alle hohen und mittleren Befunde aus dem Anhang umgesetzt oder im Anhang mit Begründung als bewusst verworfen markiert
- [ ] Niedrige Befunde zu Tippfehlern, Schreibweise und „…“ umgesetzt
- [ ] Bedienelemente ohne Wirkung gesperrt mit „Not available yet“
- [ ] Sachfehler behoben: Align-Ergebnis zeigt nicht mehr immer „best quality 100 %“
- [ ] Einheitliche Schreibweise (HIG: Title Case für Knöpfe und Menüs, Sentence case für Felder und Tooltips, „…“ bei Dialogen),
      US-Englisch, Einheiten an den Feldern
- [ ] Anwenderseiten `app/src/docs/pages/` nutzen die neuen Namen, neue Seite „Glossary“ (neuer Name, the reference program-Name, Erklärung)
- [ ] `docs/design.md` und `docs/ui.md` (Spalte für den Namen in der App) nachgezogen
- [ ] Ein Sammelpunkt in `docs/parity/WINDOWS_PARITY.md` mit der vollständigen Zuordnung alt → neu, den Tooltips und den
      gesperrten Elementen, damit Windows nachziehen kann
- [ ] Keine Erwähnung von the reference program oder „original“ in sichtbaren Texten beider Apps und in `app/src/docs/pages/`; Suche im Ticket
      dokumentiert; Regel in `CLAUDE.md` (Abschnitt „Name“) festgehalten
- [ ] Tests grün (Mac, `pnpm test` für die Doku-Seiten), Texte in Tests angepasst
