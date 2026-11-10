# RK-012 Anwenderdoku in der App

- Status: erledigt
- Abhängig von: RK-010

## Ziel
Die Anwenderdoku ist in der App über die UI erreichbar, im gleichen Design wie die übrige Oberfläche. Die Regel „Doku in jedem Commit mitpflegen“ steht in der `CLAUDE.md` („Doku in der App“); dieses Ticket schafft die Grundlage dafür.

Im Design (`docs/design.md`) fehlt dafür bisher ein Einstieg: Die Werkzeugleiste hat rechts nur About und CPUs, und es gibt kein Icon für Hilfe/Doku. Zuerst wird das Design ergänzt, danach umgesetzt.

## Akzeptanzkriterien
- [x] Design ergänzt: Einstieg in der Werkzeugleiste („Help“ ohne Rahmen links von About, mit Icon), Doku-Ansicht als eigener Bildschirm mit Seitenliste links und Text rechts, jeweils dunkel und hell
- [x] Icon im Stil der übrigen Oberfläche entworfen (Größe, Strichstärke, Farbe aus den Tokens) und in `docs/design/siderea-planets.figcraft` angelegt; `docs/design/figcraft-build.mjs` erzeugt es mit (Help-Button in der Werkzeugleiste, im Entwurf und in `docs/design.md` beschrieben)
- [x] `docs/design.md` und Soll-Bilder nachgezogen (13 und 14 aus der FigCraft-Datei). `tokens.css` brauchte nichts Neues. Der klickbare Entwurf zeigt die Doku bewusst nicht: die Umsetzung in der App ist die Vorlage, ein zweiter Stand in HTML wäre doppelte Pflege
- [x] Doku-Ansicht in `app/src` umgesetzt, aus der Werkzeugleiste erreichbar, Rückweg zur Arbeitsansicht ohne Verlust des Zustands (geladenes Video, Reglerwerte)
- [x] Doku-Inhalte als Markdown im Repo (`app/src/docs/pages/*.md`, je Seite ein Kopf mit title, group, order), in der App gerendert, Seitenliste und Suche vorhanden
- [x] Erste Seiten geschrieben für das, was schon läuft: Aufnahme wählen (SER, AVI), Oberfläche im Überblick
- [x] Doku bleibt auf Anwenderebene (Bedienung und Ergebnis); Reverse-Engineering-Notizen bleiben in `docs/`

## Hinweis für RK-010
Die Doku-Ansicht ist in den Platzhalter `App.vue` eingehängt (Help-Schalter, `v-show` für die Arbeitsansicht). Baut RK-010 die echte Werkzeugleiste, wandert der Help-Schalter dorthin; `app/src/docs/DocsView.vue` bleibt unverändert und meldet das Schließen per `close`.
