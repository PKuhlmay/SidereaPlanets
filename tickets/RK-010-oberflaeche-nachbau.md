# RK-010 Oberfläche nach Vorbild des Originals

- Status: in Arbeit (Oberfläche umgesetzt; offen: Abgleich der Menüs und der Interaktionen im laufenden Fenster, Testlauf, Abbrechen mit echter Berechnung)
- Abhängig von: RK-002, RK-003

## Ziel
Modernes Erscheinungsbild. Das Design ist in `docs/design.md` festgelegt (Tokens, klickbarer Entwurf und Soll-Bilder in `docs/design/`); separate Agenten setzen es im Frontend unter `app/src` um. Das Gerüst dafür (Tauri + Vue) steht, die UI ist nur ein Platzhalter.

Gleicher Aufbau und gleiche Begriffe wie im Original, sodass bisherige the reference program-Nutzer sich sofort zurechtfinden.

## Akzeptanzkriterien
- [x] Design festgelegt: `docs/design.md`, `docs/design/tokens.css`, klickbarer Entwurf und zwölf Soll-Bilder
- [ ] Umsetzung folgt `docs/design.md` (Aufbau, Tokens, Komponenten, Zustände) und ist mit den Soll-Bildern in `docs/design/screenshots/` abgeglichen, dunkel und hell
- [x] Reiter und Schritte in der Reihenfolge des Originals, gleiche Beschriftungen
- [ ] Bildbereich mit Zoom, Pan, Vorher/Nachher
- [x] Reglerwerte und Standardwerte wie in `docs/ui.md`
- [ ] Fortschrittsanzeige und Abbrechen bei langen Berechnungen

## Umsetzung

- App-Shell, Reiter, Aktionszeile, drei linke Panels, Functions-Panel, Bildbereich, Frame-Leiste, Statusleiste, Menüs mit Flyouts,
  schwebende Karten, helles und dunkles Thema (Settings ▸ Theme). Tokens unverändert aus `docs/design/tokens.css`.
- Reine Logik (Sperrlogik, Zahlenfelder, Zoom/Pan-Rechnung, Diagramm, MRU, Standardwerte) liegt in `app/src/logic/` mit vitest-Tests daneben.
- Angebunden ans Backend: Select (`open_recording`), Frame-Leiste (`read_frame`), Cancel und Pause (`cancel_processing`, `pause_processing`).
- Die Schaltflächen, deren Rechenkern fehlt, stehen im richtigen Zustand und melden „not available yet“. Die Anbindung ist eine Stelle:
  `app/src/actions.ts` (Tabelle `handlers`, Zuordnung zu RK-005 bis RK-009 im Dateikopf). Ein Handler startet seine Rechnung mit `runJob`
  (`app/src/jobs.ts`): Fortschritt, Zeiten, Fenstertitel, Cancel und Pause laufen dann von selbst. Ergebnisse gehören in `session` und
  `status` (`app/src/state.ts`): `session.step` schaltet Schritte und Reiter, `session.alignpoints`, `session.frames`, `session.stackImage`,
  `session.processedImage` füllen Bild, Karten und Marken. Die Werte der Bedienelemente stehen in `settings`.
- In-App-Doku: neue Seiten zu Bildbereich, den drei Reitern, Menüs und Fortschritt; „The window at a glance“ aktualisiert.
- `docs/design.md` ergänzt: Theme-Eintrag und Flyouts in Settings, leerer Zustand, Fehlermeldungen, Dev-Vorschau.

## Geprüft und nicht geprüft

- Geprüft: `pnpm build` (vue-tsc und vite) und `cargo check -p planets-app` laufen durch. Der Abgleich mit den Soll-Bildern 04, 06, 08
  (dunkel) sowie 01, 03, 05, 11 und 12 (hell, gegen das dunkle Soll-Bild des Zustands bzw. gegen 11/12) erfolgte per Bildschirmfoto der Dev-Vorschau in einem Browser ohne Fenster (Edge headless gegen `pnpm exec vite`,
  Zustände über die URL-Parameter der Vorschau): Aufbau, Maße, Zustände und Farben stimmen mit den Soll-Bildern überein.
- Nicht geprüft: Lauf im Tauri-Fenster (Select, Dateidialog, Frames laden); Interaktionen (Mausrad-Zoom, Ziehen, Menüs öffnen, Flyouts);
  Abgleich mit den Soll-Bildern 02, 07, 09, 10, 13, 14. Die vitest-Tests sind geschrieben, aber noch nicht gelaufen und damit auch noch nicht
  „rot zuerst“ geprüft (Philipps Wunsch: Testläufe am Ende).
- Offen bis RK-005: Abbrechen und Fortschritt mit einer echten Berechnung. Der Mechanismus (`runJob`, Cancel, Pause) steht, Align ist aber nicht angebunden.
- Offen bis RK-007: Vorher/Nachher zeigt erst einen Unterschied, wenn es ein bearbeitetes Bild gibt (`session.processedImage`).
- Bewusst ausgelassen: Speicherangabe in der Reiterleiste (braucht einen Backend-Befehl).
