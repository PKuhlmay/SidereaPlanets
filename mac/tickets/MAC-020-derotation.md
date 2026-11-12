# MAC-020 Derotation beim Stacken

- Status: erledigt
- Abhängig von: RK-015, MAC-007

## Ziel

Planeten drehen sich während der Aufnahme (Jupiter in rund 9 h 55 min einmal). Bei Videos über etwa eine bis zwei Minuten
verschmieren die Details zum Rand der Scheibe hin; die Alignpoints gleichen nur kleine Verschiebungen aus. Die gesperrte Option
**De-rotate images** im Schritt Stack wird gebaut (Philipp, 2026-11-12): Jeder Frame wird vor dem Stacken auf eine gemeinsame
Bezugszeit zurückgedreht. Teleskopdaten sind nicht nötig.

## Entwurf (eigene Umsetzung, in `docs/stacking.md` als Vermutung markieren, bis ein Vergleich sie bestätigt)

- **Planet:** Jupiter (System II, 9 h 55 min 40,6 s), Saturn (System III, 10 h 33 min 38 s), Mars (24 h 37 min 22,7 s). Auswahl im
  Panel, Vorschlag aus dem Dateinamen bzw. den SER-Kopfdaten, wenn erkennbar.
- **Zeit je Frame:** SER-Zeitstempel je Frame, wenn vorhanden; sonst Startzeit (Feld, Vorschlag aus den Dateidaten) plus Frame-Nummer
  durch Bildrate. Bezugszeit: Mitte der ausgewählten Frames.
- **Geometrie:** Lage der Drehachse (Positionswinkel P, Neigung D_E) und Abplattung aus einer eingebauten, vereinfachten Ephemeride
  (Meeus, „Astronomical Algorithms“, Kapitel 42 Mars, 43 Jupiter, 45 Saturn; Genauigkeit ein Grad reicht). Position und Größe der Scheibe
  werden im Referenzbild gemessen (Schwelle und Ellipsenanpassung); der Nutzer kann P zusätzlich drehen, falls die Kamera gedreht war
  (Feld „Camera angle“, Standard 0, Hinweis: Norden oben).
- **Rechnung:** Für jeden Pixel der Scheibe Länge und Breite auf dem abgeplatteten Kugelmodell (orthografische Projektion) bestimmen, um
  Δλ = 360° · Δt / Rotationsdauer verschieben, zurückprojizieren und bilinear abtasten; außerhalb der Scheibe bleibt der Pixel. Läuft vor
  der lokalen Ausrichtung jedes Frames im Batch (RK-015), parallel, bitgleich für Δt = 0.

## Akzeptanzkriterien

- [x] Kern (`crates/core`): Ephemeride (P, D_E, Abplattung) für Jupiter, Saturn, Mars mit Tests gegen Werte aus Meeus' Beispielen;
      Scheibenmessung; Derotation eines Frames
- [x] Synthetischer Test: eine texturierte, abgeplattete Kugel zu zwei Zeiten gerendert; der derotierte spätere Frame gleicht dem
      früheren innerhalb einer Schwelle (PSNR), Δt = 0 ist bitgleich
- [x] SER-Zeitstempel werden gelesen und genutzt; ohne Zeitstempel Startzeit plus Bildrate
- [x] Session/FFI: Stack-Einstellungen bekommen die Derotation (optional, Standard aus; Tauri baut weiter)
- [x] Mac: **De-rotate images** freigeschaltet, mit Planet, Startzeit (bei AVI), Camera angle, Anzeige der gemessenen Scheibe als
      Overlay und der Zeitspanne der Aufnahme („Span 3 min 12 s, rotation 5.8°“); Hinweis, wenn die Spanne so kurz ist, dass es nichts bringt
- [x] Projektdatei speichert die Werte (optionaler Abschnitt)
- [x] Laufzeit mit `SaturnVideo.avi` vorher und nachher im Ticket
- [x] Hilfe (Schritt Stack, Glossar), `docs/stacking.md`, `docs/design.md`, `docs/parity/WINDOWS_PARITY.md`
- [x] Tests grün, neue Tests einmal rot an einer Assertion

## Umsetzung

**Kern.** `crates/core/src/planet.rs`: `Planet` (Jupiter, Saturn, Mars: Periode, Abplattung, Ring), `Planet::pole_geometry(unix)` mit `D_E`, `P` und Abstand aus den mittleren
Bahnelementen (Meeus, Tabelle 31.A), Lichtlaufzeit und IAU-Pol; `unix_seconds` für Kalenderdaten. `crates/core/src/derotate.rs`: `View` (Achse, Kamerawinkel,
Spiegelung), `measure_disc` (Otsu, größter Fleck, Löcher gefüllt, Randpunkte, RANSAC und Tukey-Anpassung der Ellipse, Saturn: Kugel und Ringkante gemeinsam),
`Derotator::derotate(frame, Sekunden)` (Ellipsoid, Ring, Δt = 0 gibt den Frame bitgleich zurück), `View::shape` für das Overlay. `stack.rs`:
`stack_sequence_derotated` mit `Derotation { derotator, offsets }`, im Batch parallel vor `align_frame`; `stack_sequence` ruft es mit `None`. `synthetic.rs` (nur Tests,
Feature `synthetic-planet` für die Sitzungstests): die texturierte, abgeplattete, rotierende Kugel, per Strahlverfolgung im Kamerasystem gerendert, damit der Test die Formeln
des Moduls nicht nachrechnet.

**Zeiten.** `crates/io/src/ser.rs`: Nachspann (ein 64-Bit-Tickwert je Frame, UTC) und `DateTimeUTC` des Kopfes (`SerReader::timestamps`, `start_time`); unplausible oder nur aus Nullen bestehende
Werte zählen als fehlend. `recording.rs`: `RecordingInfo.has_timestamps`, `start_time`, `start_time_from_file_date` (Änderungszeit der Datei minus Länge, nur AVI mit Bildrate).
`crates/session/src/derotate.rs`: `DerotationSettings`, `Session::derotation_info`, der Plan fürs Stacken (Zeit je Frame, Bezugszeit = Mitte zwischen erstem und letztem behaltenem Frame; das
Bezugsframe wird auf diese Zeit gedreht, bevor das `Reference` entsteht). FFI: `DerotationSettings`, `DerotatePlanet`, `DerotationInfo`, `DiscShape`, `CoreSession.derotation_info`,
`derotation_planet_from_name`, `StackSettings.derotation` und die neuen Felder von `RecordingInfo` mit Standardwerten (der Swift-Aufruf der alten Form bleibt gültig). Tauri: unverändert, `derotation` hat `#[serde(default)]`.

**Mac.** `StackLogic.swift` (Optionen, `derotationSettings`, `spanText`, `derotationSummary`, Abschnitt `derotation`), `AppModel+Derotation.swift` (Vorschläge beim Öffnen,
`refreshDerotation` verzögert um 120 ms), `StackPanel.swift` (`DerotateOptions`), `ImageOverlays.swift` (`drawDisc`), `DebugLaunch.swift` (`-derotate`). Aufbau und Texte in `docs/design.md`,
Abschnitt „Derotation (MAC-020)“.

**Entscheidungen.**

- *Derotation vor der Ausrichtung, nicht danach:* würde man die Scheibe erst nach dem Registrieren drehen, folgten die Alignpoints und das lokale Feld der Drehung selbst (die Merkmale am
  Rand wandern), und die Derotation käme obendrauf. Darum wird je Frame in dessen eigener Lage gedreht (Mitte aus dem Schwerpunkt des Flecks, siehe `docs/stacking.md`) und danach registriert.
- *Bezugsframe mitdrehen:* sonst liegt das Referenzbild der Alignpoints auf einer anderen Zeit als die Frames.
- *Spiegelbild:* nicht im Ticket; ohne `Mirrored image` wäre ein Bild mit einem Spiegel im Strahlengang falsch herum gedreht (Test `camera_angle_and_mirror_are_followed`). Die Einstellung steht im Panel und im
  Abschnitt `derotation` der Projektdatei (`mirrored`).
- *Startzeit:* Feld in der Ortszeit des Macs (Minuten genau, mehr braucht die Achse nicht), darunter dieselbe Zeit in UTC. Der Vorschlag aus dem Dateidatum ist eine Schätzung und sagt das.
- *Zeitbasis UTC statt TT:* 69 s, ohne Wirkung auf die Achse.
- *Save Aligned und Save Integrated derotieren nicht* (die Videos sind eigene Ergebnisse mit eigenen Zeiten; bei Bedarf ein Folgeticket).
- *Das Panel zeigt das Bezugsframe,* sobald die Scheibe gemessen ist (dort wurde gemessen); das Overlay erscheint auch auf dem fertigen Stack, dessen Lage die des Bezugsframes ist.

## Prüfung

- `cargo fmt --all`, `cargo clippy --workspace --all-targets` (0 Warnungen), `cargo test --workspace`, `scripts/build-ffi.sh`, `swift test` im Paket, `xcodebuild test` (App, 414 Tests),
  `pnpm test` im Ordner `app` (243 Tests).
- **Ephemeride gegen JPL Horizons** (30 Fälle: 3 Planeten × 10 Daten von 1992 bis 2030): größte Abweichung **0,23° bei `D_E`** (Saturn 12.11.2026) und **0,32° bei `P`**; Jupiter unter 0,02° bei `D_E`.
  Meeus' Beispiele (Jupiter und Saturn 16.12.1992, Mars 9.11.1992) innerhalb 0,15° bis 0,3°; das Buch lag nicht vor, die Beispielwerte sind deshalb gegen Horizons abgeglichen (Meeus' `D_E` ist die Breite des
  Richtungsvektors, Horizons' „ObsSub-LAT“ die planetographische Breite der Scheibenmitte: `tan D_E = (1 − f)² tan φ`).
- **Synthetische Kugel** (PSNR innerhalb der Scheibe gegen das Bild zur Bezugszeit): Jupiter 5 min später **43,8 dB** mit Derotation gegen **30,5 dB** ohne; Saturn mit Ring 10 min später **34,3 dB** gegen **26,2 dB**;
  Stack aus drei Jupiterframes über 10 min **58,7 dB** gegen **37,7 dB**; über die Sitzung mit SER-Zeitstempeln **52,1 dB** gegen **35,6 dB**. Δt = 0 ist bitgleich.
- **Rot zuerst, Rust:**
  - `planet.rs`: `pole_geometry` als Stub mit Nullen → `meeus_worked_examples` (Jupiter `D_E` 0 statt −2,48) und `agrees_with_jpl_horizons_over_forty_years` schlugen an Assertions fehl; die Saturn-Periode im Test
    war zuerst falsch gerechnet (34,14 statt 34,09 °/h) und fiel an `rotation_follows_the_period…` auf.
  - `derotate.rs`: (a) `derotate` gibt den Frame unverändert zurück → 5 Tests rot (`a_five_minute_rotation_of_jupiter_is_taken_back`, `the_wrong_direction_is_clearly_worse`, `a_frame_that_drifted_…`,
    `camera_angle_and_mirror_are_followed`, `saturn_is_turned_…`); (b) Vorzeichen der Drehung umgekehrt → 6 rot; (c) Scheibe = Schwerpunkt mit Radius 10 → 7 rot (u. a. Mars mit Terminator, Rauschen und Mond);
    (d) Ring-Ausnahme abgeschaltet → `saturn_is_turned_and_its_rings_are_left_alone` rot.
  - `stack.rs`: `derotation` im Batch ignoriert → `de_rotated_frames_stack_to_the_reference_time_not_a_smear` rot (37,66 dB gleich 37,66 dB).
  - `ser.rs`: Nachspann nicht gelesen → `reads_the_frame_timestamps_of_the_trailer` rot.
  - Sitzung (`crates/session/tests/derotation.rs`): Derotation in `Session::stack` ausgeschaltet → `the_time_stamps_of_a_ser_file_turn_the_frames_back` und
    `without_time_stamps_the_start_time_and_the_frame_rate_decide` rot. Nicht einzeln rot geprüft: Umwandlungen im FFI (reine Feldkopien) und `start_time_from_file_date`.
- **Rot zuerst, Swift** (Stubs: `derotationSettings` immer `nil`, Spanne ohne Sekunden, Hinweis aus, Bezugsframe nicht gezeigt, Abschnitt nicht geschrieben, Winkel nicht begrenzt): 24 Prüfungen in `StackLogicTests`
  und `DerotationModelTests` schlugen an Assertions fehl. Nachträglich verändert (Spanne unter 10 s mit einer Nachkommastelle, `spanText(seconds: 0) == "0.0 s"`) und nur durch Lesen des alten Stands als rot
  bestätigt, nicht erneut gelaufen. Swift-Paket: `DerotationFlowTests` mit der echten Kette (JUPITER.AVI).
- **Rot zuerst, Doku-Test:** `stack-pages.test.ts` gegen die alte Seite `step-stack.md` (Zeile „Not available yet“) schlug an der neuen Prüfung fehl.

## Laufzeit mit `SaturnVideo.avi`

800 × 600, 3051 Frames, 30 fps (101,7 s), ohne Zeitstempel (Startzeit angegeben), alle Frames, 12 Kerne, Profil `ffi-dev` (die Messung lief mit einer Wegwerf-Sonde, nicht im Repo), 10 Alignpoints, *Correct geometry* an:

| Lauf | ohne Derotation | mit Derotation |
|---|---|---|
| 1 | 8,5 s | 10,7 s |
| 2 | 11,0 s | 9,7 s |

Der Unterschied liegt im Rauschen (Plattenzugriff, die Datei hat 4,4 GB); Der Aufschlag der Derotation (Schwelle und Fleck je Frame, Drehung der Scheibe von etwa 55 px Radius) liegt unter dem Rauschen dieser Messung. Saturn dreht sich in diesen 101 s nur um 0,9°,
das Panel sagt „Too short to matter“. Die gemessene Scheibe liegt im Bezugsframe auf dem Planeten (Ellipse für die Kugel, gestrichelt der Ring; mit dem vorgeschlagenen Datum, dem Dateidatum, nicht dem der Aufnahme: die
Achsenneigung stimmt dann nicht).

## Grenzen

- Nur Jupiter, Saturn, Mars; Perioden: System II, System III. Die Achse ist auf etwa ein halbes Grad genau (mittlere Bahnelemente, keine Störungen); sie braucht Datum und Uhrzeit auf wenige Minuten.
- Wind und Schatten (Ring auf Kugel, Kugel auf Ring) fehlen; Merkmale, die gerade um den Rand gedreht sind, haben keine Quelle, ihre Randpixel bleiben.
- Saturn: die Ringkante muss sich von der Kugel abheben; der Radius der Kugel hat ± 1–2 % Unsicherheit, bei fast kantenständigem Ring ist die Ringellipse nur eine Linie.
- Mars mit Phase: gemessen wird die beleuchtete Kante, der Terminator zieht nicht (RANSAC), bei sehr schmaler Sichel fehlt aber die Kante.
- Gegen das Original nicht verglichen: das Original rechnet es vermutlich anders, das Verfahren steht in `docs/stacking.md` als Vermutung.

