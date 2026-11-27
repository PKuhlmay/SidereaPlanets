# SP-002 GUI-Toolkit auswählen

- Status: erledigt
- Abhängig von: SP-001

## Ziel
Entscheiden, womit die Oberfläche gebaut wird, sodass sie dem Original in Aufbau und Bedienung nahekommt
(dichte Desktop-Oberfläche mit Reitern, Reglern, großem Bildbereich, Zoom) — im modernen Erscheinungsbild.

## Akzeptanzkriterien
- [x] Spike für das gewählte Toolkit (Tauri) mit Messwerten: Bildübertragung Rust → Oberfläche, Zoom/Pan, viele Regler
- [x] Entscheidung mit Begründung und Messwerten in `docs/gui-toolkit.md`

## Änderung am Umfang
Ursprünglich sollten mehrere Kandidaten (egui, iced, Slint) per Spike verglichen werden. Das wurde ersetzt durch den
Spike nur für Tauri: Die Wahl fiel aus Designgründen (Web-Tooling für die Design-Agenten), und der Prototyp soll schnell
vorankommen. Die anderen Kandidaten sind weder gebaut noch gemessen; das steht auch so in `docs/gui-toolkit.md`.

## Ergebnis
Zoom, Pan und viele Regler laufen mit 60 Hz. Engpass ist die Bildübertragung (rund 20 Bilder/s bei 1024 × 680,
5 Bilder/s bei 2048 × 2048). Maßnahmen für die Live-Vorschau stehen in SP-007.
Spike-Code liegt auf dem Branch `spike/rk-002-tauri`.
