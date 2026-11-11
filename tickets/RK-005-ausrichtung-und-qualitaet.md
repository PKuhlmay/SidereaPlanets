# RK-005 Ausrichten und Qualitätsbewertung der Frames

- Status: in Arbeit (Rechenkern, Tauri-Befehle und Doku stehen; Qualitätsmaß weicht vom Original ab, Bedienung im Align-Reiter angebunden, aber nie gestartet)
- Abhängig von: RK-004

## Ziel
Wie im Original: Referenzbereich wählen, alle Frames darauf ausrichten, Qualität je Frame schätzen,
Frames nach Qualität sortieren und begrenzen.

## Akzeptanzkriterien
- [x] Ausrichtung per Korrelation, subpixelgenau, an mehreren Ausrichtungspunkten (Alignpoints); davor eine grobe Suche für Drift bis 64 px
- [x] Ausrichtung auf einem frei wählbaren Bereich (*R.o.Interest*): Alignpoints und Driftsuche nur im Rechteck (`Roi`, `PointParams::roi`, `Reference::with_roi`). Am Original unbeobachtet, eigene Umsetzung nach den Namen (`docs/offene-fragen.md`, Frage 21)
- [ ] *Scan Frames* (mit *Keep Alignpoints inside ScanFrame*), *Estimate Rotation*, *Align by Centre of gravity*: nicht umgesetzt, die Bedeutung ist weder beobachtet noch aus den Namen ableitbar (Frage 21)
- [x] Qualitätsmaß je Frame (Schärfe), sortierte Liste, Auswahl über Prozentsatz/Anzahl (*Lowest Quality %*, *Best Frames %*, *Best Frames*)
- [ ] Qualitätsmaß entspricht dem Original: bei Referenzfall R1 sind die drei besten Frames dieselben (Original 1, 4, 3; Klon jetzt 1, 3, 10, vorher 1, 10, 8). Näher (Rangkorrelation 0,79 statt 0,54), aber nicht erreicht: 240 geprüfte Maße ergaben keine gleiche Reihenfolge, welches Maß das Original nutzt, ist offen (`docs/offene-fragen.md`, Frage 17, `docs/testdaten.md`), ein längerer Referenzfall in RK-011 würde das klären
- [x] Limit-Modus *Frames/Apoint*: `LimitMode::FramesPerPoint`, jeder Punkt behält seine besten Frames (`best_frames_per_point`), Limit liefert die Vereinigung. Am Original unbeobachtet, Bedeutung nach dem Namen (Frage 21); das Stacking mit punktweisen Frame-Mengen gehört zu RK-006
- [x] Verhalten mit dem Original an Testdaten verglichen (Alignpoint-Zahl, Reihenfolge, Limit); Ergebnis und Abweichungen in `docs/testdaten.md`, Test `crates/io/tests/align_reference.rs`. Versatzwerte des Originals sind nicht auslesbar, nur gegen künstliche Verschiebungen geprüft
- [x] Alignpoint-Zahl abgeleitet statt angepasst: Abstand = *Minimum distance between* + *Max Alignpoint movement* (quadratisch), Threshold-Standard ein Zehntel der Punkte. Bei R1 87 gefunden (Original 91), 78 nach Threshold (Original 82). Beides sind Vermutungen aus je einer Beobachtung (`docs/offene-fragen.md`, Frage 18, `docs/testdaten.md`)
- [x] Parallel über Frames (`rayon`), Frames werden nacheinander gelesen und in Blöcken parallel ausgerichtet
- [ ] Bedienung im Align-Reiter angebunden (Alignpoints setzen, Align mit Fortschritt, Cancel und Pause, Limit, Framelist, Registrationgraph): geschrieben (`app/src/align.ts`, `app/src/logic/align.ts`, Threshold-Regler, R.o.Interest-Rechteck, Frames/Apoint im Limit Setup), kompiliert, aber nie in der App gestartet und nicht mit den Soll-Bildern 01–04 und 11 verglichen. *No Align*, *Scan Frames* und *Empty List* sind bewusst ohne Funktion ("not available yet"), weil am Original unbeobachtet; Punkte von Hand setzen gibt es nicht
- [x] Erneutes Set Alignpoints und Align nach Limit, Stack und Wavelet (Entscheidung von Philipp, 2026-11-11; weicht vom Original ab, das beide ab Align sperrt): `stepButtons` lässt beide ab gesetzten Punkten zu, ein neuer Lauf verwirft Limit-Auswahl, Stack, Wavelet-Ergebnis und Farbanpassung in `session` und im Backend (`discard_results` in `app/src-tauri/src/lib.rs` leert `StackState`, `WaveletState`, `ColorState`, `ExportState`; `AlignState` leert seine Ergebnisse), Align schon beim Start, damit ein Abbruch kein altes Ergebnis stehen lässt. Tests: `app/src/logic/workflow.test.ts`, `app/src/flow.test.ts`, je ein `clearing_…`-Test in den Rust-Modulen. Doku: `docs/design.md`, `docs/workflow.md`, Seiten *Align your frames*, *The Align tab*, *The window at a glance*, *Frame quality and Limit*
- [x] Anwenderdoku zum Align-Reiter in der App (Regel „Doku in der App“): Seiten *Align your frames* und *Frame quality and Limit* in `app/src/docs/pages/`. Dazu die Übersichtsseite *The Align tab* aus RK-010, aktualisiert; *No Align*, *Scan Frames* und *Empty List* stehen dort als „Not available yet“

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: Align-Reiter komplett, Karten Framelist und Registrationgraph, Fortschritt mit Cancel und Pause (`docs/design.md`, Abschnitte Align, Zustände, Schwebende Karten)
- Soll-Bilder: `docs/design/screenshots/01-align-loaded.png` bis `04-align-aligned.png`, hell `11-align-light.png`

## Stand
Erneutes Align (2026-11-11): Set Alignpoints und Align sind nach Limit, Stack und Wavelet wieder bedienbar und verwerfen die davon abhängigen Ergebnisse (Oberfläche und Backend). Die Tests dazu liefen rot (an Assertionen) und dann grün; Entwurf `oberflaeche-entwurf.html` und die Soll-Bilder 04 und 11 sind angepasst.

Rechenkern (`crates/core/src/align.rs`), Tauri-Befehle (`app/src-tauri/src/align_commands.rs`; Bereich, Frames/Apoint, `limit`
liefert zusätzlich den mittleren Versatz je Punkt), Vergleichstest (`crates/io/tests/align_reference.rs`), Anbindung im
Align-Reiter und Doku sind geschrieben und kompilieren (`cargo clippy --workspace --all-targets`, `pnpm build`).
**Keine Tests gelaufen** (Testpause seit 2026-11-10): die Unit-Tests (Rechenkern, `app/src/logic/align.test.ts`,
`app/src/docs/align-pages.test.ts`) sind ungeprüft und noch nicht rot gesehen. Die App wurde nicht gestartet; wie der
Align-Reiter aussieht und sich verhält, ist ungeprüft. Die Zahlen in `docs/testdaten.md` (87/78 Punkte, Rangkorrelation
0,79, Bereich um den Planeten) stammen aus einem einmaligen Wegwerf-Programm gegen `JUPITER.AVI` in Phase 1, nicht aus
Tests; seitdem laufen keine Messläufe mehr.
