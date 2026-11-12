# MAC-015 CPU- und RAM-Anzeige, schneller Kern im Debug-Build

- Status: erledigt
- Abhängig von: MAC-014

## Ziel

1. **Auslastung in der Statusleiste wie bei Siderea** (Philipp, 2026-11-12): links unten CPU-Last der App mit der Zahl der Kerne
   („CPU (12)“, 100 % = alle Kerne) und belegter Speicher der App. Vorbild: Siderea (`Stackr/App/Views/ResourceReadout.swift`,
   `App/Monitor/ResourceMonitor.swift`, `App/Monitor/DarwinProcessSampler.swift`, `StatusBar.swift`): Messung nur, solange das Fenster
   sichtbar ist, Werte mit Strich, wenn es keine Messung gibt, bei Platzmangel fällt zuerst der Speicher weg statt umzubrechen;
   Schalter in Settings zum Ausblenden. Die Suite soll gleich aussehen.
2. **Align im Debug-Build schneller:** Die Mac-App, die der Deploy startet, ist ein Debug-Build, und `scripts/build-ffi.sh` baut den
   Rust-Kern dafür ohne Optimierung. Der Kern läuft dort ein Vielfaches langsamer als in RK-015 gemessen (Release). Der Kern soll auch
   im Debug-Build optimiert sein (eigenes Cargo-Profil, z. B. `ffi-dev`, erbt `release` ohne LTO, mit `panic = "unwind"`, schneller
   gebaut), Messung vorher und nachher mit `SaturnVideo.avi` in der App (Align, Stack).

## Akzeptanzkriterien

- [x] Statusleiste links: „CPU (n)“ mit Last in % und Speicher der App (wie Siderea, gleiche Schreibweise, Tooltips), Messung pausiert
      im Hintergrund und bei verdecktem Fenster, kein Umbrechen bei schmalem Fenster
- [x] Settings: Schalter „Show CPU and memory usage“ (Standard an)
- [x] Tests für Formatierung und Schwellen (rot zuerst), Messung über eine austauschbare Quelle
- [x] Der Debug-Build der App benutzt einen optimierten Kern; Align und Stack mit `SaturnVideo.avi` vorher und nachher im Ticket
- [x] Punkt in `docs/parity/WINDOWS_PARITY.md` für die Anzeige (Windows: Prozess-CPU und Arbeitsspeicher)
- [x] Anwenderseite (`window-at-a-glance.md`) nennt die Anzeige

## Umsetzung

**Anzeige.** Neu in `mac/App/Monitor/` (aus Siderea portiert, auf CPU und Speicher gekürzt, Kommentare englisch): `ProcessUsage.swift`
(Formate, Schwelle, `ProcessSample`, `ProcessSampling`), `DarwinProcessSampler.swift` (`proc_pid_rusage`), `ResourceMonitor.swift`
(fragt einmal pro Sekunde, Quelle und Takt austauschbar). Die Ansicht `ResourceReadout`/`ResourceField` liegt in
`mac/App/Views/ResourceReadout.swift`; `StatusBar` bekam nur die Zeile `ResourceField()` als erstes Feld (ganz links, vor dem
Fortschrittsbalken, mit Trennlinie). Der Schalter steht als Abschnitt „Status Bar“ in den Settings (`ResourceSettings.swift`,
`@AppStorage("showResourceUsage")`, Standard an, derselbe Schlüssel wie in Siderea). Schreibweise wie Siderea: „CPU (12)“ mit der Last
als Anteil aller Kerne (` 50 %`, bei 100 gedeckelt), „RAM“ mit „1.5 GB“ (1024er Schritte, höchstens 8 Zeichen), Strich ohne Messung,
ab 90 % aller Kerne `--accent` und fett, bei Platzmangel fällt zuerst RAM weg (`ViewThatFits`), nie Umbruch. Gemessen wird nur, wenn der
Schalter an, die Szene nicht im Hintergrund und mindestens ein Fenster sichtbar ist (Okklusionsstatus). Weggelassen gegenüber Siderea:
Projekt- und Frames-Größe (die App hat kein Projektverzeichnis). Eine Abweichung im Verhalten: liefert die Quelle keine Messung,
löscht der Monitor die Werte (Strich), Siderea behält den alten Wert.

**Schneller Kern.** `Cargo.toml` hat das Profil `[profile.ffi-dev]` (erbt `release`, `lto = false`, `codegen-units = 16`,
`panic = "unwind"`, `incremental = true`, `debug = "line-tables-only"`, `strip = false`, jede Wahl dort begründet).
`scripts/build-ffi.sh` ohne `--release` baut damit; die Dateien liegen unter `target/aarch64-apple-darwin/ffi-dev/` (das Skript
liest den Profilnamen als Ordner, `--release` bleibt wie es war). `panic = "unwind"` ist nötig, weil UniFFI Panics abfängt und als Fehler
meldet; `abort` aus `release` würde die App beenden.

## Messung

`SaturnVideo.avi` (800 x 600, 3051 Frames, USB-Platte), M4 Pro mit 12 Kernen, Standardwerte, 9 Alignpoints, Limit 80 %, Standard-CPUs
(alle Kerne). Gemessen mit einer Probe (eine kurze, nicht eingecheckte Rust-Anwendung gegen `planets-session`, dieselbe Kette
wie `crates/session/tests/jupiter_flow.rs`: Öffnen, Set Alignpoints, Align, Limit, Stack) einmal mit dem Profil `dev` (so baute
`build-ffi.sh` bisher) und einmal mit `ffi-dev`. Der Kern ist derselbe, den die Debug-App über die Schnittstelle aufruft.

| | `dev` (vorher) | `ffi-dev` (nachher) | Faktor |
|---|---|---|---|
| Align | 408,8 s | 7,2 s | 57 |
| Stack | 297,4 s | 10,6 s | 28 |
| Gesamtlauf (real) | 707,9 s | 18,8 s | 38 |
| Speicher (max. resident) | 211 MB | 230 MB | |
| Build des Kerns, sauber (`cargo build -p planets-ffi`) | 33 s | 103 s | 3,1 mal länger |
| `scripts/build-ffi.sh` sauber, gesamt | 79 s | 108 s (ohne bindgen, s. u.) | |
| Neu bauen nach Änderung an `crates/core` (Zeitstempel geändert) | 4,1 s | 2,9 s | |

Das Stackbild ist in beiden Läufen gleich (Prüfsumme der 8-Bit-Ausgabe 135682512, wie in RK-015). Zum Vergleich der Release-Bau aus
RK-015: Align 7,2 s, Stack 8,3 s; `ffi-dev` liegt damit bei Align gleichauf und bei Stack 28 % darüber. Zum Bau: Der saubere Kernbau
dauert dreimal so lange, ein Neubau nach einer Änderung nicht (der Zeitstempel-Test ändert nur die Datei, er misst also den
Verwaltungsaufwand, nicht eine echte Änderung). Der Gesamtlauf des Skripts „vorher“ enthielt die Übersetzung von `uniffi-bindgen`
(läuft weiter im Profil `dev`, rund 45 s); im Lauf „nachher“ lag sie schon im Cache, ein sauberer Lauf wäre also rund 150 s lang
(geschätzt, nicht gemessen). Gemessen wurde auf einem Mac, an dem zu der Zeit ein anderer Agent arbeitete.

## Prüfung

- `cargo clippy --workspace --all-targets` ohne Warnungen; `cargo test --workspace` grün; `xcodebuild test` (App) 259 Tests (nach dem Rebase auf MAC-014)
  grün (davor 225 plus die neuen und die von MAC-014).
- Neue Tests: `ProcessUsageTests` (Last zwischen zwei Messungen, Text als Anteil aller Kerne, Deckel, Strich ohne Messung, Bytes-Text,
  Länge höchstens 8, Schwelle 90 %, Messbedingungen, Standard an) und `ResourceMonitorTests` (erste und zweite Messung, fehlende Messung,
  Stop löscht, Start/Stop mit eingespielter Quelle und kurzem Takt).
- Rot zuerst: mit drei absichtlichen Fehlern im Code (Last nicht durch die Kernzahl geteilt, Schwelle 0,8 statt 0,9, Monitor behält den
  alten Wert bei fehlender Messung) schlugen 6 Prüfungen in 3 Tests an Assertions fehl (`cpuReading` 3, `isHigh` 2, fehlende
  Messung 1); danach zurückgesetzt, alles grün.
