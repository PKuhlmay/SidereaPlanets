# RK-009 Speichern und Export

- Status: in Arbeit
- Abhängig von: RK-006

## Ziel
Ergebnisse in gängigen Formaten ausgeben.

## Akzeptanzkriterien
- [ ] Export als TIFF (16 Bit), PNG, JPEG, FITS
- [ ] Projekt-/Einstellungsdatei, damit eine Sitzung wieder geöffnet werden kann

## Stand
Backend und Oberfläche sind angebunden. Die Testläufe sind grün (`cargo test --workspace`, `cargo clippy`, `pnpm build`, `pnpm test`), neue
Tests waren vorher rot. Die Kriterien bleiben offen, bis das Speichern in der laufenden App von Hand geprüft ist (dieser Stand
wurde nur über Tests und Bau geprüft, die App selbst wurde nicht gestartet).

- Schreiber `crates/io/src/export.rs` (BMP, JPEG, TIFF 16, PNG 16, FITS 16/32/32 Float), `fits_write.rs`, `avi_write.rs`
  (unkomprimierte AVI); Projektdatei `project.rs`; Befehle `app/src-tauri/src/export_commands.rs`.
- Oberfläche: `app/src/saving.ts` (Handler `saveImage`, `saveAligned`, `saveIntegrated`, `openProject`, `saveProject`), Formatmenü
  `SaveImageMenu.vue`, Projektabschnitte `logic/projectfile.ts`. **Open** und **Save** (Projektdatei) sind eigene Schaltflächen
  der Werkzeugleiste, nicht mehr Einträge im Menü Tools (Entscheidung Philipp, 2026-11-11).
- *Save image* im Wavelet-Reiter schreibt das Bild mit den Farb- und Tonwerten (RK-008), wie das Fenster es zeigt; ohne Anpassung wie
  zuvor. Der Stack-Reiter schreibt das Summenbild unverändert.
- *Save Aligned* und *Save Integrated*: `app/src-tauri/src/sequence_commands.rs`, Kern `planets_core::sequence`. Save Aligned schreibt die
  nach *Limit* behaltenen Frames, auf das Bezugsframe geschoben, als AVI; Save Integrated den gleitenden Mittelwert über *Average of n
  Frames* (_Vermutung_, `docs/offene-fragen.md`, Punkt 23a). Fortschritt, Cancel und Pause wie bei *Stack*.
- 16-Bit-Wertebereich: entschieden, voller Bereich bis 65535 (`docs/offene-fragen.md`, Punkt 22).
- Format der Projektdatei und der Ausgaben: `docs/formats.md`; Anwenderdoku: Seite „Save and export“.
- Offen: Menü Flat/Dark/Reference (Darkframe, Flatfield, Referenzframe laden: gehört nicht zu Speichern und Export); Optionen von
  Create AVI ohne Wirkung (Codec, Compress Qual, Image size, Use Prefilters, Method Compressed); im Wavelet-Reiter schreibt *Save
  image* das Ergebnis von *Do All*, nicht den Stand der Vorschau (*Process*).

## Design
Soweit dieses Ticket Bedienelemente anbindet oder anzeigt, gilt `docs/design.md`. Aussehen und Anordnung werden nicht neu entschieden.

- Betroffen: **Save image** in Stack und Wavelet, **Save Aligned**, **Save Integrated**, **Load Scheme** und **Save Scheme** (`docs/design.md`, Abschnitte Schaltflächen, Stack, Wavelet)
- Soll-Bilder: `docs/design/screenshots/06-stack-stacked.png`, `07-wavelet.png`
- Dateidialoge sind die des Betriebssystems
