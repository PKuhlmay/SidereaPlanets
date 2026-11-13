# RK-026 Installer wie bei Siderea

- Status: erledigt
- Abhängig von: RK-025
- Stufe: `windows-v0.1.0`

## Ziel

Die Windows-Version kommt als richtiger Installer wie bei Siderea (Philipp, 2026-11-13). Siderea baut sein Setup mit Inno Setup
(`siderea/windows/installer/Siderea.iss`, `windows/scripts/publish.sh --installer`); Siderea Planets bekommt dasselbe Verfahren statt
des NSIS-Bündels von Tauri.

## Akzeptanzkriterien

- [x] `scripts/publish-windows.sh [--installer]` baut die Release-App ohne Tauri-Bündel und legt in `build/windows/dist/` den Ordner
      `Siderea Planets/` (`Siderea Planets.exe`, `CHANGELOG.md`), `Siderea-Planets-<version>-win-x64.zip`, `…-setup.exe` und
      `…-SHA256SUMS.txt` ab; Setup und Prüfsummen zusätzlich in `release/` im Repo-Stamm (nicht in Git)
- [x] Inno-Setup-Skript `app/installer/SidereaPlanets.iss`: eigene, feste AppId; pro Benutzer ohne Adminrechte, auf Wunsch für alle;
      Seite „Was ist neu“ auf Englisch und Deutsch aus dem Abschnitt der Version in `app/CHANGELOG.md`; Startmenü, Desktop-Symbol auf
      Wunsch; schließt die laufende App beim Update; Start nach der Installation; Deinstallation
- [x] WebView2: Setup prüft die Laufzeit und startet Microsofts Bootstrapper nur, wo sie fehlt
- [x] Release-Skill (`.claude/skills/release`, Schritt 3a) und `CLAUDE.md` beschreiben den neuen Weg
- [x] Der Installer baut durch (`Siderea-Planets-0.1.0-setup.exe`, 5,6 MB); Installieren, Starten und Deinstallieren als Probe von Hand
      unter `### RK-026` in `docs/parity/Handprobe.md` eingetragen

## Umsetzung

- `scripts/publish-windows.sh` nach dem Vorbild von Sidereas `publish.sh`: `pnpm tauri build --no-bundle`, die App als
  `Siderea Planets.exe` mit `CHANGELOG.md` im Ordner, ZIP mit dem `tar.exe` von Windows, SHA256SUMS; mit `--installer` die Seiten „Was ist
  neu“ (UTF-8 mit BOM, CRLF) aus `app/CHANGELOG.md`, Microsofts WebView2-Bootstrapper (einmal geholt, in `build/windows/cache/`) und
  `ISCC.exe`. Das Setup liegt danach auch in `release/` (Philipp, 2026-11-13; nicht in Git).
- `app/installer/SidereaPlanets.iss`: AppId `{120B1A26-4EA3-4AB8-BF8A-07BC17678E5A}` (nie ändern), Herausgeber Philipp Kuhlmay, Symbol
  `app/src-tauri/icons/icon.ico`, Windows 10 1809 oder neuer, x64. Die WebView2-Laufzeit wird über die Registry von EdgeUpdate erkannt.
- Anders als Siderea kein Eintrag im Kontextmenü: Siderea Planets nimmt noch keine Datei über die Kommandozeile an.
- Nicht signiert: SmartScreen warnt beim Herunterladen, wie bei Siderea.
