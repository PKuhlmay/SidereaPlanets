# Stacking (SP-006)

Was Siderea Planets beim Stacken tut und warum. Bedienelemente und Standardwerte stehen in `docs/ui.md` (Stack-Reiter), der Ablauf
in `docs/workflow.md`. Die Regressionsprüfung gegen gespeicherte eigene Referenzergebnisse steht am Ende des Verfahrens.
Parameter, die gewählt und nicht hergeleitet sind, stehen als Entwurfsentscheidung da.

## Stand

Rechenkern, Tauri-Befehle und Anbindung an den Stack-Reiter sind fertig. Die Tests in `stack.rs` (18) laufen grün; der
Regressionstest gegen die gespeicherte Referenz erreicht **54,85 dB** (Schwelle 45 dB), siehe „Regressionsprüfung“. Die beiden Tests
der Untersuchung (`scatter_of_the_points_does_not_distort_the_frame`, `matches_at_the_limit_of_the_search_range_are_not_followed`)
waren gegen den Stand davor rot.

## Ablauf und Standardwerte

Aus `docs/workflow.md` und `docs/ui.md`:

- Gestackt wird im Schritt **Stack**, nach Align und Limit. Ausgerichtet wird beim Stacken selbst, es gibt keinen
  eigenen Optimierungsschritt.
- Optionen im Panel *Stacking options*: *Use Nearestby Alignpoints* = 3, *Correct geometry* an, *Normalization of
  frame intensity* aus, *Do not stack n pixels at the frameedge* = 0.
- Das Ergebnis ist ein **Mittelwert**, keine Summe: Die Werte des Summenbilds liegen im Bereich der Einzelframes
  (0..255 bei 8 Bit, 0,0..1,0 im Bildtyp `Image`).
- Vor dem Mitteln wird jeder Frame lokal verschoben (Verfahren unten), nicht einfach ohne Ausrichtung gemittelt.

## Das Verfahren

Umgesetzt in `crates/core/src/stack.rs`.

1. **Registrierung.** Jeder Frame wird mit `Reference::align_frame` (SP-005) gegen das Bezugsframe registriert. Das liefert
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
   der Zuordnung bei 0, dann bewegt sich der Frame als Ganzes. Das ist eine Entwurfsentscheidung.
3. **Abtasten.** Der Frame wird an der verschobenen Stelle bilinear abgetastet (Entwurfsentscheidung; bilinear glättet etwas,
   bikubische Abtastung oder ganzzahlige Verschiebungen wären Alternativen).
4. **Mitteln.** Je Pixel werden die Werte aller Frames summiert, die den Pixel abdecken, und durch deren Anzahl geteilt.
   Am Rand, den nicht jeder Frame erreicht, gilt der Mittelwert der abdeckenden Frames. Ein Pixel ohne Abdeckung bleibt schwarz.
5. **Normalisierung** (Option): Jeder Frame wird mit `Mittel(Bezugsframe) / Mittel(Frame)` skaliert.
6. **Randabstand** (Option): Abtaststellen näher als *n* Pixel am Rand des Frames werden ausgelassen.

### Genauigkeit und Speicher

- Die Summe liegt in `f64`, die Zählung in `u32` je Pixel. Samples sind in `f32` auf `0..1` normiert; tausende 16-Bit-Frames
  laufen ohne Überlauf und ohne sichtbares Rundungsrauschen auf (Test mit 70 000 Frames voller Aussteuerung).
  Der Preis ist Speicher: 8 Byte je Messwert, bei 4096 × 4096 RGB etwa 400 MB. Bei Bedarf auf `f32` mit Blocksummen umstellen.
- Gestreamt wird in Stapeln von etwa einem Frame je Thread des Pools (Feld *CPUs*; bei sehr großen Frames weniger, höchstens
  1 GiB für zwei Stapel): Ein Thread liest die Datei der Reihe nach, und während ein Stapel bearbeitet wird, liest er den nächsten
  (`crates/core/src/batch.rs`). Im Stapel werden die Frames parallel registriert (`rayon`, über die Frames, nicht nur über die
  Alignpoints) und dann in der Reihenfolge der Auswahl addiert, parallel nach Bildzeilen. Das Summenbild ist deshalb bitgleich zu
  dem, das Frame für Frame entsteht, und hängt nicht von der Zahl der Threads ab (Tests in `stack.rs` und
  `crates/session/tests/parallel.rs`). Fortschritt zählt 1, 2, 3, ... und kommt je Stapel gesammelt; Abbruch und Pause greifen
  nach einem Stapel (SP-015; vorher nach jedem Frame, aber nur mit der Arbeit eines Frames auf wenigen Kernen). Gleiches gilt für
  Align; Save Aligned/Integrated arbeiten weiter Frame für Frame. Im Speicher liegen höchstens zwei Stapel und die Summen.
- Frames anderer Größe oder Kanalzahl werden ausgelassen und nicht mitgezählt (`StackResult::frames`).

### Schnittstelle für Folgetickets

- `planets_core::stack::stack_sequence(...) -> StackResult { image: Image, frames: u32 }`. Das Ergebnis ist der
  gewöhnliche Bildtyp `Image` (`f32`, `0.0..=1.0`, verschachtelte Kanäle), dieselbe Größe und Kanalzahl wie das Bezugsframe.
- `Stacker` (`new`, `add_aligned`, `finish`) nimmt Frames samt schon berechneter `FrameAlignment` entgegen. Damit lässt sich
  später das Ergebnis von *Align* wiederverwenden statt neu zu registrieren; `stack_sequence` registriert derzeit selbst.
- `stack::psnr(a, b)` ist die Messgröße der Regressionstests (Spitzenwert 1,0, bei 16 Bit also 65535).
- Frontend: `app/src/stack.ts` (Handler `stack` in `app/src/actions.ts`, über `runJob`), Anfrage aus dem Fensterzustand in
  `app/src/logic/stack.ts`. Das Bezugsframe steht in `session.referenceFrame`; *Set Alignpoints* (SP-005) muss es setzen,
  bis dahin gilt Frame 0.
- Sitzung: `planets_session::Session::stack` und `stacked_image` (`crates/session/src/stack.rs`) halten das letzte Summenbild; die
  Tauri-Befehle `stack`, `stacked_image`, `cancel_stacking`, `pause_stacking` sind dünne Hüllen darüber, Ereignis `stack-progress`.

### Warum das Vertrauen

Die Versätze der einzelnen Alignpoints streuen bei weichen Aufnahmen gegenüber dem Frameversatz: Bei der Testaufnahme R1 (Jupiter) um
median 3,4 bis 4,4 px (90 %-Wert 5,8 bis 6,7 px), 20 bis 30 von 78 Punkten liegen am Rand des Suchbereichs von ±5 px, und benachbarte
Punkte haben unabhängige Abweichungen. Die Korrelationswerte sind dabei hoch (im Mittel 0,88): Das Bild ist weich, die Korrelation hat
einen breiten Rücken, und das Maximum landet irgendwo darauf. Eine so große, unzusammenhängende Verzerrung ist bei einem Planeten
nicht echt. Folgte das Feld solchen Punkten, würde Bildinhalt aus hellen Bereichen weggeschoben und das Summenbild gleichmäßig dunkler.
Mit dem Vertrauen (Abschnitt Verfahren, Punkt 2) bewegt sich der Frame bei unzusammenhängender Streuung als Ganzes; bei einer echten,
zusammenhängenden Verzerrung folgt das Feld den Punkten (Test `local_correction_follows_a_coherent_distortion`).

Treffer am Rand des Suchbereichs werden ausgelassen, weil `PointShift::at_limit` ausdrücklich sagt, dass die Bewegung dort größer sein
kann. Bei R1 ändert das am Ergebnis fast nichts.

**Was das für weiche Aufnahmen heißt:** Die Standardeinstellung liefert dort praktisch den Frameversatz-Stack (das Vertrauen liegt
nahe 0). Die lokale Korrektur wirkt bei einer Aufnahme mit starker Luftunruhe. Die Ursache der Streuung liegt in der Zuordnung der Punkte
(SP-005: weiche Strukturen, breiter Korrelationsrücken), nicht im Stacken; eine robustere Zuordnung (Punkte nach Schärfe der
Korrelationsspitze wählen oder gewichten) würde die lokale Korrektur auch bei weichen Aufnahmen nutzbar machen.

### Regressionsprüfung

Test: `crates/io/tests/stack_reference.rs`. Er vergleicht das Stack-Ergebnis mit einem gespeicherten eigenen Referenzergebnis
(`re-local/reference/jupiter-stack.tif`) für Referenzfall R1 (Jupiter-Testaufnahme `re-local/testdata/astropix/JUPITER.AVI`).
`re-local/` ist nicht in Git; fehlt eine der beiden Dateien, wird der Test übersprungen (in einem Worktree fehlt `re-local/`, also im
Haupt-Checkout laufen lassen, oder `PLANETS_RE_LOCAL` auf ein anderes `re-local/` zeigen lassen).

1. Alignpoints auf Frame 1 mit Standardwerten setzen und Standard-Schwelle, registrieren, alle 10 Frames stacken (Limit behält bei R1 alle).
2. Das Referenz-TIFF laden (Werte ÷ 256 ÷ 255 auf die Skala 0..1; 16 Bit = 8 Bit × 256).
3. PSNR des Ergebnisses gegen das TIFF, Spitzenwert 1,0. Schwelle, die der Test prüft: **≥ 45 dB**.
4. Zur Einordnung werden mitgemessen und ausgegeben: PSNR bei *Correct geometry* aus (nur Frameversatz), PSNR von Frame 1 gegen das
   TIFF und die Kanalmittel beider Bilder.

Letzter Lauf (2026-11-11, `cargo test -p planets-io --test stack_reference --release -- --nocapture`): **54,85 dB** mit Standardwerten,
ebenso mit nur Frameversatz.

## Derotation (MAC-020)

Planeten drehen sich während der Aufnahme: Jupiter in 9 h 55 min 40,6 s (System II), Saturn in 10 h 33 min 38 s (System III), Mars in
24 h 37 min 22,7 s. *De-rotate images* ist ein Häkchen im Stack-Reiter; die Rechnung ist ein eigener Entwurf, geprüft gegen JPL Horizons und gegen
synthetische Szenen (unten). Code: `crates/core/src/planet.rs`
(Ephemeride), `derotate.rs` (Scheibe messen, Frame drehen), `stack.rs` (`stack_sequence_derotated`), `crates/session/src/derotate.rs`
(Zeiten, Plan, Angaben für das Panel), SER-Zeitstempel in `crates/io/src/ser.rs`.

**Zeit je Frame.** SER: der Nachspann der Datei hat je Frame einen 64-Bit-Wert in .NET-Ticks (100 ns seit 0001-01-01, UTC); er gilt nur,
wenn alle Werte zwischen 1990 und 2100 liegen (Kameras ohne Uhr schreiben Nullen), der Kopf (`DateTimeUTC`) liefert sonst die Startzeit.
Alles andere (AVI, Bildserien): Startzeit plus Frame-Nummer durch Bildrate. Die Startzeit gibt der Nutzer an; der Vorschlag ist die
Änderungszeit der Datei minus Länge der Aufnahme (die Datei wurde zuletzt am Ende geschrieben), er ist nur ein Vorschlag.
Ohne Bildrate und ohne Zeitstempel (SER ohne Nachspann) gibt es keine Zeiten: Fehler „De-rotation needs time stamps or a frame rate, and this recording has neither“.
Bezugszeit ist die Mitte zwischen dem ersten und dem letzten der ausgewählten Frames; das Bezugsframe, auf dem die Alignpoints stehen,
wird auf diese Zeit gedreht, bevor das `Reference` entsteht, sonst folgten die Alignpoints Strukturen, die gewandert sind.

**Achse.** Aus den mittleren Bahnelementen von Erde und Planet (Meeus, *Astronomical Algorithms*, 2. Aufl., Tabelle 31.A, auf das Äquinoktium
des Datums bezogen), der Lichtlaufzeit (Kap. 42, 43, 45) und dem IAU-Pol (Jupiter 268,0566° / 64,4953°, Saturn 40,589° / 83,537°, Mars
317,681° / 52,886°, jeweils mit der Änderung je Jahrhundert) folgen `D_E` (Breite der Erde im Planetensystem, bei Saturn Meeus' `B`) und der
Positionswinkel `P` des Nordpols. Störungen, Nutation und Aberration fehlen. Gegen **JPL Horizons** (geozentrisch, 0 h UT, 10 Daten von 1992
bis 2030 je Planet, Test `agrees_with_jpl_horizons_over_forty_years`): größte Abweichung **0,23° bei `D_E`** (Saturn 2026) und **0,32° bei
`P`**; Jupiter bleibt unter 0,02° bei `D_E`. Meeus' Beispiele (Test `meeus_worked_examples`, Toleranz 0,15° bis 0,3°; das Buch lag beim Schreiben nicht vor, die Zahlen sind deshalb zusätzlich gegen Horizons abgeglichen):
Jupiter 16.12.1992 0 h: `D_E` −2,48°, `P` 24,80°; Saturn 16.12.1992: `B` 16,42°, `P` 6,74°; Mars 9.11.1992: `D_E` 12,43°, `P` 347,64°. (Meeus' `D_E` ist die Breite des
Richtungsvektors zur Erde; Horizons' „ObsSub-LAT“ ist die planetographische Breite der Scheibenmitte und größer, `tan D_E = (1 − f)² tan φ`.)
Zeitbasis ist UTC statt TT (69 s, ohne Wirkung).

**Scheibe.** Im Bezugsframe: Otsu-Schwelle auf dem 3 × 3-geglätteten Grauwert, größter 4-zusammenhängender Fleck, Löcher gefüllt (dunkle
Bänder), Randpunkte jeweils auf halbem Weg zwischen Fleckpixel und Außenpixel (Punkte am Bildrand fallen weg, der Planet ist dort
abgeschnitten). An diese Punkte wird die Ellipse angepasst, die der Planet zeigen muss: Achsenverhältnis `sqrt(sin² D + (1 − f)² cos² D)`,
Richtung aus `P` und „Camera angle“ (zuerst Positionswinkel, dann gegebenenfalls Spiegelung, dann Kamerawinkel; Orientierung: bei Norden
oben ist Osten links). RANSAC (600 Dreierproben, deterministisch) findet die Ellipse, auf der die meisten Randpunkte liegen, wenige
Tukey-Schritte (Grenze 3 px) verfeinern sie: so ziehen ein Terminator (Mars) oder ein Mondrand nicht. **Saturn:** angepasst wird die
Vereinigung aus der Kugel (`1`, `k`) und der Außenkante des A-Rings (2,27 Äquatorradien, Verhältnis `|sin D|`); jeder Punkt gehört zum
Modell, dem er näher liegt, beide haben dieselbe Mitte und dasselbe `a`. Grenzen: der Ring muss sich von der Kugel abheben (die
Schwelle verläuft an der Außenkante des A-Rings, ± 1–2 % im Radius), eine Ringkante von fast 0° Neigung ist nur eine Linie (das Modell
wird dort auf `|sin D| ≥ 0,03` gehalten). Jeder weitere Frame: Mitte = Mitte der Bezugsscheibe plus Verschiebung seines Flecks
(Schwerpunkt) gegen den des Bezugsframes; das hebt alles auf, was die Schwelle der Form antut (Phase, Ring).

**Drehung.** Ein Pixel der Scheibe wird auf das abgeplattete Rotationsellipsoid zurückverfolgt (orthografische Projektion, Abplattung
0,06487 / 0,09796 / 0,00589), um Δλ = 360° · Δt / Periode um die Achse gedreht (Δt = Zeit des Frames minus Bezugszeit, die Oberfläche
läuft linksherum vom Nordpol aus gesehen, ein Merkmal wandert also vom Ost- zum Westrand: Test `the_wrong_direction_is_clearly_worse`),
zurückprojiziert und bilinear abgetastet; Pixel außerhalb der Scheibe bleiben, bei Δt = 0 wird der Frame unverändert (bitgleich)
zurückgegeben. Quelle hinter dem Limb (Tiefe ≤ 0) → Pixel bleibt. Saturn: wo der Ring vor der Kugel liegt (Strahl zum Beobachter
schneidet die Ringebene zwischen 1,239 und 2,27 Radien), bleibt der Pixel, am Ziel wie an der Quelle. Läuft im Batch vor `align_frame`,
parallel über die Frames (`rayon`), Frame für Frame gestreamt wie bisher; für Δt = 0 ist das Ergebnis bitgleich zum Stack ohne Derotation
(Test `without_time_differences_de_rotation_changes_nothing`). Nicht modelliert: Schatten des Rings und der Kugel, Wind
(Merkmale, die gegen die Oberfläche driften), Änderung von `D_E` und `P` während der Aufnahme (Hundertstelgrad in einer Stunde).
*Save Aligned* und *Save Integrated* derotieren nicht.

**Prüfung.** Synthetische, texturierte, abgeplattete Kugel (Bänder, Flecken, Sturm; Strahlverfolgung im Kamerasystem, unabhängig von den
Formeln des Moduls), PSNR nur innerhalb der Scheibe gegen das Bild zur Bezugszeit:

| Fall | mit Derotation | ohne |
|---|---|---|
| Jupiter (`D_E` −2,5°, `P` 24,8°), Frame 5 min später | 43,8 dB | 30,5 dB |
| Saturn mit Ring (`B` 19°), Frame 10 min später | 34,3 dB | 26,2 dB |
| Jupiter, Stack aus 3 Frames über 10 min, Bezug die Mitte (Kern) | 58,7 dB | 37,7 dB |
| dasselbe über die Sitzung mit einer SER-Datei samt Zeitstempeln | 52,1 dB | 35,6 dB |

Δt = 0 ist bitgleich; die falsche Drehrichtung ist um mehr als 10 dB schlechter; Kamerawinkel und Spiegelung sind mit einer um 40°
gedrehten und einer gespiegelten Szene geprüft. Alle neuen Tests waren zuerst rot (Stub oder umgekehrtes Vorzeichen, siehe Ticket).

**Laufzeit** mit `SaturnVideo.avi` (800 × 600, 3051 Frames, 30 fps, alle Frames, 12 Kerne, optimierter Bau): ohne Derotation 8,5 und 11,0 s,
mit Derotation 10,7 und 9,7 s (zwei Läufe je Fall; der Unterschied liegt im Rauschen der Plattenzugriffe). In dieser Aufnahme dreht sich
Saturn nur um 0,9°: das Panel sagt „Too short to matter“.

## Nicht umgesetzt

Nicht Teil der Akzeptanzkriterien; jeweils nur der Name ist vorgesehen:

- *StackSize* (Minimized, Maximized, R.of Interest), *Stretch histogram after stacking*. In der App
  sind sie gesperrt und mit „Not available yet“ gekennzeichnet (Default bei StackSize bleibt wählbar und ist das, was der
  Stack tut); die Einstellungen `stackSize` und `stretchHistogram` bleiben im Datenmodell, sind aber nicht Teil
  von `StackRequest` und das Backend kennt sie nicht. *De-rotate images* ist gebaut (Abschnitt „Derotation (MAC-020)“; im
  Stack-Reiter seit SP-023).
- **Resampling** (Bell, Bspline, Lanczos, Mitchell, Faktor) und **Drizzling**, die das Ergebnis vergrößern. In der App sind
  beide Häkchen gesperrt und mit „Not available yet“ gekennzeichnet; die Einstellungen `resampling`/`drizzling` bleiben im
  Datenmodell, das Backend ignoriert sie.
- **Create AVI** (*Save Aligned*, *Save Integrated*): in SP-009 gebaut, siehe `docs/formats.md`, Abschnitt „Create AVI“.
- Stackgraph und Stacklist.

## Offene Punkte

- Interpolation und Gewichtung der nächsten Punkte: bei weichen Aufnahmen ohne Wirkung (nur Frameversatz). Das gewichtete Mittel der nächsten
  Punkte gibt ein lineares Feld nicht exakt wieder (Fehler bis etwa 0,4 px bei Rasterpunkten); eine lokale Ebene durch die Punkte
  wäre genauer, wurde nicht umgesetzt, weil sie bei R1 nichts ändert.
- Der Stack registriert in `Session::stack` neu und liest die Ergebnisse von *Align* dort nicht (nur *Save Aligned* tut es); der Kern
  kann beides (`Stacker::add_aligned`).
- „Tests erst rot“: Der Regressionstest war rot (43,92 dB gegen 45), die beiden Tests der Untersuchung gegen den Stand ohne
  Vertrauen und ohne `at_limit`-Filter ebenfalls. Die übrigen Tests in `stack.rs` wurden nicht gegen kaputte Varianten geprüft.
