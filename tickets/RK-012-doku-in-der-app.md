# RK-012 Anwenderdoku in der App

- Status: offen
- Abhängig von: RK-010

## Ziel
Die Anwenderdoku ist in der App über die UI erreichbar, im gleichen Design wie die übrige Oberfläche. Die Regel „Doku in jedem Commit mitpflegen“ steht in der `CLAUDE.md` („Doku in der App“); dieses Ticket schafft die Grundlage dafür.

Im Design (`docs/design.md`) fehlt dafür bisher ein Einstieg: Die Werkzeugleiste hat rechts nur About und CPUs, und es gibt kein Icon für Hilfe/Doku. Zuerst wird das Design ergänzt, danach umgesetzt.

## Akzeptanzkriterien
- [ ] Design ergänzt: Einstieg in der Werkzeugleiste (Vorschlag: „Help“ ohne Rahmen links von About, mit Icon), Doku-Ansicht als eigener Bildschirm mit Seitenliste links und Text rechts, jeweils dunkel und hell
- [ ] Icon im Stil der übrigen Oberfläche entworfen (Größe, Strichstärke, Farbe aus den Tokens) und in `docs/design/siderea-planets.figcraft` angelegt; `docs/design/figcraft-build.mjs` erzeugt es mit
- [ ] `docs/design.md`, `docs/design/tokens.css`, klickbarer Entwurf und Soll-Bilder nachgezogen
- [ ] Doku-Ansicht in `app/src` umgesetzt, aus der Werkzeugleiste erreichbar, Rückweg zur Arbeitsansicht ohne Verlust des Zustands (geladenes Video, Reglerwerte)
- [ ] Doku-Inhalte als Markdown im Repo (Pfad im Ticket festhalten), in der App gerendert, Suche oder Seitenliste vorhanden
- [ ] Erste Seiten geschrieben für das, was schon läuft: Aufnahme wählen (SER, AVI), Oberfläche im Überblick
- [ ] Doku bleibt auf Anwenderebene (Bedienung und Ergebnis); Reverse-Engineering-Notizen bleiben in `docs/`
