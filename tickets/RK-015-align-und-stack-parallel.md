# RK-015 Align und Stack über die Frames parallel

- Status: erledigt
- Abhängig von: RK-005, RK-006, MAC-003 (`crates/session`)

## Ziel

Align und Stack nutzen alle Kerne, auch wenn eine Aufnahme nur wenige Alignpoints hat. Gemessen am 2026-11-12 mit
`SaturnVideo.avi` (OpenDML, 800×600, 3051 Frames, 4,4 GB, externe USB-Platte, M4 Pro mit 12 Kernen, Release-Bau, Standardwerte,
9 Alignpoints): Align 38,6 s, Stack 84,5 s, Rechenzeit 116 s bei 137 s Laufzeit. Reines Lesen aller Frames dauert etwa 6 s. Heute
verteilt der Kern die Arbeit innerhalb eines Frames auf die Alignpoints (`par_iter` in `crates/core/src/align.rs` und
`stack.rs`); bei 9 Punkten bleibt fast alles auf einem Kern.

Gilt für beide Apps, denn beide rufen denselben Kern über `crates/session`.

## Akzeptanzkriterien

- [x] Align verteilt die Frames auf den Thread-Pool der Session (CPUs-Feld); Lesen bleibt in Aufnahme-Reihenfolge und streamt
      (kein Halten aller Frames im Speicher, Speicher wächst höchstens um eine feste Anzahl Frames je Thread)
- [x] Stack ebenso; das Summenbild ist **bitgleich** zu heute (feste Summationsreihenfolge, z. B. Teilsummen je Block in fester
      Reihenfolge zusammengeführt, oder parallel über Bildzeilen statt über Frames), belegt durch einen Test gegen den bisherigen Pfad
- [x] Fortschritt, Abbrechen und Pausieren verhalten sich wie bisher (bestehende Tests grün, Fortschritt zählt monoton)
- [x] CPUs = 1 ergibt dasselbe Ergebnis wie alle Kerne (Test)
- [x] Messung mit `SaturnVideo.avi` vorher und nachher hier im Ticket eingetragen (Align, Stack, Speicher)
- [x] `cargo test --workspace`, `clippy` ohne Warnungen; Mac-Tests (`swift test`, `xcodebuild test`) und `cd app && pnpm test` grün

## Umsetzung

`crates/core/src/batch.rs`: Die Aufnahme wird der Reihe nach von einem Thread gelesen, in Stapeln von einem Frame je Thread des
Pools (`rayon::current_num_threads()`, also das CPUs-Feld; bei sehr großen Frames weniger, zwei Stapel höchstens 1 GiB). Während
ein Stapel bearbeitet wird, liest `rayon::join` den nächsten. `align_sequence` registriert die Frames eines Stapels parallel
(`par_iter` über die Frames, darin weiter über die Alignpoints). `stack_sequence` registriert den Stapel ebenso parallel und
übergibt ihn dann mit allen Frames in Auswahlreihenfolge an `Stacker::add_aligned`; das addiert parallel nach Bildzeilen, je
Pixel also die Frames in fester Reihenfolge, wie bisher. Dazu bleibt der Frame-Mittelwert für die Normalisierung weg, wenn sie aus
ist (`Stacker::gain`, vorher je Frame umsonst berechnet).

- Fortschritt zählt 1, 2, 3, ..., kommt aber gesammelt nach jedem Stapel; Abbruch und Pause greifen nach einem Stapel. Bei CPUs = 1
  ist der Stapel ein Frame, also wie vorher. Der Mac-Test `cancellingARealAlignLeavesTheStepReadyToRunAgain` setzt deshalb CPUs = 1
  (auf allen Kernen sind die 10 Frames von JUPITER.AVI in einem Stapel fertig, ehe der Test abbrechen kann).
- Save Aligned/Integrated (`aligned_frames`) arbeiten weiter Frame für Frame, sie waren nicht Teil dieses Tickets.
- Bitgleichheit: Das Summenbild entsteht aus denselben Frames in derselben Reihenfolge je Pixel, nur die Registrierung läuft
  parallel und ist je Frame unabhängig und deterministisch. Belegt durch `stack_sequence_is_bit_identical_to_stacking_frame_by_frame_on_any_number_of_threads` (`stack.rs`, 23 farbige Frames mit je eigener Verzerrung und Helligkeit, Pools mit 1, 2, 3 und 8 Threads,
  drei Optionenkombinationen, Vergleich der `f32`-Bitmuster mit dem alten Pfad Frame für Frame), `align_sequence_is_identical_to_registering_frame_by_frame_on_any_number_of_threads` (`align.rs`) und `one_cpu_gives_the_result_of_all_cores` (`crates/session/tests/parallel.rs`,
  CPUs 1, 2, 3 und alle, auch der Fortschritt). Rot gesehen: ein falsch zugeordnetes Alignment im Stapel und ein umgekehrt
  eingetragenes Align-Ergebnis schlagen in allen drei Tests an einer Assertion fehl. Eine bloß umgekehrte Summationsreihenfolge
  blieb dagegen bitgleich, weil die `f64`-Summe von `f32`-Werten in dieser Größenordnung exakt ist; die Reihenfolge ist trotzdem fest.
- Do All auf dem Stack: kein Fehler. Die Standardwerte der Wavelets sind neutral (Regler auf 1,0), dann braucht der Lauf keine
  Schicht und wendet nur Tonkurve und Farbbehandlung auf das Stackbild an (Quelle ist das Stackbild, `Session::stack` setzt sie);
  das sind 2 ms bei 800 x 600. Mit Verstärkung auf Schicht 2 und 3 dauert es 5 ms und das Bild weicht vom Stack ab.

## Messung

`SaturnVideo.avi` (800 x 600, 3051 Frames, USB-Platte), M4 Pro mit 12 Kernen, Release-Bau, Standardwerte, 9 Alignpoints, Limit 80 %
(alle Frames bleiben), ein Probelauf durch die Session (Align, Limit, Stack, Do All), `/usr/bin/time -l`. Die Werte vorher und nachher
stammen vom selben Tag; die vom Anfang des Tickets (38,6 s und 84,5 s) wurden bei einem anderen Lauf gemessen und liegen höher.

| | vorher | nachher |
|---|---|---|
| Align | 27,4 s | 7,2 s |
| Stack | 37,0 s | 8,3 s |
| Gesamtlauf (real) | 65,4 s | 16,4 s |
| Rechenzeit user + sys | 77,4 s + 14,3 s | 78,9 s + 4,9 s |
| Speicher (max. resident) | 84 MB | 235 MB |

Das Stackbild ist bitgleich (gleiche Prüfsumme der 8-Bit-Ausgabe, 135682512). Align liegt jetzt etwa auf der Lesezeit von rund 6 s; den
Rest begrenzt das Lesen, nicht die Rechnung. Der Speicher wächst um zwei Stapel zu je 12 Frames (5,8 MB je Frame als `f32`) und die Felder.
