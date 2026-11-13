# RK-023 De-rotate images (Windows zieht nach)

- Status: offen
- Abhängig von: RK-016, MAC-020
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

- [ ] Häkchen frei; eingeschaltet Planet (Vorschlag aus dem Dateinamen), Start time (nur ohne Zeit je Frame, Ortszeit plus UTC, Hinweis
      bei Dateidatum), Camera angle, Mirrored image, Zeile „Span … · rotation …°“ mit „Too short to matter“ unter 1°
- [ ] Stack schickt `derotation` mit den Werten aus dem Parity-Punkt (Test)
- [ ] Overlay im Schritt Stack: Scheibe als Ellipse, bei Saturn die Ringkante gestrichelt, Strich mit **N**, auf dem Bezugsframe;
      verzögert neu berechnet nach Änderung der Felder und der Frame-Auswahl
- [ ] Projektdatei-Abschnitt `derotation` schreiben und lesen, fehlende Schlüssel bleiben (Test)
- [ ] `stack-pages.test.ts` zählt sechs gesperrte Optionen; Zeile „Works in the Mac version“ in `step-stack.md` und der Test
      `describes De-rotate images as working in the Mac version` angepasst
- [ ] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-023 De-rotate images (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen
