# MAC-003 Sitzungslogik im Kern (`crates/session`)

- Status: erledigt
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

- [x] `crates/session` ohne Tauri-Abhängigkeit, mit Unit-Tests für Zustandswechsel (Verwerfen, Abbrechen, Pausieren, CPUs)
- [x] Die Tauri-Befehle delegieren an die Session; ihre Signaturen und JSON-Formen bleiben gleich (Vue unverändert)
- [x] Alle bestehenden Tests grün (`cargo test --workspace`, `cd app && pnpm test`), `clippy` ohne Warnungen
- [x] Handprobe-Punkt unter „Windows“ in `docs/parity/Handprobe.md`: ganzer Ablauf im Fenster nach dem Umzug
- [x] Commit `[TASK] MAC-003 - …` mit Hinweis auf die geänderte Kern-Schnittstelle, getrennt vom Mac-Teil

## Umsetzung

`crates/session` (Paket `planets-session`, Lib `planets_session`, erbt die Workspace-Lints, `unsafe_code` verboten; Abhängigkeiten nur
`planets-core`, `planets-io`, `rayon`, `serde`, `thiserror`). Die Tauri-Shell (`app/src-tauri/src/*_commands.rs`, `lib.rs`) enthält
keine Logik und keinen Zustand mehr; `compute.rs` ist in den Crate gewandert.

- **`Session`** (`Send + Sync`, `&self`-Methoden, innere Mutexe): offene Aufnahme, Alignpoints/Align/Limit-Ergebnisse, Stack,
  Wavelet-Quelle und -Ergebnis, Farbquelle, Export-Bilder, Thread-Pool, Abbrechen/Pausieren je Vorgang. Ein Schritt je Methode:
  `open_recording`, `read_frame`, `set_alignpoints`, `set_alignpoint_threshold`, `align`, `limit`, `stack`, `stacked_image`,
  `wavelet_set_source_frame`, `wavelet_preview`, `wavelet_do_all`, `set_color_source`, `estimate_rgb_align`, `apply_colors`,
  `color_histogram`, `auto_histogram_points`, `save_image`, `require_image`, `save_aligned`, `save_integrated`, `cancel`, `pause`,
  `set_cpus`, `cpus`, `cpu_count`. Zustandslose Dinge sind freie Funktionen: `export_formats`, `project_save`, `project_load`,
  `save_wavelet_scheme`, `load_wavelet_scheme`, `wavelet_default_settings`, `with_avi_extension`, `logical_cores`.
- **Verwerfen** (`discard_results`, nicht öffentlich) läuft in `set_alignpoints`, `align` (am Anfang, also auch bei einem abgebrochenen
  Lauf) und `limit`; die Schwellenregler-Bewegung verwirft nichts.
- **Fortschritt:** `trait ProgressSink: Send + Sync { fn progress(&self, operation: Operation, done: u32, total: u32, elapsed_ms: u64) }`
  mit `Operation::{Align, Stack, Sequence, Wavelet}`; Closures und `NoProgress` implementieren es. Die Shell bildet es auf die Ereignisse
  `align-progress`, `stack-progress`, `sequence-progress` (je `{done,total,elapsedMs}`) und `wavelet-progress` (`{done,total}`) ab.
- **Abbrechen/Pausieren:** `cancel(Operation)` und `pause(Operation, bool)`; `Sequence` (Save Aligned/Integrated) teilt die Merker mit
  `Stack`, wie das Fenster es schon tat. Ein Abbruch oder eine Pause von vor dem Lauf wirkt nicht auf den nächsten.
- **Fehler:** `SessionError` (thiserror), die Texte sind die bisherigen des Fensters (Test `tests/contract.rs`); die Shell gibt sie als
  String weiter (`CommandError`).
- **Bilder:** `RgbaImage` (RGBA8 plus `width()`/`height()`), `WaveletPreview` (Region plus RGBA8); die Shell packt sie wie bisher in die
  binäre `Response`.
- **Typen:** Anfragen und Ergebnisse sind einfache Rust-Typen mit den serde-Attributen des Fensters (camelCase), die Befehle reichen sie
  unverändert durch. `tests/contract.rs` sichert diese JSON-Formen.
- Dateidialoge bleiben in der Shell (`export_commands.rs`, `sequence_commands.rs`); sie rufen nur `Session::save_image` usw.

Tests: die 28 Tests der Shell sind mit der Logik in den Crate gezogen (die Shell hat keine eigenen mehr); neu sind 31 Tests in
`crates/session/tests/` (Verwerfen, Abbrechen, Pausieren, CPUs, JSON-Formen und Fehlertexte, ganzer Ablauf auf JUPITER.AVI).
Jeder neue Test war einmal an einer Assertion rot, indem die jeweilige Stelle der Umsetzung kurz kaputtgemacht wurde (Verwerfen
entfernt, Klemmung der CPUs aufgehoben, Abbruch/Pause ignoriert, falscher Merker je Vorgang, camelCase entfernt, Fehlertext geändert).
Der Fensterlauf der Windows-App ist Handprobe am Windows-PC (`docs/parity/Handprobe.md`, „MAC-003 Sitzungslogik im Kern“).
