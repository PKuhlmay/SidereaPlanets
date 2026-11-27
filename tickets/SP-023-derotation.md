# SP-023 De-rotate images (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-016, MAC-020
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Das gesperrte Häkchen **De-rotate images** im Schritt Stack wird frei: Windows dreht die Frames von Jupiter, Saturn und Mars beim
Stacken gegen die Eigendrehung des Planeten zurück, wie die Mac-App.

## Umfang

Felder, Overlay, Projektdatei und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „De-rotate images (Mac: MAC-020)“, Aufbau
in `docs/design.md`, „Derotation (MAC-020)“.

Der Kern liegt auf `main` (`StackSettings.derotation`, `Session::derotation_info`, Zeitstempel in `RecordingInfo` und `planets_io::ser`).
Windows baut die Gruppe in `StackStep.vue`, schickt `derotation` mit und braucht einen Tauri-Befehl für `derotation_info` (Span,
Drehung, gemessene Scheibe für das Overlay).

## Akzeptanzkriterien

- [x] Häkchen frei; eingeschaltet Planet (Vorschlag aus dem Dateinamen), Start time (nur ohne Zeit je Frame, Ortszeit plus UTC, Hinweis
      bei Dateidatum), Camera angle, Mirrored image, Zeile „Span … · rotation …°“ mit „Too short to matter“ unter 1°
- [x] Stack schickt `derotation` mit den Werten aus dem Parity-Punkt (Test)
- [x] Overlay im Schritt Stack: Scheibe als Ellipse, bei Saturn die Ringkante gestrichelt, Strich mit **N**, auf dem Bezugsframe;
      verzögert neu berechnet nach Änderung der Felder und der Frame-Auswahl
- [x] Projektdatei-Abschnitt `derotation` schreiben und lesen, fehlende Schlüssel bleiben (Test)
- [x] `stack-pages.test.ts` zählt sechs gesperrte Optionen; Zeile „Works in the Mac version“ in `step-stack.md` und der Test
      `describes De-rotate images as working in the Mac version` angepasst
- [x] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## SP-023 De-rotate images (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen

## Umsetzung

- **Tauri:** Befehl `derotation_info` in `app/src-tauri/src/stack_commands.rs` (in `lib.rs` registriert), dünne Hülle über
  `Session::derotation_info`; `DerotationView` und `DiscView` bringen Antwort und Scheibe in camelCase ans Fenster (`ring` und `north` als
  Paare). `open_recording` reichte `hasTimestamps`, `startTime` und `startTimeFromFileDate` schon durch, nur `RecordingInfo` in `state.ts`
  kannte sie nicht. Der Kern blieb unverändert.
- **Logik** (`app/src/logic/derotation.ts`): was Stack und der Befehl bekommen, Planet aus dem Dateinamen (wie `Planet::from_name`),
  Spanne, Zeile mit Hinweis, UTC-Text, Feld für Datum und Uhrzeit (Ortszeit, minutengenau), Overlay-Geometrie und der Abschnitt
  `derotation`. `settings.stack` hat neu `planet`, `cameraAngle`, `mirrored`, `startTime` (Sekunden seit 1970 UTC); `buildStackRequest`
  schickt `derotation` nur, wenn das Häkchen an ist.
- **Ablauf** (`app/src/derotation.ts`, gestartet in `main.ts`): fragt den Kern 120 ms nach jeder Änderung der Felder, des Schritts, des
  Bezugsframes oder der Frame-Auswahl; ältere Antworten fallen weg; mit Scheibe zeigt der Schritt Stack das Bezugsframe. Beim Öffnen
  setzt `startSession` die Startzeit der Datei (oder keine) und den Planeten aus dem Dateinamen; das Häkchen bleibt, wie es ist.
- **Oberfläche:** Gruppe in `StackStep.vue` (18 px eingerückt, Texte und Tooltips wie am Mac; der Tooltip von Start time sagt „this
  computer“ statt „this Mac“), Overlay in `ImageOverlays.vue` (Ellipse `--accent` 1,5 px, Ring gestrichelt 1 px mit 70 %, Strich und N in
  Bildpixeln).
- **Projektdatei:** Abschnitt `derotation` mit den Schlüsseln der Mac-App; der Abschnitt `stack` trägt die Werte der Derotation nicht mehr
  (ein altes `stack.derotate` wird beim Lesen übergangen).
- **Doku:** `step-stack.md` (Zeile „Works.“, „time of your computer“, Satz zur Windows-Version gestrichen), `docs/design.md`,
  `docs/stacking.md`.
- **Rot zuerst:** `logic/derotation.test.ts` (16 Tests, zwei Stubs: erst neutrale Rückgaben, dann gegenteilige, alle 16 an Assertions rot),
  `derotation.test.ts` (8 Tests, Stub ohne Watcher: 7 rot; „asks nothing while the option is off“ rot mit einem Stub, der das Häkchen
  übergeht), `stack.test.ts` und `projectfile.test.ts` (3 neue Tests rot vor der Umsetzung), `stack-pages.test.ts` (3 geänderte bzw. neue
  Tests rot gegen das alte `StackStep.vue` und die alte Seite), Rust `the_derotation_info_reaches_the_window_in_its_shape` rot mit einer
  Umwandlung, die Nullen lieferte. Im echten Fenster nicht von Hand probiert, dafür die Handprobe.
