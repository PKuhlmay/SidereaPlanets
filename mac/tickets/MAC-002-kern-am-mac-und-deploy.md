# MAC-002 Kern am Mac und Deploy der Mac-App

- Status: offen
- Abhängig von: MAC-001 (Klammer)
- Teil von: MAC-001, Kriterium „Der Kern baut und testet am Mac“

## Ziel

Der Rechenkern baut und testet am Mac, die Referenztests finden die Übungsvideos aus `re-local/testdata/astropix/`, und jeder
Commit auf `main` baut und startet am Mac die **Mac-App** statt der Windows-Debug-App.

## Akzeptanzkriterien

- [ ] `cargo test --workspace` und `cargo clippy --workspace --all-targets` laufen am Mac grün und ohne Warnungen (Toolchain aus
      `rust-toolchain.toml` über rustup)
- [ ] Referenztests, die nur die Eingabevideos brauchen, laufen am Mac mit `re-local/testdata/astropix/` (von astropix.com geladen);
      Tests, die die Original-Ergebnisse aus `re-local/reference/` brauchen, überspringen sich mit Hinweis
- [ ] `scripts/deploy-mac.sh`: baut unter macOS `mac/` (xcodegen, `xcodebuild` Debug, arm64) und startet die App neu; Protokoll
      `deploy-mac.log` im Repo-Stamm (gitignored); schlägt der Bau fehl, bleibt die laufende App unangetastet
- [ ] `.githooks/post-commit` ruft unter macOS `deploy-mac.sh`, sonst wie bisher `deploy-debug.sh`; nur Haupt-Checkout auf `main`
- [ ] Solange es `mac/project.yml` noch nicht gibt, meldet das Skript das und beendet sich ohne Fehler
- [ ] `mac/README.md` (Voraussetzungen, rustup über Homebrew ist keg-only: `/opt/homebrew/opt/rustup/bin` in den PATH) und
      `CLAUDE.md` (Deploy am Mac) beschreiben das
