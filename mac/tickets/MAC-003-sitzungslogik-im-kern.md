# MAC-003 Sitzungslogik im Kern (`crates/session`)

- Status: offen
- Abhängig von: MAC-002
- Teil von: MAC-001, Kriterium „`crates/session` enthält die Sitzungslogik“

## Ziel

Der Zustand, der heute in den Tauri-Befehlen steckt (`app/src-tauri/src/*_commands.rs`, `compute.rs`, `lib.rs`), zieht in einen
GUI-freien Crate `crates/session`. Die Tauri-Shell wird zur dünnen Hülle darüber; die Mac-Schnittstelle (MAC-004) nutzt denselben
Crate. Verhalten der Windows-App bleibt gleich.

## Umfang

- Eine `Session` mit offener Aufnahme, Alignpoints/Align/Limit-Ergebnissen, Stack, Wavelet-Quelle und -Ergebnis, Farbquelle,
  Export-Bildern; `discard_results` (Verwerfen nach neuem Set Alignpoints/Align/Limit) gehört in die Session
- Thread-Pool der CPU-Auswahl (`compute.rs`), Abbrechen und Pausieren je Vorgang, Fortschritt über einen Callback (Trait), den
  Tauri auf Events und die Mac-Schnittstelle auf Swift abbildet
- Projektdatei, Schemes, Save Aligned/Integrated, Export bleiben in `crates/io`, die Session ruft sie

## Akzeptanzkriterien

- [ ] `crates/session` ohne Tauri-Abhängigkeit, mit Unit-Tests für Zustandswechsel (Verwerfen, Abbrechen, Pausieren, CPUs)
- [ ] Die Tauri-Befehle delegieren an die Session; ihre Signaturen und JSON-Formen bleiben gleich (Vue unverändert)
- [ ] Alle bestehenden Tests grün (`cargo test --workspace`, `cd app && pnpm test`), `clippy` ohne Warnungen
- [ ] Handprobe-Punkt unter „Windows“ in `docs/parity/Handprobe.md`: ganzer Ablauf im Fenster nach dem Umzug
- [ ] Commit `[TASK] MAC-003 - …` mit Hinweis auf die geänderte Kern-Schnittstelle, getrennt vom Mac-Teil
