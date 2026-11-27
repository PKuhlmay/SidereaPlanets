# MAC-022 Statusleiste wie Windows

- Status: offen
- Abhängig von: MAC-008, MAC-009, MAC-015, SP-025
- Stufe: Nachzug auf `v0.1.0` (Windows: `windows-v0.1.0`)

## Ziel

Die Statusleiste des Macs zeigt, was die Windows-App in `windows-v0.1.0` schon zeigt: in den Schritten Wavelets und Save Position und
Wert unter dem Mauszeiger (heute bleibt das Feld leer, `status.position` in `mac/App/Model/AppModel.swift`), und rechts außen wie in
Siderea Version, Commit und Bauzeit. Danach hat `docs/parity/MAC_PARITY.md` für `windows-v0.1.0` nur noch den Tooltip von Show mask offen,
der als eigener kleiner Fix kommt.

## Umfang

- Werte, Texte und Windows-Dateien: `docs/parity/MAC_PARITY.md`, Abschnitte „Lücken nach MAC-002 bis MAC-010“ (Position unter dem
  Mauszeiger) und „Version in der Statusleiste“. Format des Positionsfelds: `docs/design.md`, „Statusleiste“, Feld 4.
- Mac: `mac/App/Views/StatusBar.swift`, Mausverfolgung im Bildbereich (`ImageArea.swift` bzw. `ImageCanvas.swift`), Umrechnung von
  Fenster- auf Bildkoordinaten über `ImageViewport`. Vorlage für die Version: `../SidereaStack/App/Views/BuildInfo.swift`.
- Kein neuer Kern: Den Wert unter dem Zeiger liest die Oberfläche aus dem angezeigten Bild; braucht es dafür einen Aufruf über `crates/ffi`,
  bekommt die Schnittstellenänderung einen eigenen Commit.

## Akzeptanzkriterien

- [ ] Wavelets und Save: Position und Wert unter dem Mauszeiger wie Windows, in Bildkoordinaten auch gezoomt, verschoben, gedreht und
      zugeschnitten; außerhalb des Bilds und in den anderen Schritten bleibt das Feld leer
- [ ] Rechts in der Statusleiste Version · Commit · Bauzeit mit dem Tooltip aus der Parity-Liste; Unbekanntes fällt weg; im Debug-Bündel
      und im Release-Bündel (`scripts/build-mac-release.sh`) richtig
- [ ] Bei Platzmangel bleibt die Statusleiste lesbar (kleinste Fenstergröße), die Reihenfolge des Wegfallens steht in `docs/design.md`
- [ ] Beide Punkte aus `docs/parity/MAC_PARITY.md` gelöscht; `docs/design.md` (Statusleiste, Mac-Abweichungen) nachgeführt; Hilfe unter
      `app/src/docs/pages/` geprüft, wo sie die Statusleiste beschreibt
- [ ] Probe von Hand als `### MAC-022 Statusleiste wie Windows (v0.1.0)` unter `## Mac` in `docs/parity/Handprobe.md`
- [ ] Tests grün (`xcodebuild test`), neue Tests einmal rot an einer Assertion
