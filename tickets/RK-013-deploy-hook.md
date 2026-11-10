# RK-013 Debug-App nach jedem Commit bereitstellen

- Status: erledigt
- Abhängig von: —

## Ziel
Nach jedem Commit auf `main` läuft die Debug-App mit dem letzten Stand, ohne dass jemand daran denken muss. Bisher stand das als Regel in der `CLAUDE.md`; jetzt erledigt es ein Git-Hook im Repo, auch für Commits anderer Agenten.

## Akzeptanzkriterien
- [x] `.githooks/post-commit` ruft `scripts/deploy-debug.sh` auf und blockiert den Commit nie
- [x] Nur Commits im Haupt-Checkout auf `main` lösen aus; Worktrees und andere Branches nicht
- [x] Das Skript installiert Abhängigkeiten (`pnpm install --frozen-lockfile`), baut das Frontend (`pnpm build`) und startet `pnpm tauri dev`, falls noch keine Sitzung läuft; läuft eine, übernimmt sie die Änderung selbst
- [x] Schlägt der Bau fehl, wird nicht deployt und der Grund steht in `deploy-debug.log`
- [x] Gleichzeitige Commits führen nicht zu zwei parallelen Deploys (Sperre, ein Nachlauf)
- [x] Tests laufen nur mit `DEPLOY_TESTS=1` (vorübergehend aus, siehe `CLAUDE.md`)
- [x] `CLAUDE.md` beschreibt Hook, Aktivierung (`git config core.hooksPath .githooks`) und Protokolldateien
- [x] Ein Commit auf `main` startet die App tatsächlich (geprüft)
