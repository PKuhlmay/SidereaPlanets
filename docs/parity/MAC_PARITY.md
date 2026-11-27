# Mac nachziehen — Änderungen der Windows-App

Das Gegenstück zu `docs/parity/WINDOWS_PARITY.md`: Sammelstelle für alles, was die Windows-App (`app/`) bekommen hat und die
Mac-App (`mac/`) noch übernehmen muss. **Kein eigenes Ticket pro Anpassung:** diese Datei ist die Liste, im Mac-Ticket
(`mac/tickets/MAC-NNN-…`) genügt der Verweis darauf. Hier stehen nur offene Punkte: Der Commit, der einen Punkt auf dem Mac umsetzt,
löscht ihn (ein Abschnitt ohne Punkte fällt mit weg).

Quelle der Wahrheit für das Aussehen ist `docs/design.md`, für das Verhalten die Windows-Implementierung (Dateien im Punkt). Die
Windows-Seite ist Vue und Tauri und nicht übertragbar; übernommen wird das **Verhalten und die Werte**. Logik gehört in den
Rechenkern (`crates/`), dann muss der Mac nur die Oberfläche nachbauen.

Ein Punkt je Verhalten, mit den Werten (Größen, Deckkräfte, Token-Namen), den Windows-Dateien, den Texten und dem Hinweis, was
davon schon gemeinsam auf `main` liegt (Rechenkern, Doku). Abschnitte heißen wie das Feature, gruppiert nach der Windows-Version,
die es zuerst hatte.

`.githooks/commit-msg` verlangt diese Datei in jedem Commit, der die Windows-Oberfläche (`app/src`, `app/src-tauri/src`) ändert —
außer der Commit löscht einen erledigten Punkt aus `docs/parity/WINDOWS_PARITY.md` (Windows zieht nach) oder trägt
`[windows-only]` in der Nachricht. `scripts/parity-status.sh docs/parity/MAC_PARITY.md` listet die offenen Punkte je
Windows-Version.

## Stand vor dem ersten Mac-Release (Windows, 2026-11-12)

Die Mac-App gibt es noch nicht. Bis `MAC-001` fertig ist, ist der **gesamte Funktionsumfang der Windows-App** offen. Er steht nicht
als Checkbox-Liste hier, sondern in den Spezifikationen und Tickets, die das Mac-Ticket abarbeitet:

- Aussehen und Zustände: `docs/design.md` (Schrittleiste mit sieben Schritten, Panels, Bildbereich, Soll-Bilder in
  `docs/design/screenshots/`), Begriffe und Werte: `docs/ui.md`, Ablauf: `docs/workflow.md`.
- Verhalten und Grenzen: `docs/stacking.md`, `docs/wavelets.md`, `docs/formats.md`, offene Fragen: `docs/offene-fragen.md`.
- Anwenderdoku: `app/src/docs/pages/` (gleiche Markdown-Seiten, die Mac-App zeigt sie ebenfalls).
- Erledigte und offene Kriterien der Windows-App: `tickets/SP-*.md`.

Erst wenn `MAC-001` abgehakt ist, gilt die Regel „ein Punkt je Verhalten“: Von da an trägt jede Windows-Änderung hier ihren Mac-Punkt
ein.

## Tooltip von Show mask (Windows: SP-020, 2026-11-13)

- [ ] Der Tooltip des Häkchens **Show mask** in der Karte Masking sagt am Mac „Tints the part that is sharpened red.“, die Maske ist aber
      blau (`mask_overlay` im Kern, Hilfeseite `color-and-tones.md`). Windows schreibt „Tints the part that is sharpened blue. This only
      changes what you see here, not the image.“ (`app/src/components/MaskCard.vue`). Mac: `mac/App/Views/Wavelets/RefineCards.swift` (`MaskCard`).
