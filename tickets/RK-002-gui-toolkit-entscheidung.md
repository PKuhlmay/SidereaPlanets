# RK-002 GUI-Toolkit auswählen

- Status: erledigt (Entscheidung ohne Benchmark-Spike, siehe `docs/gui-toolkit.md`)
- Abhängig von: RK-001

## Ziel
Entscheiden, womit die Oberfläche gebaut wird, sodass sie dem Original in Aufbau und Bedienung nahekommt
(dichte Windows-Desktop-Oberfläche mit Reitern, Reglern, großem Bildbereich, Zoom).

## Akzeptanzkriterien
- [ ] Kandidaten (z. B. egui, iced, Slint) mit kleinem Spike verglichen: Bildanzeige mit Zoom/Pan,
  viele Regler, Reaktionszeit bei 16-Bit-Bildern — bewusst übersprungen, Tauri nach Gespräch gewählt;
  die Bildübertragung ist als Messwert in der App sichtbar (Ladezeit pro Frame)
- [x] Entscheidung mit Begründung in `docs/gui-toolkit.md`
