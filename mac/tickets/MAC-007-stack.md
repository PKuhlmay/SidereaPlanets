# MAC-007 Schritt Stack

- Status: erledigt
- Abhängig von: MAC-006
- Teil von: MAC-001, Kriterium „Schritt Stack“

## Akzeptanzkriterien

- [x] Stack mit Fortschritt, Pause und Cancel; Ergebnis im Bildbereich, Statusangaben („Stack size (n=…) - (…)“)
- [x] Wirksame Optionen wie Windows (`StackStep.vue`): Nearestby Alignpoints, Correct geometry, Normalization, Edge pixels
- [x] Unwirksame Optionen (Resampling, Drizzling, StackSize, Stretch histogram, De-rotate, Flat/Dark/Reference …) gesperrt mit
      „Not available yet“
- [x] Swift-Test oder Kern-Test: Stack von JUPITER.AVI hat 1024×680 und n=10

## Umsetzung

- Zustände und Aufruf: `mac/App/Model/AppModel+Stack.swift` (Stack mit Fortschritt, Karte Alignmentpoints, Ringe), reine Rechnung (Werte und
  Voreinstellungen, was an den Kern geht, Texte) in `StackLogic.swift`, nach `app/src/stack.ts` und `app/src/logic/stack.ts`. Panel
  `Views/StackPanel.swift`, Ringe in `ImageOverlays.swift`, Karte in `FloatingCards.swift`, Show-Menü in `ImageArea.swift`.
- Wie in Windows: wirksam sind Use Nearestby Alignpoints (3), Correct geometry (an), Normalization of frame intensity (aus) und Do not stack
  pixels at the frameedge (0); dazu gehen die angehakten Alignpoints (Karte Alignmentpoints), die angehakten Frames, Alignmentbox size,
  Max Alignpoint movement und R.o.Interest an den Kern. Gesperrt mit Tooltip und „Not available yet“ (bei langem Namen darunter,
  rechtsbündig): StackSize Minimized, Maximized, R.of Interest (Default bleibt gewählt), Stretch histogram after stacking, De-rotate images,
  Resampling, Drizzling. Advanced zeigt „6 more settings“; die Gruppe Create AVI (Codec, Save Aligned, Save Integrated …) folgt mit
  MAC-009 und steht deshalb noch nicht im Panel.
- Statusleiste wie Windows: „Stacking frame: n“ während des Laufs, danach „Stack size (n=10) - (1024x680)“ und „Stacking Range : 0-255“;
  Ergebniszeile „10 frames · 1024×680“. Nach dem Stack öffnet sich Wavelets (dessen Panel bleibt bis MAC-008 der Platzhalter). Cancel (⌘.)
  und Pause/Resume wirken in der Werkzeugleiste und im Menü Process auch auf Stack (`SessionOperation.stack`); ein abgebrochenes „Stack
  again“ lässt den früheren Stack stehen.
- Bildbereich wie Windows (`pickImage`): Der Schritt Stack zeigt bis zum Stack den Frame (mit Ringen und Karte), danach das Summenbild mit der Beschriftung
  „Stack of n frames“; Wavelets und Save zeigen ebenfalls das Summenbild, die Schritte davor immer den Frame. Der Schritt Stack hat das
  Show-Menü des Stack-Reiters (Show Full Image, Show AlignSections, Show Stackgraph, Show Stacklist; die ersten, dritten und vierten ohne
  Wirkung wie in Windows); Show AlignSections zeigt die Ringe der Punkte mit dem Verschiebungsstrich (vierfach) und die Karte Alignmentpoints.
- Verwerfen: Der Kern lässt den Stack fallen, wenn Set Alignpoints, Align oder Limit laufen; `AppModel.progress` setzt dann auch `stacked` auf `nil`
  (der Schritt ist wieder gesperrt, der Bildbereich zeigt den Frame).
- Der Rechenkern und die Schnittstelle blieben unverändert; `SessionBackend` bekam `stack` und `stackImage`. Der Aufruf `-run` kennt jetzt `stack` und
  `stackpaused`.
- Noch nicht im Stack-Schritt: Save image unter dem Ergebnis und Create AVI (MAC-009).
- Geprüft am laufenden Fenster mit JUPITER.AVI (dunkel und hell): Schritt vor dem Stack mit Ringen und Karte (Soll-Bild `06`), Lauf mit
  Pause, fertiger Schritt mit Advanced (Soll-Bild `07`), Wavelets danach mit dem Summenbild.
- Tests: `StackLogicTests`, `StackModelTests` (Attrappe), `JupiterModelTests` (echter Kern: 1024×680, n=10, erneutes Align verwirft den Stack).
