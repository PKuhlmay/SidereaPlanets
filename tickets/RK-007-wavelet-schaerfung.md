# RK-007 Wavelet-Schärfung

- Status: in Arbeit
- Abhängig von: RK-006

## Ziel
Das Herzstück von the reference program: mehrschichtige Wavelet-Zerlegung mit Reglern je Ebene für Schärfen und Entrauschen.

## Akzeptanzkriterien
- [ ] Zerlegung in Ebenen mit Verstärkung und Rauschunterdrückung je Ebene (Anzahl und Details laut `docs/`)
- [ ] Vorschau reagiert flüssig auf Reglerbewegung (Maßnahmen und Messwerte: `docs/gui-toolkit.md`, Abschnitt Spike: Vorschau auf Ausschnitt begrenzen, RGBA-Umwandlung mit `rayon`, Übertragung neu messen)
- [ ] Ergebnis bei gleichen Reglerwerten gegen das Original verglichen (PSNR-Schwelle in `docs/` festlegen)
- [ ] Einstellungen speicher- und ladbar

## Stand (Phase 1: Rechenkern, Befehle, Doku)
Alles unten ist umgesetzt und mit `cargo clippy` übersetzt, die Tests sind geschrieben, aber wegen der Testpause
(seit 2026-11-10) **noch nicht gelaufen**. Deshalb ist noch kein Kriterium abgehakt.

- Zerlegung: `crates/core/src/wavelet/` (sechs Ebenen, Gain, Denoise, Sharpen, Linear/Dyadic, beide Filter), Modell und Belege in
  `docs/wavelets.md`. Belegt am Original ist nur Ebene 1 (Python-Modell 85,2 dB), der Rest ist geraten (`docs/offene-fragen.md`, 22–28).
- Vorschau: Ausschnitt mit Rand, gespeicherte Ebenen, parallele RGBA-Umwandlung. Messlauf
  `crates/core/tests/wavelet_preview_timing.rs` ist geschrieben, Messwerte fehlen; die Übertragung wird mit der Oberfläche gemessen (Phase 2).
- Vergleich: Schwellen festgelegt (A ≥ 45 dB, B ≥ 65 dB nach Kennlinie), Test `crates/io/tests/wavelet_reference.rs`, noch nicht gelaufen.
- Schemata: `.rwv` des Originals lesen und schreiben, Befehle `save_wavelet_scheme`/`load_wavelet_scheme`; Dateidialog und Liste
  *Available schemes* gehören zu RK-009.
- Anwenderdoku: `app/src/docs/pages/wavelet-sharpening.md` (Bedienung ist als „Not available yet“ markiert).

## Stand (Phase 2: Oberfläche)
Angebunden (`app/src/wavelet.ts`, `app/src/logic/wavelet.ts`, `WaveletPanel.vue`, Handler in `actions.ts`); `pnpm build` und clippy laufen,
Tests (`logic/wavelet.test.ts`) sind geschrieben, nicht gelaufen. Das Programm wurde nicht gestartet, es gibt keine Messwerte.
- Live-Vorschau der Processing Area bei jeder Änderung, Process, Do All (`runJob`, Ergebnis in `session.processedImage`), Before (aus RK-010).
- Load/Save Scheme über einen Dateidialog (`pickSchemePath`, RK-009 ersetzt ihn durch `pick_scheme_path`).
- Automatic, Hold, Linked: geraten (`docs/offene-fragen.md`, Frage 29).
- Nicht angebunden: Realign with Processed (braucht RK-005/RK-006), Preview-Schaltfläche je Ebene, Liste *Available schemes* (RK-009).
- Das Summenbild liefert RK-006: nach Stack `WaveletState::set_source` aufrufen und `session.stackImage` setzen.

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: linkes Panel im Wavelet-Reiter mit den sechs Ebenen, **Process**, **Do All**, Umschalter **Before**, Processing Area (`docs/design.md`, Abschnitte Wavelet, Regler, Bildbereich)
- Soll-Bilder: `docs/design/screenshots/07-wavelet.png`, `08-wavelet-sharpened.png`, hell `12-wavelet-light.png`
