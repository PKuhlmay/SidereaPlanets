# MAC-021 Ausrichten am Helligkeitsschwerpunkt

- Status: erledigt
- Abhängig von: MAC-006, RK-005

## Ziel

Die App sucht die Grobverschiebung jedes Frames heute per Bildvergleich auf verkleinerten Bildern, bis höchstens 64 px Drift. Springt
der Planet bei starkem Seeing oder schlechter Nachführung weiter, liegen die Alignpoints außerhalb ihres Suchbereichs. Die gesperrte
Option **Align by center of gravity** im Schritt Align wird gebaut (Philipp, 2026-11-12), dazu das Feld **Brightness threshold**: Die
Grobverschiebung kommt dann aus dem Helligkeitsschwerpunkt des Planeten, ohne Driftgrenze. Für Oberflächenaufnahmen (Mond, Sonne in
Ausschnitten) bleibt die Option aus.

## Getroffene Entscheidungen

- **Verfahren:** Der Schwerpunkt ersetzt die Grobsuche; die Alignpoints suchen wie bisher ±*Max. point shift* um die so gefundene
  Verschiebung. Keine zusätzliche Nachsuche per Bildvergleich.
- **Schwelle:** *Brightness threshold* auf der Skala 0–255 wie *Min./Max. brightness*, Startwert 20. Gewicht eines Pixels ist seine
  Helligkeit über der Schwelle, darunter zählt er nicht (Himmelshintergrund, Rauschen).
- **Bereich:** Gemessen wird im ganzen Frame, auch wenn *R.o.Interest* gesetzt ist; ein fester Bereich verlöre den Planeten genau bei den
  großen Sprüngen, für die die Option da ist.
- Gilt für Align und für das erneute Registrieren beim Stacken. *Estimate rotation* bleibt gesperrt.

## Entwurf (eigene Umsetzung, im Original nie beobachtet; in `docs/offene-fragen.md` Frage 21 als Vermutung)

- Kern: `AlignParams` bekommt `centre_of_gravity: Option<f32>` (Schwelle 0–255, `None` = aus). `Reference` merkt sich den Schwerpunkt
  des Bezugsframes; `align_frame` nimmt als Grobverschiebung die gerundete Differenz der Schwerpunkte. Liegt im Bezugsframe oder im
  Frame kein Pixel über der Schwelle, fällt die Suche auf den Bildvergleich zurück.
- Session und FFI: `AlignSettings` und `StackSettings` bekommen das optionale Feld (Standard aus, Tauri baut unverändert).

## Akzeptanzkriterien

- [x] Kern: Schwerpunkt mit Schwelle; synthetischer Test mit einer Drift über 64 px, der ohne die Option scheitert und mit ihr die
      Verschiebung trifft; Fallback, wenn nichts über der Schwelle liegt
- [x] Session/FFI: Align und Stack reichen die Option durch (optional, Standard aus; Tauri baut weiter)
- [x] Mac: Häkchen und Feld im Schritt Align (Advanced) entsperrt, Feld nur mit Häkchen aktiv, Tooltips; Werte in der Projektdatei
- [x] Hilfe (Schritt Align, Glossar), `docs/ui.md`, `docs/design.md`, `docs/offene-fragen.md`, `docs/parity/WINDOWS_PARITY.md`
- [x] Tests grün, neue Tests einmal rot an einer Assertion

## Ergebnis

- Kern: 70,5 px statt 110 px gefunden ohne die Umsetzung (Test rot), mit Schwerpunkt auf 0,3 px genau; der Fallback-Test war rot, solange
  ohne Pixel über der Schwelle (0, 0) zurückkam.
- Tests: `cargo test --workspace` grün, clippy ohne Warnungen, `pnpm test` 243 grün, `xcodebuild test` 421 grün.
- Von Hand zu prüfen: `docs/parity/Handprobe.md`, Abschnitt MAC-021.
