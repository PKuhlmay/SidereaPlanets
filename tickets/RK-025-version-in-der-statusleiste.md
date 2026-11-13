# RK-025 Version in der Statusleiste

- Status: erledigt
- Abhängig von: RK-019
- Stufe: `windows-v0.1.0`

## Ziel

Die Versionsnummer steht in der Oberfläche, damit man auf einen Blick sieht, welcher Stand läuft (Philipp, 2026-11-13: „wie bei
Siderea“). Siderea zeigt rechts in der Statusleiste `0.1.0 · 3801a70 · 2026-11-02 14:32` (Version, Commit, Bauzeit) mit dem Tooltip
„Version of Siderea and the time this build was made.“

## Akzeptanzkriterien

- [x] Rechts in der Statusleiste steht „Version · Commit · Bauzeit“ (`0.1.0 · a4e826b · 2026-11-13 13:38`), Unbekanntes fällt weg (Test)
- [x] Tooltip „Version of Siderea Planets and the time this build was made.“
- [x] Die Version kommt aus `app/package.json` (der Release setzt sie zusammen mit `tauri.conf.json`), Commit und Zeit beim Bau
- [x] Hilfeseite `window-at-a-glance.md` nennt das Feld; Punkt für den Mac in `docs/parity/MAC_PARITY.md`
- [x] `cd app && pnpm test` und `pnpm build` grün, der neue Test einmal rot gesehen

## Umsetzung

- `app/src/logic/buildInfo.ts` (`buildLabel`, Test `buildInfo.test.ts`); `app/vite.config.ts` schreibt `__APP_VERSION__` (aus
  `package.json`), `__APP_COMMIT__` (`git rev-parse --short=7 HEAD`, ohne Git leer) und `__APP_BUILT__` (Zeit des Baus) ins Bündel.
- Feld ganz rechts in `app/src/components/StatusBar.vue`.
