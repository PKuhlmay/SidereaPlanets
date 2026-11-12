# MAC-009 Schritt Save, Export und Projektdatei

- Status: in Arbeit
- Abhängig von: MAC-008
- Teil von: MAC-001, Kriterium „Schritt Save“

## Akzeptanzkriterien

- [ ] Save image (⌘S) in allen Formaten der Windows-App (`export_formats`), mit Farbanpassungen, nativer Dialog
      (Formate, Dialog und ⌘S fertig; offen bleiben die Farbanpassungen: die Karten RGB Align, RGB Balance, Histogram, Gamma und
      Contrast/Brightness gibt es erst mit MAC-008, `AppModel.saveAdjustments` ist die Stelle, an der sie eingehen)
- [x] Save Aligned und Save Integrated mit Fortschritt
- [x] Save project und Open project (gleiches Format wie Windows, `crates/io/src/project.rs`)
- [x] Nach erneutem Align ist Save image gesperrt, bis neu gestapelt ist (Session)
- [x] Ganzer Ablauf mit JUPITER.AVI bis TIFF 16 Bit läuft durch (Test oder dokumentierter Lauf)

## Umsetzung

- Zustände und Aufrufe: `mac/App/Model/AppModel+Save.swift` (Save image, Save Aligned, Save Integrated, Save project, Open project), reine Rechnung
  (Create-AVI-Werte, Projektabschnitte, `JSON`, Dateinamen) in `SaveLogic.swift`, Dialoge in `SavePanels.swift`; nach `app/src/saving.ts` und
  `app/src/logic/{exporting,projectfile,stack}.ts`. Ansichten in `Views/Save/SaveViews.swift`: Schritt Save, Menü Save image
  (`SaveImageMenu`: sieben Dateitypen im Schritt Stack unter dem Ergebnis, fünf im Schritt Save als Aktion) und die Gruppe Create AVI im
  Advanced des Stack-Panels (Codec, Compress Qual, Framerate, Save Aligned mit Image size, Sort On Quality, Use Prefilters, Save Integrated
  mit Moving average/Compressed (gesperrt) und Average of n Frames; die Werte liegen in `StackOptions`).
- Die Modellfunktionen bekommen den Dialog als Closure (`SaveChooser`, `ProjectChooser`), die Tests laufen dieselbe Logik ohne Fenster.
  Save image wie Windows: Quelle ist das geschärfte Bild in Wavelets und Save, sonst der Stack; vor dem Dialog `requireImage`; ein
  gespeichertes geschärftes Bild setzt `progress.saved` und öffnet Save (Ergebnis: der Dateiname, „Open another recording“). Nach neuem
  Align, Limit oder Set Alignpoints ist `stacked` weg und Save image gesperrt, bis neu gestapelt ist. Save Aligned und Save Integrated
  brauchen Align und laufen als Lauf `.sequence` (Cancel ⌘., Pause, „Writing frame: n“).
- Projektdatei wie Windows (`.rkproj`, `crates/io/src/project.rs`): Abschnitte `align` (alle Schlüssel von `settings.align` ohne `threshold`)
  und `stack` (die Schlüssel, die der Mac hat, einschließlich Create AVI); beim Öffnen zählt ein Wert nur, wenn der Schlüssel da ist und denselben
  Typ hat (`SectionReader`, wie `mergeKnown`). Fehlende Dateien, leere Aufnahme und neuere Version melden sich wie in Windows. Die Ergebnisse
  stehen nicht in der Datei.
- Mac-Art: Menü File mit Save Image… (⌘S), Save Project… und Open Project…; Open und Save in der Werkzeugleiste (Windows: dieselben
  Schaltflächen); `NSSavePanel` mit dem gewählten Typ (`docs/design.md`, „Mac-App: bewusste Abweichungen“).
- Anschlussstellen für MAC-008 (Wavelets): `saveAdjustments` (Farbwerte, `nil` solange neutral), `ensureProcessedImage()` (Do All vorher, wie
  `doAllIsUpToDate() || runDoAll()`), `projectSections`/`applyProjectSections` (Abschnitte `wavelet` und `colour` ergänzen) und
  `progress.waveletsDone`. Bis dahin meldet das Speichern des geschärften Bildes „Do All: not available yet“. Ein Projekt, das ein Abschnitt
  enthält, den der Mac noch nicht kennt (`wavelet`, `colour`), verliert ihn beim erneuten Speichern.
- Tests: `SaveLogicTests`, `SaveModelTests` (Attrappe), `JupiterSaveTests` (echter Kern: TIFF 16 bit, Save Aligned, Projekt-Rundlauf),
  im Paket `SaveFlowTests` (JUPITER.AVI bis TIFF 16 bit, Projektdatei).
