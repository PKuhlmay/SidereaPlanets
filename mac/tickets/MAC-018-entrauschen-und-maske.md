# MAC-018 Entrauschen, Entringen und Maske

- Status: erledigt
- Abhängig von: MAC-008, MAC-009, MAC-014

## Ziel

Die zwei gesperrten Adjustments-Schaltflächen **Denoise/Deringing** und **Masking** im Schritt Wavelets bekommen je eine Karte
(Philipp, 2026-11-12). Beide laufen im Rechenkern (`crates/core/src/refine.rs`, Entwurf in `docs/wavelets.md`, Abschnitt „Denoise/Deringing und
Masking“), zeigen eine Live-Vorschau wie die übrigen Karten und wirken auf Do All und Save Image.

1. **Denoise/Deringing:** Rauschen und der dunkle Ring am Planetenrand nach dem Schärfen vermindern. Denoise ist ein kantenerhaltender
   bilateraler Filter (5 × 5) mit Stärke in Prozent, Deringing hebt Pixel an, die unter das lokale Minimum des ungeschärften Summenbilds
   gefallen sind, nur in der Nähe starker Kanten, mit Stärke in Prozent und Radius in Pixeln.
2. **Masking:** Das Schärfen gilt nur dort, wo das ungeschärfte Summenbild heller als eine Schwelle ist (Planetenscheibe), mit weicher
   Kante (Feather in Pixeln), optional umgekehrt, mit Vorschau der Maske (Show mask). Das Ergebnis wird mit dem Summenbild nach der
   Maske gemischt.

## Akzeptanzkriterien

- [x] Nur diese zwei Schaltflächen sind freigeschaltet, die übrigen bleiben gesperrt; Texte nach den Regeln von MAC-014 (US-Englisch,
      Title Case für Knöpfe, Sentence case für Felder und Tooltips, Einheiten an den Feldern, kein „the reference program“)
- [x] Rechenkern: Stärke 0 und Maske aus geben das Bild bitgleich zurück; Deringing macht kein Pixel dunkler und hebt nie über das lokale
      Minimum des Originals hinaus an, ändert nichts an schwachen Kanten; Maske 0 und 1 an den Extremen exakt, Feather steigt stetig und die
      Kante wird mit dem Radius breiter
- [x] Live-Vorschau in der Karte (Wert ändern, Bild folgt), Show mask zeigt die Maske blau über dem Bild
- [x] Wirkt auf Anzeige, Histogramm, Save Image (Wavelet-Bild gegen das Summenbild) und damit nach Do All; Save Image schreibt ohne Karten, solange alles neutral ist
- [x] Werte im Abschnitt `refine` der Projektdatei (optional, `wavelet` und `colour` bleiben im Windows-Layout, unbekannte Abschnitte ignoriert die
      Windows-App); Reset in der Karte und bei Restore
- [x] Tauri-Schale baut unverändert (`ColorSettings.refine` mit `#[serde(default)]`)
- [x] Tests für Kern, Sitzung und Swift-Modell, rot zuerst (siehe unten)
- [x] Anwenderseite (`color-and-tones.md`, `step-wavelets-save.md`), Glossary, `docs/design.md`, `docs/wavelets.md`, Punkt in `docs/parity/WINDOWS_PARITY.md`

## Umsetzung

- **Kern** `crates/core/src/refine.rs`: `RefineSettings`/`MaskSettings` (serde, Standard aus), `refine()` in der Reihenfolge Deringing, Denoise, Maske;
  `deringing`, `denoise`, `mask`, `blend_by_mask`, `mask_overlay`. Ein Graustufen-Summenbild als Bezug eines RGB-Bildes wird auf drei Kanäle gebracht
  (das Fenster hält jedes Bild als RGB; dieser Fall fiel in den Tests der Sitzung auf).
- **Sitzung:** `ColorSettings.refine`; `Session::refined` (Anzeige, Histogramm, Auto-Punkte), `refine_mask_overlay`, und in `save()` für das Wavelet-Bild
  gegen das Summenbild. Der Bezug ist `ExportState::stack`, also braucht die Mac-App keinen zusätzlichen Aufruf.
- **FFI/Swift:** Records `RefineSettings`, `MaskSettings`, `refine_default_settings`, `refineMaskOverlay`; `RefineValues` (`RefineLogic.swift`), Karten in
  `RefineCards.swift`, Show mask über `AppModel.requestMaskOverlay`.
- **Reihenfolge der Schritte** (Anzeige und Speichern gleich): Deringing, Denoise, Maske, dann RGB Align, RGB Balance, (Color Mixing,) Histogram, Gamma,
  Contrast/Brightness. Die Maske wirkt zuletzt der neuen Schritte, damit auch Entrauschen und Entringen auf die Scheibe beschränkt bleiben.

## Prüfung

Siehe Bericht des Commits; Rot-zuerst-Läufe: Kern (Stub, der das Bild unverändert zurückgibt: 12 von 25 Tests an Assertions rot, die übrigen durch
Mutationen: Stärke immer +0,1, Schutz gegen Stärke 0 entfernt, Kantenschwelle 0, Maske bei „aus“ angewendet), Sitzung (Verdrahtung abgeschaltet, `serde(default)` entfernt, Maske
gegen das falsche Bild), Swift (15 Assertionen in 11 Tests durch neun Mutationen rot, `resetTone` einzeln).
