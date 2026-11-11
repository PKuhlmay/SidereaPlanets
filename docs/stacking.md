# Stacking (RK-006)

Was der Klon beim Stacken tut, warum, und wie weit das gegen the reference program geprüft ist. Bedienelemente und Standardwerte
stehen in `docs/ui.md` (Stack-Reiter), der Ablauf in `docs/workflow.md`, Referenzfall R1 in `docs/testdaten.md`.
Alles, was nur geraten ist, steht als **Vermutung** da.

## Stand

Rechenkern und Tauri-Befehle sind geschrieben und kompilieren (`cargo clippy --workspace --all-targets` ohne Warnungen).
**Es wurde kein Test und kein Vergleich ausgeführt**, auf ausdrücklichen Wunsch von Philipp (Testläufe sind vorerst aus).
Alle Messwerte unten sind deshalb leer. Die Tests sind geschrieben und müssen nach dem Ende der Pause einmal laufen, und
jeder muss einmal an einer Assertion rot gewesen sein (Regel „Tests erst rot“, siehe Abschnitt „Offen“).

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
   `p + (dx, dy)`. Punkte, die nicht verfolgt werden konnten, fehlen und zählen nicht.
2. **Verschiebungsfeld** (bei *Correct geometry* an). Für jeden Pixel werden die *n* nächsten Alignpoints
   (*Use Nearestby Alignpoints*) gemittelt, mit Gewichten, die zum (n+1)-ten Punkt hin auf null fallen, damit das Feld
   nicht springt, wenn sich die Menge der nächsten Punkte ändert. Ausgewertet wird das Feld auf einem Raster von 4 px und
   dazwischen bilinear interpoliert, das spart bei tausenden Frames die Suche nach den nächsten Punkten pro Pixel.
   Bei *Correct geometry* aus gilt für den ganzen Frame der Medianversatz.
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
- Gestreamt wird in Stapeln von `2 × Threads` Frames: lesen nacheinander (die Datei wird nur von einem Thread gelesen),
  registrieren parallel (`rayon`), addieren parallel nach Bildzeilen. Im Speicher liegen nur der Stapel und die Summen.
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
- Tauri: `stack_commands::StackState::image()` liefert eine Kopie des letzten Summenbilds; Befehle `stack`, `stacked_image`,
  `cancel_stacking`, `pause_stacking`, Ereignis `stack-progress`.

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

| Messung | Wert |
|---|---|
| Klon (Standardwerte, lokale Korrektur) gegen `jupiter-stack.tif` | **offen**, nicht gemessen |
| Klon mit *Correct geometry* aus gegen `jupiter-stack.tif` | **offen** |
| Frame 1 gegen `jupiter-stack.tif` | **offen** (zum Vergleich: Original-BMP gegen Frame 1 = 43,99 dB) |
| Kanalmittel Klon gegen Original (R 6674,1 · G 5130,5 · B 6099,5) | **offen** |

Einordnung der Erwartung, nicht gemessen: Die Alignpoint-Zahl (80 statt 82) und die Registrierung (RK-005) sind nicht
identisch mit dem Original, und der Klon registriert am Frame 1; ob das Original ein Mittelbild baut, ist unbekannt
(`docs/offene-fragen.md`, Punkt 19). All das geht in den Versatz jedes Pixels ein. Ob 45 dB erreicht werden, ist offen.

### Wenn die Schwelle verfehlt wird

In dieser Reihenfolge prüfen, jeweils mit dem Test messen:

1. Abtastung: bikubisch statt bilinear.
2. Gewichtung der nächsten Punkte (Abstand hoch 1 statt 2, harte Nachbarschaft ohne Auslaufen).
3. Registrierung von RK-005 (Bezugsframe, Subpixel-Verfeinerung), weil dort die größte Unsicherheit liegt.
4. Ob das Original Frames vor dem Mitteln gewichtet (nach Qualität).

## Nicht umgesetzt

Nicht Teil der Akzeptanzkriterien und nie beobachtet; jeweils nur der Name bekannt:

- *StackSize* (Minimized, Maximized, R.of Interest), *Stretch histogram after stacking*, *De-rotate images*.
- **Resampling** (Bell, Bspline, Lanczos, Mitchell, Faktor) und **Drizzling**, die das Ergebnis vergrößern.
- **Create AVI** (*Save Aligned*, *Save Integrated*); gehört zu RK-009.
- Stackgraph und Stacklist.

## Offen

- Interpolation und Gewichtung wie oben, bis der Vergleich vorliegt.
- Ob das Original beim Stacken neu registriert oder die Ergebnisse von *Align* weiterverwendet. Der Klon registriert im
  Tauri-Befehl neu, weil der Zustand der Align-Befehle nicht von außen lesbar ist; der Kern kann beides.
- „Tests erst rot“: Die Tests in `stack.rs` und `stack_reference.rs` wurden nicht ausgeführt, also auch nicht rot gesehen.
  Nach der Testpause: jeden Test gegen eine absichtlich kaputte Variante laufen lassen (z. B. Vorzeichen des Versatzes
  in `Stacker::add_aligned` vertauschen, `edge_skip` ignorieren) und prüfen, dass er an einer Assertion scheitert.
