# SP-010 Oberfläche nach Vorbild des Originals

- Status: in Arbeit (Oberfläche umgesetzt; offen: Abgleich der Menüs und der Interaktionen im laufenden Fenster, Testlauf, Abbrechen mit echter Berechnung)
- Abhängig von: SP-002, SP-003

## Ziel
Modernes Erscheinungsbild. Das Design ist in `docs/design.md` festgelegt (Tokens, klickbarer Entwurf und Soll-Bilder in `docs/design/`); separate Agenten setzen es im Frontend unter `app/src` um. Das Gerüst dafür (Tauri + Vue) steht, die UI ist nur ein Platzhalter.

Gleicher Aufbau und gleiche Begriffe wie im Original, sodass sich bisherige Nutzer des Referenzprogramms sofort zurechtfinden.

## Akzeptanzkriterien
- [x] Design festgelegt: `docs/design.md`, `docs/design/tokens.css`, klickbarer Entwurf und zwölf Soll-Bilder
- [ ] Umsetzung folgt `docs/design.md` (Aufbau, Tokens, Komponenten, Zustände) und ist mit den Soll-Bildern in `docs/design/screenshots/` abgeglichen, dunkel und hell
- [x] Reiter und Schritte in der Reihenfolge des Originals, gleiche Beschriftungen
- [x] Bildbereich mit Zoom, Pan, Vorher/Nachher (Release-Fenster, 2026-11-12: Mausrad-Zoom um den Zeiger, Ziehen, Klick auf den Maßstab = 100 %, Einpassen bei Schrittwechsel, Before-Umschalter)
- [x] Reglerwerte und Standardwerte wie in `docs/ui.md`
- [x] Fortschrittsanzeige und Abbrechen bei langen Berechnungen (Release-App, 2026-11-12: Align mit Box 100 und Max movement 40 zeigt Balken, Frame-Zähler und Restzeit, Pause hält an, Cancel stoppt nach 0,2 s und leert den Balken)

## Umsetzung

- App-Shell, Reiter, Aktionszeile, drei linke Panels, Functions-Panel, Bildbereich, Frame-Leiste, Statusleiste, Menüs mit Flyouts,
  schwebende Karten, helles und dunkles Thema (Settings ▸ Theme). Tokens unverändert aus `docs/design/tokens.css`.
- Reine Logik (Sperrlogik, Zahlenfelder, Zoom/Pan-Rechnung, Diagramm, MRU, Standardwerte) liegt in `app/src/logic/` mit vitest-Tests daneben.
- Angebunden ans Backend: Select (`open_recording`), Frame-Leiste (`read_frame`), Cancel und Pause (`cancel_processing`, `pause_processing`).
- Die Schaltflächen, deren Rechenkern fehlt, stehen im richtigen Zustand und melden „not available yet“. Die Anbindung ist eine Stelle:
  `app/src/actions.ts` (Tabelle `handlers`, Zuordnung zu SP-005 bis SP-009 im Dateikopf). Ein Handler startet seine Rechnung mit `runJob`
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
  Abgleich mit den Soll-Bildern 09, 10, 13, 14 (Menüs und Doku brauchen Klicks im Fenster). Am 2026-11-11 lief die Debug-App nicht mehr, ihr
  Protokoll `tauri-dev.log` zeigt nur Baufehler aus der Zeit der parallelen Arbeit (fehlendes Modul, `marked` nicht installiert), keinen
  Laufzeitfehler; das Fenster selbst wurde nicht bedient.
- Abbrechen und Fortschritt: Align und Stack sind angebunden (Fake-Backend-Tests in `app/src/flow.test.ts`); mit der echten Berechnung im Fenster noch nicht gesehen.
- Vorher/Nachher unterscheidet sich, sobald Process oder Do All gelaufen ist (`session.processedImage`); im Fenster noch nicht gesehen.
- Bewusst ausgelassen: Speicherangabe in der Reiterleiste (braucht einen Backend-Befehl).

## Betriebsbereit-Prüfung (2026-11-11)

- Backend-Kette: `crates/io/tests/pipeline.rs` läuft JUPITER.AVI durch Alignpoints, Align, Limit, Stack, Wavelet (Standard, Ebene 1 = 10,0, Vorschau
  gegen Do All) und Save image in allen sieben Formaten mit Wiedereinlesen; dazu Cancel und leere Auswahl. Grün; rot zuerst an der Schärfe-Assertion gesehen.
- Zusammenspiel Fenster und Befehle: `app/src/flow.test.ts` fährt die Handler gegen ein Fake-Backend mit den Antwortformen der Rust-Befehle (Schritte, Cancel
  von Align und Stack über das jeweils richtige Flag, Fehler, Fortschrittsereignisse, neuer Stack macht das geschärfte Bild ungültig).
- Gefunden und behoben: (1) `session.referenceFrame` wurde nie gesetzt, Stack registrierte immer gegen Frame 1, auch wenn die Alignpoints auf einem anderen
  Frame lagen; (2) ein neuer Stack ließ das alte geschärfte Bild für Save image im Wavelet-Reiter stehen (`ExportState::set_stack`); (3) getippte
  Kommazahlen oder negative Werte in ganzzahligen Feldern (Abstände, Boxgröße, Anzahl, Initial Layer, Step Increment, Max shift) ließen die Befehle mit einem
  Deserialisierungsfehler scheitern; (4) die Referenztests fanden `re-local/` in einem Worktree nicht und übersprangen sich (`tests/common/mod.rs`).
- Abgleich mit Soll-Bildern 02 und 07 (Edge ohne Fenster, Dev-Vorschau): Aufbau und Maße stimmen; Unterschiede sind die Help-Schaltfläche (SP-012),
  die fehlende Speicherangabe (bewusst ausgelassen) und eine Pixelzeile Höhenversatz der Werkzeugleiste.
- Offen: Do All und Process lassen sich nicht abbrechen (kein Abbruch-Flag im Rust-Befehl, Cancel verwirft nur das Ergebnis); die Farb- und Tonkarten
  fließen nicht in Do All und Save image ein (SP-008/SP-009); ein zweites Align ist nach dem Design nicht vorgesehen (Schaltflächen gesperrt, Zustandstabelle).
- UI-Prüflauf der Release-App (2026-11-12), behoben: Bild wird bei jedem Schrittwechsel neu eingepasst und ein Klick ohne Ziehen hebt das Mitfolgen der Fläche nicht mehr auf (Zoom blieb nach Wavelets bei 73 %); Fortschrittsbalken steht außerhalb einer Berechnung auf 0 % (auch nach Abbruch und Fehler); Limit meldet „Stacksize: n  Low quality: x.xx%“ (Bedeutung von Low quality ist eine Vermutung, `docs/workflow.md`); die Vorschau des Processing Area läuft nur noch im Schritt Wavelets und nicht, solange ein Do All für die Werte vorliegt (der Wechsel nach Save ersetzte das Do-All-Bild durch Summenbild plus Ausschnitt und überschrieb „Do All finished“); Save image trägt in Stack und Save den Pfeil nach unten. „Advanced 8 more settings“ im Schritt Wavelets bleibt: es zählen die Einstellungen wie in den anderen Schritten, das Soll-Bild zählt mit Load Scheme, Save Scheme, Realign und Stack Again 12; alle Felder aus `docs/ui.md` sind vorhanden.
