# MAC-007 Schritt Stack

- Status: offen
- Abhängig von: MAC-006
- Teil von: MAC-001, Kriterium „Schritt Stack“

## Akzeptanzkriterien

- [ ] Stack mit Fortschritt, Pause und Cancel; Ergebnis im Bildbereich, Statusangaben („Stack size (n=…) - (…)“)
- [ ] Wirksame Optionen wie Windows (`StackStep.vue`): Nearestby Alignpoints, Correct geometry, Normalization, Edge pixels
- [ ] Unwirksame Optionen (Resampling, Drizzling, StackSize, Stretch histogram, De-rotate, Flat/Dark/Reference …) gesperrt mit
      „Not available yet“
- [ ] Swift-Test oder Kern-Test: Stack von JUPITER.AVI hat 1024×680 und n=10
