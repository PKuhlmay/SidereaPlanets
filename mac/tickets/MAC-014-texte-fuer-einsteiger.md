# MAC-014 Texte für Einsteiger

- Status: offen
- Abhängig von: MAC-013
- Anhang: `MAC-014-anhang-text-check.md` (Text-Check vom 2026-11-12, 165 Befunde: 24 hoch, 64 mittel, 77 niedrig, je mit Datei:Zeile,
  Begründung, Vorschlag, Art und Schwere)

## Ziel

Die Oberfläche soll für Einsteiger in die Planetenfotografie verständlich sein (Philipp, 2026-11-12). Entschieden:

- **Klar benennen, the reference program im Tooltip:** Unklare Begriffe bekommen einen verständlichen Namen; der Tooltip endet mit
  „the reference program: ‹Originalname›“, damit Anleitungen im Netz auffindbar bleiben. Etablierte Begriffe (Alignpoints, Wavelets, Stack, Do All,
  Limit) bleiben und bekommen erklärende Tooltips.
- **Ohne Wirkung = gesperrt:** Schalter, Felder und Knöpfe, die nichts tun, werden gedimmt mit Tooltip „Not available yet“, wie die
  übrigen noch nicht gebauten Optionen.
- **US-Englisch** überall (Color, Center …); die britische Schreibweise von the reference program steht im Tooltip, wo sie ein Begriff des
  Originals ist.
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
- [ ] Tests grün (Mac, `pnpm test` für die Doku-Seiten), Texte in Tests angepasst
