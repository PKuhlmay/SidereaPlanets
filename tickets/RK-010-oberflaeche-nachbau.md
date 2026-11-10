# RK-010 Oberfläche nach Vorbild des Originals

- Status: in Arbeit (Design festgelegt, Umsetzung in `app/src` offen)
- Abhängig von: RK-002, RK-003

## Ziel
Modernes Erscheinungsbild. Das Design ist in `docs/design.md` festgelegt (Tokens, klickbarer Entwurf und Soll-Bilder in `docs/design/`); separate Agenten setzen es im Frontend unter `app/src` um. Das Gerüst dafür (Tauri + Vue) steht, die UI ist nur ein Platzhalter.

Gleicher Aufbau und gleiche Begriffe wie im Original, sodass bisherige the reference program-Nutzer sich sofort zurechtfinden.

## Akzeptanzkriterien
- [x] Design festgelegt: `docs/design.md`, `docs/design/tokens.css`, klickbarer Entwurf und zwölf Soll-Bilder
- [ ] Umsetzung folgt `docs/design.md` (Aufbau, Tokens, Komponenten, Zustände) und ist mit den Soll-Bildern in `docs/design/screenshots/` abgeglichen, dunkel und hell
- [ ] Reiter und Schritte in der Reihenfolge des Originals, gleiche Beschriftungen
- [ ] Bildbereich mit Zoom, Pan, Vorher/Nachher
- [ ] Reglerwerte und Standardwerte wie in `docs/ui.md`
- [ ] Fortschrittsanzeige und Abbrechen bei langen Berechnungen
