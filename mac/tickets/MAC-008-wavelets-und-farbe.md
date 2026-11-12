# MAC-008 Schritt Wavelets, Farbe und Functions

- Status: erledigt
- Abhängig von: MAC-007
- Teil von: MAC-001, Kriterium „Schritt Wavelets“

## Akzeptanzkriterien

- [x] Sechs Ebenen mit Reglern wie Windows (`WaveletsStep.vue`), Process, Live-Vorschau (Processing Area), Before (Vorher/Nachher),
      Do All mit Fortschritt, Pause, Cancel
- [x] Load Scheme und Save Scheme (native Dialoge, gleiches Dateiformat wie Windows)
- [x] Colour/LRGB, Functions-Karten Histogram, Gamma, RGB Align (inkl. Auto), RGB Balance, Contrast/Brightness wie Windows
      (`FunctionCards.vue`, `*Card.vue`)
- [x] Do All wirkt wie in der Windows-App auch auf Colour und LRGB

## Umsetzung

- Zustände und Aufrufe: `mac/App/Model/AppModel+Wavelets.swift` (Vorschau, Process, Do All, Schemes, Karten, Contrast/Brightness, Copy To, Hooks
  der Eigenschaften in `AppModel`), reine Rechnung (Werte und Voreinstellungen, verknüpfte Regler, Processing Area, Tonwerte, Histogramm-Hilfen,
  Differenz) in `WaveletLogic.swift`, nach `app/src/wavelet.ts`, `colour.ts`, `logic/{wavelet,colour,settings}.ts`. `WaveletBackend` sagt, was der
  Schritt vom Kern braucht (`SessionBackend` schließt es ein). Ansichten in `Views/Wavelets/`: `WaveletsPanel` (Colour, LRGB, sechs Ebenen, Advanced
  mit Schemes), `FunctionsPanel` (rechts: 15 Functions-Schaltflächen, Contrast/Brightness, Copy To), `FunctionCards` (Histogram, Gamma, RGB Align,
  RGB Balance), `WaveletControls` (Dezimalfeld, Segmente, Textlinks). `FloatCard` hat jetzt Aktionen (Reset, Auto) in der Titelzeile.
- Wie in Windows: Werte und Voreinstellungen kommen aus `waveletDefaultSettings()` des Kerns (Linear, Gaussian, Initial 1, Step 0, sechs Ebenen auf 1.0,
  Denoise 0.00, Sharpen 0.100); die Regler laufen von −5.0 bis 100.0, die Preview-Schaltfläche zeigt den Reglerwert und ist gesperrt. Jede Änderung der Werte,
  von Colour/LRGB oder der Processing Area rechnet den Ausschnitt neu (ein Aufruf zugleich, Änderungen währenddessen werden zu einem weiteren Aufruf
  zusammengefasst; „layer setting changed“, dann „Preview: n ms“ und „Calculating wavelets done“). Die erste Vorschau nach Stack lässt die Statusleiste
  mit der Stackgröße stehen. Process rechnet sofort (Cancel verwirft das Ergebnis), Do All sendet Werte und Farbmodus (Colour, LRGB, Gray), zeigt den
  Fortschritt je Ebene („Do_all processing“), öffnet danach Save; Automatic ruft Do All 600 ms nach der letzten Änderung ohne Weiterschalten. Wavelets gilt
  als erledigt, solange das Ergebnis zu Stack, Werten und Farbmodus passt (`DoAllKey`). Ein neuer Stack setzt die Ebenen zurück, außer Hold Wavelet
  Setting; Open project behält sie für den ersten Stack (`keepWaveletValuesForNextStack`).
- Karten und Farbwerte wie Windows: die Werte gehen immer auf das Ausgangsbild (Vorschau oder Do-All-Ergebnis) im Kern (`setColorSource` einmal je
  Ausgangsbild, `applyColors`); neutral zeigt das Bild ohne Aufruf. Histogram mit Input/Result, ziehbaren Dreiecken, Auto und Clip %, Gamma, RGB Align mit
  Auto (Max shift), RGB Balance; RGB Align und Balance sind bei Graustufen gesperrt („Needs a colour image“). Copy To, Load to, Difference (vierfach
  verstärkt) und Toggle wie in `colour.ts`. Functions-Schaltflächen ohne Karte melden „‹Name›: not available yet“.
- Mac-Art: Before als Schaltfläche in der Zoom-Leiste, Functions » neben Show, Load Scheme und Save Scheme zusätzlich in Settings… (dort auch die
  Processing Area, Standard 512), die Fortschrittsmeldung des Do All geht an Cancel (⌘.) und Pause im Menü Process. Ein Do All, das beim Cancel schon
  fertig war, meldet sein Ergebnis (wie bei Align). Eine Vorschau, die beim Ende eines Do All noch rechnet, wird verworfen, damit sie das Ergebnis
  nicht überdeckt (in Windows ein Wettlauf). Die Position unter dem Zeiger in der Statusleiste ist noch nicht gebaut.
- Save-Anschlussstellen (MAC-009): `saveAdjustments`, `ensureProcessedImage()`, Abschnitte `wavelet` und `colour` der Projektdatei im Windows-Format
  (`WaveletProject.swift`), Tests `WaveletSaveTests`.
- Geprüft am laufenden Fenster mit JUPITER.AVI (dunkel und hell): Vorschau im Ausschnitt mit gestricheltem Rahmen, Karten Histogram, Gamma, RGB Align,
  RGB Balance, Do All und Wechsel nach Save; Soll-Bilder `08`, `09` und `16`.
- Tests: `WaveletLogicTests`, `WaveletModelTests` (Attrappe: Vorschau, Process, Do All, Cancel, Pause, Before, Hold, Schemes, Karten, Copy To),
  `WaveletSaveTests`, `JupiterWaveletTests` (echter Kern: Do All auf dem Stack von JUPITER.AVI hat 1024×680 und ändert das Bild leicht, die Vorschau
  gleicht Do All im Ausschnitt, Gray, Karten, Histogramm mit 256 Klassen).
