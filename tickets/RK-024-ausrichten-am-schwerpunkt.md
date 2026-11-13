# RK-024 Align by center of gravity (Windows zieht nach)

- Status: offen
- Abhängig von: RK-016, MAC-021
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die gesperrten Optionen **Align by center of gravity** und **Brightness threshold** im Schritt Align werden frei, wie in der Mac-App.

## Umfang

Werte, Tooltips, Projektdatei und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „Align by center of gravity (Mac:
MAC-021)“, Aufbau in `docs/design.md`, „3 Align“. Kern und Session liegen auf `main`; die Tauri-Befehle reichen das Feld schon durch.

## Akzeptanzkriterien

- [ ] Häkchen (Standard aus) und Brightness threshold (0–255, Standard 20, eingerückt, nur mit Häkchen aktiv) in `AlignStep.vue`;
      Estimate rotation bleibt gesperrt
- [ ] Align und Stack schicken `centreOfGravity` (Zahl oder `null`) mit (`alignArgs`, `app/src/logic/stack.ts`) (Test)
- [ ] Projektdatei: `align.centreOfGravity` und `align.brightnessThreshold` schreiben und lesen, fehlende Werte bleiben (Test)
- [ ] Windows-Hinweise in `align-your-frames.md`, `steps-alignpoints-align-limit.md` und `glossary.md` gestrichen
- [ ] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [ ] Probe von Hand als `## RK-024 Align by center of gravity (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen
