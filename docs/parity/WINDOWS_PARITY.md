# Windows nachziehen — Änderungen der Mac-App

Sammelstelle für alles, was die Mac-App (`mac/`, SwiftUI) bekommen hat und die Windows-App (`app/`, Tauri und Vue) noch
übernehmen muss; beide liegen auf `main`, die Gegenrichtung steht in `docs/parity/MAC_PARITY.md`. **Kein eigenes Ticket pro
Anpassung:** diese Datei ist die Liste, im Windows-Ticket (`tickets/RK-NNN-…`) genügt der Verweis darauf. Hier stehen nur
offene Punkte: Der Commit, der einen Punkt in Windows umsetzt, löscht ihn (ein Abschnitt ohne Punkte fällt mit weg); was erledigt
wurde, zeigt die Git-Historie dieser Datei.

Quelle der Wahrheit für das Aussehen ist `docs/design.md`, für das Verhalten die Mac-Implementierung (Dateien im Punkt). Die
Mac-Seite ist SwiftUI und nicht übertragbar; übernommen wird das **Verhalten und die Werte**. Gemeinsame Dateien (Rechenkern in
`crates/`, Doku-Seiten unter `app/src/docs/pages/`, Beispieldaten-Beschreibungen) liegen schon auf `main`.

Ein Punkt je Verhalten, mit den Werten (Größen, Deckkräfte, Token-Namen), den Mac-Dateien, den Texten und dem Hinweis, was davon
schon gemeinsam auf `main` liegt. Abschnitte heißen wie das Feature, gruppiert nach der Mac-Version, die es zuerst hatte.

`.githooks/commit-msg` verlangt diese Datei in jedem Commit, der die Mac-Oberfläche (`mac/`) ändert — außer der Commit löscht einen
erledigten Punkt aus `docs/parity/MAC_PARITY.md` (Mac zieht nach) oder trägt `[mac-only]` in der Nachricht.
`scripts/parity-status.sh` listet die offenen Punkte je Mac-Version.

Bis `MAC-001` fertig ist, ist die Windows-App die Vorlage: hier stehen nur Verhalten, die der Mac vorab hat, weil sie nützlich sind
und keine reine Mac-Konvention (Menüleiste, ⌘-Kürzel, Gesten und native Dialoge stehen als Abweichung in `docs/design.md`,
„Mac-App“). Danach führt der Mac, und diese Liste füllt sich mit jeder Änderung.

## v0.1.0 (Mac, in Arbeit)

### Schritt Open: Dateiangaben (MAC-005)

- [ ] Das Panel des Schritts Open zeigt unter dem Dateinamen die Zeilen Format, Codec, Size, Bit depth, Frames, Colour, Frame rate
  und, wo die Datei sie hat, Observer, Instrument und Telescope (12 px, Beschriftung `--muted`, Wert `--text`), statt nur Codec,
  Größe und Frame-Zahl in einer Zeile. `RecordingInfo` des Rechenkerns liefert alle Werte schon. Mac:
  `mac/App/Model/RecordingDetails.swift`, `mac/App/Views/StepPanel.swift` (`OpenStepView`).

### Frame-Leiste: vorheriger und nächster Frame (MAC-005)

- [ ] In der Frame-Leiste stehen zwischen Goto Frame und Regler zwei Schaltflächen „vorheriger Frame“ und „nächster Frame“
  (Pfeil links und rechts, ohne Rahmen, am ersten bzw. letzten Frame gesperrt; Tooltip nennt das Kürzel). Der Mac belegt sie mit ⌘[ und
  ⌘], eine Windows-Belegung (z. B. ← und →, solange kein Feld den Fokus hat) legt das Windows-Ticket fest. Mac:
  `mac/App/Views/FrameBar.swift`, `AppModel.showNextFrame(by:)`.
