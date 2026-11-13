# RK-020 Color Mixing, Denoise/Deringing und Masking (Windows zieht nach)

- Status: offen
- Abhängig von: RK-016, MAC-018, MAC-019
- Stufe: `windows-v0.1.0` (Nachzug zu Mac `v0.1.0`)

## Ziel

Drei gesperrte Knöpfe im Panel Adjustments öffnen ihre Karten. Alle drei sind Werte in `ColorSettings`, die der Kern schon versteht;
Windows baut nur Karten, Payload und Projektdatei.

## Umfang

Werte, Reihenfolge im Kern, Projektdatei und Mac-Dateien in `docs/parity/WINDOWS_PARITY.md`:

- Abschnitt „Adjustments: Color Mixing und View Compare (Mac: MAC-019)“, nur der Punkt **Color Mixing** (View Compare kommt mit RK-021)
- Abschnitt „Denoise/Deringing und Masking (Mac: MAC-018)“, alle Punkte, auch **Show mask** (braucht einen Tauri-Befehl für
  `Session::refine_mask_overlay`)

Rechenkern liegt auf `main` (`planets_core::color::mix_channels`, `crates/core/src/refine.rs`, `ColorSettings.mix` und `.refine` mit
`#[serde(default)]`) und braucht kein Ticket, nur die Bestätigung durch die Tests.

## Akzeptanzkriterien

- [ ] Karte Color Mixing (3 × 3 Matrix, Preset, Preserve brightness, Reset); `colourPayload` sendet `mix`, `isNeutral` kennt die Matrix (Test)
- [ ] Karte Denoise/Deringing und Karte Masking mit den Werten und Grenzen aus dem Parity-Punkt; `refine` geht an alle Kernaufrufe, die
      `ColorSettings` nehmen, auch an die Histogramm-Anfrage (Test)
- [ ] Show mask nur bei offener Karte Masking im Schritt Wavelets; Before, Remembered image und Difference haben Vorrang; nicht in Save Image
- [ ] Projektdatei: `wavelet.tone.mix` und Abschnitt `refine` schreiben und lesen, fehlende Werte bleiben, Zahlen auf Grenzen gekürzt (Test)
- [ ] Die Knöpfe Color Mixing, Denoise/Deringing und Masking sind nicht mehr gesperrt
- [ ] Punkte aus `docs/parity/WINDOWS_PARITY.md` gelöscht (Color Mixing und den Abschnitt MAC-018)
- [ ] Probe von Hand als `## RK-020 Color Mixing, Denoise/Deringing und Masking (windows-v0.1.0)` in `docs/parity/Handprobe.md` eingetragen
- [ ] `cargo test --workspace`, `clippy` ohne Warnungen, `cd app && pnpm test` und `pnpm build` grün, neue Tests einmal rot gesehen
