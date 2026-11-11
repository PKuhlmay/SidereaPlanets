# RK-009 Speichern und Export

- Status: in Arbeit
- Abhängig von: RK-006

## Ziel
Ergebnisse in gängigen Formaten ausgeben.

## Akzeptanzkriterien
- [ ] Export als TIFF (16 Bit), PNG, JPEG, FITS
- [ ] Projekt-/Einstellungsdatei, damit eine Sitzung wieder geöffnet werden kann

## Stand
Backend und Oberfläche sind angebunden und kompilieren (`cargo clippy`, `pnpm build`). Die Tests sind geschrieben, aber noch nicht
gelaufen. Deshalb bleiben die Kriterien offen, bis die Testläufe grün sind und das Speichern in der laufenden App geprüft ist.

- Schreiber `crates/io/src/export.rs` (BMP, JPEG, TIFF 16, PNG 16, FITS 16/32/32 Float), `fits_write.rs`, `avi_write.rs`
  (unkomprimierte AVI); Projektdatei `project.rs`; Befehle `app/src-tauri/src/export_commands.rs`.
- Oberfläche: `app/src/saving.ts` (Handler `saveImage`, Tools ▸ Open/Save project), Formatmenü
  `SaveImageMenu.vue`, Projektabschnitte `logic/projectfile.ts`.
- Format der Projektdatei und der Ausgaben: `docs/formats.md`; Anwenderdoku: Seite „Save and export“.
- Offen: *Save Aligned* und *Save Integrated* (brauchen die ausgerichteten Frames aus RK-005/RK-006; der Schreiber ist fertig, die
  Schaltflächen melden „not available yet“), Menü Flat/Dark/Reference (Darkframe, Flatfield, Referenzframe laden: gehört nicht zu
  Speichern und Export), Aufruf von `ExportState::set_stack` und `set_wavelet` durch Stack und Wavelet, bis dahin meldet
  *Save image* „there is no stacked image yet“. 16-Bit-Wertebereich: `docs/offene-fragen.md`, Punkt 22.

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: **Save image** in Stack und Wavelet, **Save Aligned**, **Save Integrated**, **Load Scheme** und **Save Scheme** (`docs/design.md`, Abschnitte Schaltflächen, Stack, Wavelet)
- Soll-Bilder: `docs/design/screenshots/06-stack-stacked.png`, `07-wavelet.png`
- Dateidialoge sind die des Betriebssystems
