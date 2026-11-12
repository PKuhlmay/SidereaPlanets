# RK-007 Wavelet-Schärfung

- Status: in Arbeit
- Abhängig von: RK-006

## Ziel
Das Herzstück von the reference program: mehrschichtige Wavelet-Zerlegung mit Reglern je Ebene für Schärfen und Entrauschen.

## Akzeptanzkriterien
- [x] Zerlegung in Ebenen mit Verstärkung und Rauschunterdrückung je Ebene (Anzahl und Details laut `docs/`)
- [ ] Vorschau reagiert flüssig auf Reglerbewegung (Maßnahmen und Messwerte: `docs/gui-toolkit.md`, Abschnitt Spike: Vorschau auf Ausschnitt begrenzen, RGBA-Umwandlung mit `rayon`, Übertragung neu messen). Rechenkern gemessen (Reglerschritt 4,8 ms bei 512 px, 20 Bilder/s verlangen 50 ms); im Release-Fenster gemessen (Statusleiste „Preview: n ms“, Anfrage bis Zeichnen): 38, 49 und 79 ms je Reglerschritt, also um das Ziel von 50 ms; ein flüssiges Ziehen des Reglers von Hand wurde nicht beurteilt, deshalb bleibt das Kriterium offen
- [x] Ergebnis bei gleichen Reglerwerten gegen das Original verglichen (PSNR-Schwelle in `docs/` festlegen): Standardwerte 98,81 dB, Ebene 1 = 10.0 84,73 dB, Schwellen 45/65/85 dB (`docs/wavelets.md`)
- [ ] Einstellungen speicher- und ladbar. `.rwv` lesen und schreiben ist getestet (byteidentisch zur Datei des Originals), Befehle und Dateidialog sind im laufenden Programm **nicht** ausprobiert

## Stand (Abbrechen und Pausieren, 2026-11-11)
Do All ist im Rechenkern abbrechbar und pausierbar; ein abgebrochenes Do All ändert weder `processed` noch `ExportState`. Rot-zuerst
belegt (Tests an Assertionen gescheitert, dann grün).
- Kern: `wavelet::do_all_until(image, settings, mode, checkpoint)` ruft `checkpoint(fertige_Ebenen, Ebenen)` vor der ersten und nach jeder
  Ebene auf, `None` bei Abbruch. `process` und `do_all` bleiben unverändert, die Vorschau (`Decomposition`, `Previewer`) benutzt den Pfad nicht,
  ist also nicht langsamer.
- Befehl: `WaveletState::do_all` hält Abbruch- und Pause-Flag (`cancel_wavelet`, `pause_wavelet`), setzt Ergebnis und Export erst nach dem
  letzten Layer und meldet Fortschritt über das Ereignis `wavelet-progress`. Abbruch- und Pause-Granularität ist eine Ebene, bei großen Bildern
  ein Moment.
- Oberfläche: Auftrag „Wavelet“ in `app/src/jobs.ts` schickt Cancel und Pause an die neuen Befehle; `doAll` zeigt den Fortschritt. Process
  rechnet nur die Processing Area (Bruchteil einer Sekunde): Das Backend bricht dort nicht mitten drin ab, Cancel verwirft das Ergebnis beim
  Eintreffen (bewusst so, sonst müsste der Vorschau-Pfad Prüfungen bekommen).
- Tests: `crates/core/src/wavelet/mod.rs` (`do_all_until_*`), `app/src-tauri/src/wavelet_commands.rs`, `app/src/flow.test.ts`.
- Doku: *Progress, Cancel and Pause*, *The Wavelet tab*, *Wavelet sharpening*.

## Stand (Load Scheme meldet sich, 2026-11-12)
Fund aus dem Durchlauf der Release-App: Save Scheme meldete „Scheme saved: …“, Load Scheme nichts. `loadScheme` setzte die Meldung, aber die geladenen
Werte starten im Schritt Wavelets eine neue Vorschau, und deren Meldung („Calculating wavelets done“) überschrieb sie. `loadScheme` wartet jetzt
(`nextTick`, dann den laufenden Vorschau-Lauf `running`) und setzt die Meldung danach. Test: `app/src/schemes.test.ts`, rot an einer Assertion gesehen,
dann grün.

## Stand (Tonkennlinie und Farbe, 2026-11-11)
Die Tests laufen wieder (`cargo test --workspace`, `pnpm test`, clippy ohne Warnungen). Neu in diesem Schritt:
- **Tonkennlinie des Originals** (`0,988231·x + 0,015595·x³`, `wavelet/output.rs`) am Ende von Do All und der Vorschau; gegen die Referenz
  98,81 dB (Standardwerte) und 84,73 dB (Ebene 1 = 10.0), vorher 55,6 dB. Belegt ist die Form bis 0,61 des Wertebereichs, darüber geraten
  (`docs/offene-fragen.md`, Frage 22).
- **Colour und LRGB** wirken auf Vorschau und Do All (`ColourMode`, `wavelet_preview(..., mode)`, `wavelet_do_all(..., mode)`).
  Geschlossen aus den Namen, nicht beobachtet (Frage 26). Die Häkchen sitzen weiter nur im Align- und im Stack-Reiter.
- Offen, nicht in diesem Schritt: Vorschau-Übertragung messen, Dateidialog für Schemata ausprobieren, Ebenen 2 bis 6 gegen das Original.

## Stand (Phase 1: Rechenkern, Befehle, Doku)
Alles unten ist umgesetzt. Der Vergleich mit der Referenz lief am 2026-11-11 (siehe oben); die Aussagen unten zu „nicht gelaufen“ sind überholt,
soweit sie Rust-Tests und PSNR betreffen.

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
