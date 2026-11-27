# SP-024 Align by center of gravity (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-016, MAC-021
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Die gesperrten Optionen **Align by center of gravity** und **Brightness threshold** im Schritt Align werden frei, wie in der Mac-App.

## Umfang

Werte, Tooltips, Projektdatei und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`, Abschnitt „Align by center of gravity (Mac:
MAC-021)“, Aufbau in `docs/design.md`, „3 Align“. Kern und Session liegen auf `main`; die Tauri-Befehle reichen das Feld schon durch.

## Akzeptanzkriterien

- [x] Häkchen (Standard aus) und Brightness threshold (0–255, Standard 20, eingerückt, nur mit Häkchen aktiv) in `AlignStep.vue`;
      Estimate rotation bleibt gesperrt
- [x] Align und Stack schicken `centreOfGravity` (Zahl oder `null`) mit (`alignArgs`, `app/src/logic/stack.ts`) (Test)
- [x] Projektdatei: `align.centreOfGravity` und `align.brightnessThreshold` schreiben und lesen, fehlende Werte bleiben (Test)
- [x] Windows-Hinweise in `align-your-frames.md`, `steps-alignpoints-align-limit.md` und `glossary.md` gestrichen
- [x] Abschnitt aus `docs/parity/WINDOWS_PARITY.md` gelöscht
- [x] Probe von Hand als `## SP-024 Align by center of gravity (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cd app && pnpm test` und `pnpm build` grün, neue oder geänderte Tests einmal rot gesehen

## Umsetzung

- `app/src/logic/settings.ts`: neue Werte `settings.align.centreOfGravity` (aus) und `settings.align.brightnessThreshold` (20).
- `app/src/logic/align.ts`: `centreOfGravityThreshold` (aus = `null`, an = Schwelle gerundet und auf 0–255 begrenzt, wie am Mac);
  `alignArgs` und `buildStackRequest` (`app/src/logic/stack.ts`) schicken `centreOfGravity` mit, damit auch Save Aligned und Save Integrated.
- `app/src/components/AlignStep.vue`: Häkchen frei, Brightness threshold eingerückt (20 px), Zahlenfeld 0–255, ohne Häkchen gesperrt und
  auf 45 % gedimmt; Tooltips wie am Mac. Show alignment und Estimate rotation bleiben gesperrt.
- Projektdatei (`app/src/logic/projectfile.ts`): die Schlüssel kommen mit dem Abschnitt `align` (gleiche Namen wie am Mac), beim Lesen wird
  die Schwelle auf 0–255 begrenzt; fehlende Schlüssel lassen die Werte stehen.
- Hilfe: Windows-Hinweise in `align-your-frames.md` und `steps-alignpoints-align-limit.md` gestrichen; `glossary.md` hatte keinen.
- Tests: `align.test.ts`, `stack.test.ts`, `projectfile.test.ts`, `app/src/docs/locked-buttons.test.ts` (Häkchen frei, Show alignment
  und Estimate rotation gesperrt), alle einmal rot gesehen.
