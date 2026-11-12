# Stacking (RK-006)

Was der Klon beim Stacken tut, warum, und wie weit das gegen the reference program geprüft ist. Bedienelemente und Standardwerte
stehen in `docs/ui.md` (Stack-Reiter), der Ablauf in `docs/workflow.md`, Referenzfall R1 in `docs/testdaten.md`.
Alles, was nur geraten ist, steht als **Vermutung** da.

## Stand

Rechenkern, Tauri-Befehle und Anbindung an den Stack-Reiter sind fertig. Die Tests in `stack.rs` (18) laufen grün; der
Vergleichstest gegen das Original erreicht **54,85 dB** (Schwelle 45 dB), siehe „Vergleich mit dem Original“. Die beiden neuen Tests
der Untersuchung (`scatter_of_the_points_does_not_distort_the_frame`, `matches_at_the_limit_of_the_search_range_are_not_followed`)
waren gegen den Stand davor rot.

## Beobachtet und geschlossen

Aus `docs/workflow.md`, `docs/ui.md` und `docs/testdaten.md`:

- Das Original stackt im Schritt **Stack** nach Align und Limit. Ausgerichtet wird beim Stacken selbst, es gibt keinen
  eigenen Optimierungsschritt mehr.
- Optionen im Panel *Stacking options*: *Use Nearestby Alignpoints* = 3, *Correct geometry* an, *Normalization of
  frame intensity* aus, *Do not stack n pixels at the frameedge* = 0.
- Das Ergebnis ist ein **Mittelwert**, keine Summe: Die Kanalmittel des Summenbilds (R 6674, G 5131, B 6100 in 16 Bit
  = 8 Bit × 256) liegen im Bereich der Einzelframes. „Stacking Range : 0-255“ nach dem Stacken.
- Das Ergebnis weicht deutlich vom einfachen Mittelwert ohne Ausrichtung ab (38,8 dB) und liegt näher an Frame 1
  (44,0 dB). Das stützt eine lokale Verschiebung vor dem Mitteln.

## Das Verfahren des Klons

Umgesetzt in `crates/core/src/stack.rs`. Alles darin außer „Mittelwert“ ist **Vermutung**, solange der Vergleich fehlt.

1. **Registrierung.** Jeder Frame wird mit `Reference::align_frame` (RK-005) gegen das Bezugsframe registriert. Das liefert
   je Alignpoint die Verschiebung `(dx, dy)`: Dort, wo das Bezugsframe an `p` etwas zeigt, zeigt der Frame es bei
   `p + (dx, dy)`. Punkte, die nicht verfolgt werden konnten, fehlen und zählen nicht, ebenso Treffer am Rand des
   Suchbereichs (`at_limit`): Sie sagen nur, dass die Bewegung mindestens so groß ist.
2. **Verschiebungsfeld** (bei *Correct geometry* an). Für jeden Pixel werden die *n* nächsten Alignpoints
   (*Use Nearestby Alignpoints*) gemittelt, mit Gewichten, die zum (n+1)-ten Punkt hin auf null fallen, damit das Feld
   nicht springt, wenn sich die Menge der nächsten Punkte ändert. Ausgewertet wird das Feld auf einem Raster von 4 px und
   dazwischen bilinear interpoliert, das spart bei tausenden Frames die Suche nach den nächsten Punkten pro Pixel.
   Bei *Correct geometry* aus gilt für den ganzen Frame der Medianversatz.
   **Vertrauen in die Punkte:** Das Feld folgt den Punkten nur so weit, wie ihre Abweichungen vom Frameversatz räumlich
   zusammenhängen. Jeder Punkt wird aus den anderen vorhergesagt (wie ein Pixel) und aus dem Frameversatz allein; das Vertrauen
   ist `1 − Fehler(Nachbarn) / Fehler(Frameversatz)`, auf 0..1 begrenzt (`coherence` in `stack.rs`), und das Feld ist
   `Frameversatz + Vertrauen × (Nachbarmischung − Frameversatz)`. Bei einer echten Verzerrung liegt es nahe 1, bei Streuung
   der Zuordnung bei 0, dann bewegt sich der Frame als Ganzes. Das ist eine eigene Entscheidung des Klons, nicht beobachtet.
3. **Abtasten.** Der Frame wird an der verschobenen Stelle bilinear abgetastet (Vermutung: Das Original interpoliert
   vielleicht anders, etwa bikubisch oder mit ganzzahligen Verschiebungen; bilinear glättet etwas).
4. **Mitteln.** Je Pixel werden die Werte aller Frames summiert, die den Pixel abdecken, und durch deren Anzahl geteilt.
   Am Rand, den nicht jeder Frame erreicht, gilt der Mittelwert der abdeckenden Frames. Ein Pixel ohne Abdeckung bleibt schwarz.
5. **Normalisierung** (Option): Jeder Frame wird mit `Mittel(Bezugsframe) / Mittel(Frame)` skaliert.
6. **Randabstand** (Option): Abtaststellen näher als *n* Pixel am Rand des Frames werden ausgelassen.

### Genauigkeit und Speicher

- Die Summe liegt in `f64`, die Zählung in `u32` je Pixel. Samples sind in `f32` auf `0..1` normiert; tausende 16-Bit-Frames
  laufen ohne Überlauf und ohne sichtbares Rundungsrauschen auf (Test mit 70 000 Frames voller Aussteuerung).
  Der Preis ist Speicher: 8 Byte je Messwert, bei 4096 × 4096 RGB etwa 400 MB. Bei Bedarf auf `f32` mit Blocksummen umstellen.
- Gestreamt wird Frame für Frame: lesen (die Datei wird nur von einem Thread gelesen), dann registrieren parallel über die
  Alignpoints (`rayon`) und addieren parallel nach Bildzeilen. Fortschritt und Abbruch/Pause greifen so nach jedem Frame
  (RK-005-Prüflauf: vorher Stapel von `2 × Threads` Frames, bei 10 Frames kam der Fortschritt einmal am Ende und ein Abbruch
  erst nach dem ganzen Lauf). Gleiches gilt für Align und für Save Aligned/Integrated. Im Speicher liegen nur der Stapel
  und die Summen.
- Frames anderer Größe oder Kanalzahl werden ausgelassen und nicht mitgezählt (`StackResult::frames`).

### Schnittstelle für Folgetickets

- `planets_core::stack::stack_sequence(...) -> StackResult { image: Image, frames: u32 }`. Das Ergebnis ist der
  gewöhnliche Bildtyp `Image` (`f32`, `0.0..=1.0`, verschachtelte Kanäle), dieselbe Größe und Kanalzahl wie das Bezugsframe.
- `Stacker` (`new`, `add_aligned`, `finish`) nimmt Frames samt schon berechneter `FrameAlignment` entgegen. Damit lässt sich
  später das Ergebnis von *Align* wiederverwenden statt neu zu registrieren; `stack_sequence` registriert derzeit selbst.
- `stack::psnr(a, b)` ist die Messgröße aus `docs/testdaten.md` (Spitzenwert 1,0, bei 16 Bit also 65535).
- Frontend: `app/src/stack.ts` (Handler `stack` in `app/src/actions.ts`, über `runJob`), Anfrage aus dem Fensterzustand in
  `app/src/logic/stack.ts`. Das Bezugsframe steht in `session.referenceFrame`; *Set Alignpoints* (RK-005) muss es setzen,
  bis dahin gilt Frame 0.
- Sitzung: `planets_session::Session::stack` und `stacked_image` (`crates/session/src/stack.rs`) halten das letzte Summenbild; die
  Tauri-Befehle `stack`, `stacked_image`, `cancel_stacking`, `pause_stacking` sind dünne Hüllen darüber, Ereignis `stack-progress`.

## Vergleich mit dem Original

### Vorgehen

Test: `crates/io/tests/stack_reference.rs`. Er braucht `re-local/testdata/astropix/JUPITER.AVI` und
`re-local/reference/jupiter-stack.tif` und wird ohne sie übersprungen (in einem Worktree fehlt `re-local/`, also im
Haupt-Checkout laufen lassen).

1. Referenzfall R1: Alignpoints auf Frame 1 mit Standardwerten setzen und Standard-Schwelle, registrieren, alle 10 Frames
   stacken (Limit behält bei R1 alle).
2. `jupiter-stack.tif` laden (Werte ÷ 256 ÷ 255 auf die Skala 0..1; 16 Bit = 8 Bit × 256, siehe `docs/testdaten.md`).
3. PSNR des Klon-Ergebnisses gegen das TIFF, Spitzenwert 1,0. Schwelle laut `docs/testdaten.md`: **≥ 45 dB**.
4. Zur Einordnung werden mitgemessen: PSNR bei *Correct geometry* aus (nur Frameversatz), PSNR von Frame 1 gegen das
   TIFF und die Kanalmittel beider Bilder.

### Messwerte

Lauf am 2026-11-11 mit `cargo test -p planets-io --test stack_reference --release -- --nocapture`.

| Messung | erster Stand | jetzt |
|---|---|---|
| Klon (Standardwerte) gegen `jupiter-stack.tif` | 43,92 dB | **54,85 dB** (Schwelle 45 dB erreicht) |
| Klon mit *Correct geometry* aus (nur Frameversatz) | 54,85 dB | 54,85 dB |
| Frame 1 gegen `jupiter-stack.tif` | 44,03 dB | 44,03 dB (Original-BMP gegen Frame 1: 43,99 dB) |
| einfacher Mittelwert ohne Ausrichtung gegen `jupiter-stack.tif` | 38,83 dB | 38,83 dB (Original-BMP gegen Mittelwert: 38,77 dB) |
| Kanalmittel R / G / B, Original 6674,1 / 5130,5 / 6099,5 | 6588 / 5064 / 6023 (−1,3 %) | 6698,6 / 5149,4 / 6121,7 (+0,36 %) |

### Ursachenanalyse

1. **Helligkeit.** Der erste Stand war in allen drei Kanälen gleichmäßig 1,3 % dunkler als das Original, die Variante mit nur
   Frameversatz dagegen 0,36 % heller. Die Abweichung kam also aus der lokalen Korrektur, nicht aus Abtastung oder Rand: Folgten
   die Punkte Zufallsversätzen, wurde Bildinhalt aus hellen Bereichen weggeschoben. Mit dem Vertrauen ist sie weg.
2. **Warum die lokale Korrektur schlechter war als der Frameversatz.** Die Versätze der einzelnen Punkte streuen gegenüber dem
   Frameversatz um median 3,4 bis 4,4 px (90 %-Wert 5,8 bis 6,7 px), 20 bis 30 von 78 Punkten liegen am Rand des Suchbereichs
   von ±5 px, und benachbarte Punkte haben unabhängige Abweichungen. Die Korrelationswerte sind dabei hoch (im Mittel 0,88): Das
   Bild ist weich, die Korrelation hat einen breiten Rücken, und das Maximum landet irgendwo darauf. Eine so große, unzusammenhängende
   Verzerrung ist bei einem Planeten nicht echt. Das Original stackt von denselben Frames ein Bild, das dem reinen Frameversatz auf
   54,85 dB gleicht; seine lokale Korrektur ist also klein oder wirkt auf verlässlichere Punkte.
3. **Größe der Nachbarschaft** (vor der Änderung, nur Messung): *n* = 1, 3, 6, 12, 40 ergab 43,47, 43,92, 44,27, 44,74, 45,81 dB.
   Je mehr geglättet wird, desto besser, die Grenze ist der Frameversatz. Das bestätigt die Streuung als Ursache.
4. **Abtastung und Gewichtung** wurden nicht geändert: Mit dem reinen Frameversatz sind 54,85 dB erreicht, bikubische Abtastung
   oder andere Gewichte wurden nicht versucht.
5. **Randbehandlung** wurde nicht verändert; bei R1 verschiebt sich das Bild um bis zu 9 px, der Rand betrifft einen kleinen
   Teil des schwarzen Hintergrunds.
6. Treffer am Rand des Suchbereichs auszulassen ändert bei R1 fast nichts (54,85 gegen 54,85 dB bei *n* = 3, 54,91 gegen 54,79 dB
   bei *n* = 40, mit gegen ohne Filter); es bleibt, weil `PointShift::at_limit` ausdrücklich sagt, dass die Bewegung dort größer
   sein kann.

**Was das für R1 heißt:** Mit dem Vertrauen liefert die Standardeinstellung bei R1 den Frameversatz-Stack (das Vertrauen
liegt nahe 0). Die lokale Korrektur ist im Kern umgesetzt und an einer künstlichen, zusammenhängenden Verzerrung geprüft
(`local_correction_follows_a_coherent_distortion`), sie wirkt aber bei R1 praktisch nicht. Ob das Original wirklich lokal
korrigiert, ist offen: Bei R1 wäre der Unterschied zum reinen Frameversatz kleiner, als 54,85 dB zeigen können. Ein Referenzfall
mit starker Luftunruhe (RK-011) würde das klären. Die Ursache der Streuung liegt in der Zuordnung der Punkte (RK-005: weiche
Strukturen, breiter Korrelationsrücken), nicht im Stacken; eine robustere Zuordnung (Punkte nach Schärfe der Korrelationsspitze
wählen oder gewichten) würde die lokale Korrektur erst nutzbar machen.

## Nicht umgesetzt

Nicht Teil der Akzeptanzkriterien und nie beobachtet; jeweils nur der Name bekannt:

- *StackSize* (Minimized, Maximized, R.of Interest), *Stretch histogram after stacking*, *De-rotate images*. In der App
  sind sie gesperrt und mit „Not available yet“ gekennzeichnet (Default bei StackSize bleibt wählbar und ist das, was der
  Klon tut); die Einstellungen `stackSize`, `stretchHistogram` und `derotate` bleiben im Datenmodell, sind aber nicht Teil
  von `StackRequest` und das Backend kennt sie nicht.
- **Resampling** (Bell, Bspline, Lanczos, Mitchell, Faktor) und **Drizzling**, die das Ergebnis vergrößern. In der App sind
  beide Häkchen gesperrt und mit „Not available yet“ gekennzeichnet; die Einstellungen `resampling`/`drizzling` bleiben im
  Datenmodell, das Backend ignoriert sie.
- **Create AVI** (*Save Aligned*, *Save Integrated*): in RK-009 gebaut, siehe `docs/formats.md`, Abschnitt „Create AVI“.
- Stackgraph und Stacklist.

## Offen

- Ob das Original lokal korrigiert und wie stark; bei R1 nicht vom Frameversatz zu unterscheiden (siehe Ursachenanalyse).
- Interpolation und Gewichtung der nächsten Punkte: bei R1 ohne Wirkung (nur Frameversatz). Das gewichtete Mittel der nächsten
  Punkte gibt ein lineares Feld nicht exakt wieder (Fehler bis etwa 0,4 px bei Rasterpunkten); eine lokale Ebene durch die Punkte
  wäre genauer, wurde nicht umgesetzt, weil sie bei R1 nichts ändert.
- Helligkeit: Der Frameversatz-Stack ist 0,36 % heller als das Original (vermutlich rundet das Original auf 8 Bit); ohne Folgen
  für die Schwelle.
- Ob das Original beim Stacken neu registriert oder die Ergebnisse von *Align* weiterverwendet. Der Klon registriert im
  `Session::stack` neu und liest die Ergebnisse von *Align* dort nicht (nur *Save Aligned* tut es); der Kern kann beides.
- „Tests erst rot“: Der Vergleichstest war rot (43,92 dB gegen 45), die beiden Tests der Untersuchung gegen den Stand ohne
  Vertrauen und ohne `at_limit`-Filter ebenfalls. Die übrigen Tests in `stack.rs` wurden nicht gegen kaputte Varianten geprüft.
