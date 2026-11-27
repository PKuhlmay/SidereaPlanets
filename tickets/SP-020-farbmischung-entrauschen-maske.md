# SP-020 Color Mixing, Denoise/Deringing und Masking (Windows zieht nach)

- Status: erledigt
- Abhängig von: SP-016, MAC-018, MAC-019
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Drei gesperrte Knöpfe im Panel Adjustments öffnen ihre Karten. Alle drei sind Werte in `ColorSettings`, die der Kern schon versteht;
Windows baut nur Karten, Payload und Projektdatei.

## Umfang

Werte, Reihenfolge im Kern, Projektdatei und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`:

- Abschnitt „Adjustments: Color Mixing und View Compare (Mac: MAC-019)“, nur der Punkt **Color Mixing** (View Compare kommt mit SP-021)
- Abschnitt „Denoise/Deringing und Masking (Mac: MAC-018)“, alle Punkte, auch **Show mask** (braucht einen Tauri-Befehl für
  `Session::refine_mask_overlay`)

Rechenkern liegt auf `main` (`planets_core::color::mix_channels`, `crates/core/src/refine.rs`, `ColorSettings.mix` und `.refine` mit
`#[serde(default)]`) und braucht kein Ticket, nur die Bestätigung durch die Tests.

## Akzeptanzkriterien

- [x] Karte Color Mixing (3 × 3 Matrix, Preset, Preserve brightness, Reset); `colourPayload` sendet `mix`, `isNeutral` kennt die Matrix (Test)
- [x] Karte Denoise/Deringing und Karte Masking mit den Werten und Grenzen aus dem Parity-Punkt; `refine` geht an alle Kernaufrufe, die
      `ColorSettings` nehmen, auch an die Histogramm-Anfrage (Test)
- [x] Show mask nur bei offener Karte Masking im Schritt Wavelets; Before, Remembered image und Difference haben Vorrang; nicht in Save Image
- [x] Projektdatei: `wavelet.tone.mix` und Abschnitt `refine` schreiben und lesen, fehlende Werte bleiben, Zahlen auf Grenzen gekürzt (Test)
- [x] Die Knöpfe Color Mixing, Denoise/Deringing und Masking sind nicht mehr gesperrt
- [x] Punkte aus `docs/parity/WINDOWS_PARITY.md` gelöscht (Color Mixing und den Abschnitt MAC-018)
- [x] Probe von Hand als `## SP-020 Color Mixing, Denoise/Deringing und Masking (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [x] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen

## Umsetzung

- Werte: `settings.wavelet.tone.mix` (Matrix und Preserve brightness, landet wie am Mac unter `wavelet.tone.mix` der Projektdatei) und
  `settings.refine` (eigener Abschnitt `refine` mit den Schlüsseln der Mac-App), Grenzen in `REFINE_LIMITS` und `TONE_LIMITS.mix`
  (`app/src/logic/settings.ts`). Beim Lesen nimmt `applyProject` die Matrix nur als neun endliche Zahlen und kürzt alle Werte auf ihre Grenzen.
- `colourPayload` schickt `mix` und `refine` (Form von `RefineSettings` des Kerns) an `apply_colors`, die Histogramm-Befehle und Save Image;
  `isNeutral` und `histogramKey` kennen beide (`app/src/logic/colour.ts`, Presets und `clampWeight` dort).
- Karten `ColorMixingCard.vue`, `DenoiseCard.vue`, `MaskCard.vue` (Zeilen über `RefineSlider.vue`); die drei Knöpfe sind frei, gesperrt
  bleiben acht.
- Show mask: neuer Tauri-Befehl `refine_mask_overlay` (`app/src-tauri/src/color_commands.rs`, ruft `Session::refine_mask_overlay`);
  `refreshMaskOverlay` in `colour.ts` holt das Bild nur, solange Show mask an, die Karte Masking offen und der Schritt Wavelets gezeigt
  ist. Im Bildbereich ersetzt es das aktuelle Bild; Before, Remembered image und Difference gehen vor. Nicht gespeichert.
- Der Tooltip von Show mask sagt „blue“ wie Kern und Hilfeseite; der Mac sagt „red“, dafür steht ein Punkt in `docs/parity/MAC_PARITY.md`.
