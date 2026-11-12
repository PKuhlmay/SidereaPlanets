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

### Schritte per Tastatur (MAC-010)

- [ ] Die sieben Schritte der Schrittleiste lassen sich per Tastenkürzel öffnen: Schritt n (1 Open bis 7 Save) mit einem Kürzel, am Mac
  ⌥⌘1 bis ⌥⌘7 im Menü Process > Go to Step. Wie ein Klick auf die Schrittleiste: ein gesperrter Schritt und jede laufende Berechnung
  sperren das Kürzel. Eine Windows-Belegung (z. B. Alt+1 bis Alt+7) legt das Windows-Ticket fest. Mac:
  `mac/App/Commands/StepMenuItems.swift`, `AppModel.show(_:)`.

### Go to best frame (MAC-011)

- [ ] Neue Funktion (im Original nicht vorhanden): eine Schaltfläche **Go to best frame** setzt nach Align den besten Frame als
  Referenz für ein neues Set Alignpoints. Sie steht an zwei Stellen, Standardschaltfläche (kein gefüllter Stil, Tokens wie die übrigen
  Panel-Schaltflächen): im Panel des Schritts Limit **unter dem Qualitätsdiagramm** (volle Panelbreite, 8 px Abstand zum Diagramm wie
  zwischen den Elementen des Panels) und in der Karte **Framelist** als zweite Zeile **unter** Check all · Check none · Empty List (volle Kartenbreite,
  kompakte Größe, 24 px, 12 px Text, 8 px seitlicher Rand). Tooltip: „Show the best frame and open Alignpoints to use it as
  reference“; solange Align nicht gelaufen ist (Schritt unter *aligned*, keine Bewertung) ist sie gesperrt, Tooltip „Run Align first“,
  ebenso während einer Berechnung.
  Ein Klick (a) zeigt den **besten Frame**, den ersten der Rangliste (Framelist-Reihenfolge: höchste Qualität zuerst, bei gleicher
  Qualität die kleinere Frame-Nummer), im Bildbereich und in der Frame-Leiste (Goto Frame und Regler stehen auf ihm), (b) öffnet den
  Schritt **Alignpoints** wie ein Klick auf dessen Pille, sodass Set Alignpoints den gezeigten Frame als Referenz nimmt, und (c) merkt
  sich die Frame-Nummer für den Hinweis. **Nichts wird verworfen**: Bewertung, Auswahl der Frames, Limit und Stack bleiben, bis
  Set Alignpoints (oder Align) wieder läuft; erst das löscht sie wie bisher, und mit ihnen den Hinweis. Auch ein neues Öffnen löscht ihn.
  Hinweis oben im Panel des Schritts Alignpoints, **über** dem Threshold-Regler, solange er gemerkt ist (Hinweis-Komponente aus
  `docs/design.md`: Fläche `--accent-soft`, 3 px Streifen links in `--accent`, Radius `--r`, Text 12 px `--text`, Innenabstand
  8 px oben und unten, 10 px rechts): `Best frame: N — run Set Alignpoints and Align again to use it as reference.` Das N zählt von 1
  wie die Frame-Leiste („Best frame: 295“); zwischen „N“ und „run“ steht ein Gedankenstrich (—). Der Hinweis bleibt auch, wenn die
  Anwenderin danach einen anderen Frame zeigt; dann gilt dieser als Referenz, der Hinweis nennt weiter den besten.
  Die Anwenderdoku (`app/src/docs/pages/frame-quality-and-limit.md`, Abschnitt „Use the best frame as reference“, dazu Verweise in
  `align-your-frames.md` und `steps-alignpoints-align-limit.md`) liegt schon gemeinsam auf `main`. Mac: `AppModel.goToBestFrame()`
  und `canGoToBestFrame` (`mac/App/Model/AppModel+Align.swift`), `AlignData.bestFrame` und `bestFrameHint` samt Text `bestFrameHint(frame:)`
  (`mac/App/Model/AlignLogic.swift`), Schaltflächen `GoToBestFrameButton` und `HintNote` (`mac/App/Views/AlignPanels.swift`),
  Karte (`mac/App/Views/FloatingCards.swift`).
