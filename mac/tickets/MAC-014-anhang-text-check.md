# Wording-Review Mac-App „Siderea Planets" — Verständlichkeit für Einsteiger

Stand: 2026-11-12, `main` @ af6f1d2. Geprüft: alle sichtbaren Strings unter `mac/App/**` (Schrittleiste, Panels, Buttons,
Felder, Häkchen, Radios, Menüs inkl. Show-Menü, Tooltips, Statusleiste, Ergebniszeilen, Hinweise, Fehler, „Not available yet",
Settings, Help-Fenster). Pfade relativ zu `mac/App/`. Read-only, nichts im Repo geändert.

**Art:**
- **umbenennen** = Text ersetzen.
- **Tooltip** = the reference program-Begriff behalten und per `.help()` erklären, damit Anleitungen zu the reference program weiter passen.
- **beides** = sichtbarer Text leicht entschärft, und der Original-Begriff steht im Tooltip („the reference program: …").

**Schwere:**
- **hoch** = Einsteiger versteht nicht, was passiert, oder wird aktiv in die Irre geführt.
- **mittel** = unklar oder inkonsistent, lässt sich aber erschließen.
- **niedrig** = Kosmetik (Schreibweise, Groß/klein).

Faustregel für die Abwägung: **Schrittnamen und Hauptaktionen** (das, was in YouTube-Tutorials zu the reference program genannt wird: Alignpoints,
Align, Limit, Stack, Wavelets, Do All) **behalten + Tooltip**. **Feldbeschriftungen, Statustexte, Kürzel und Tippfehler**
umbenennen. Dort hilft der Originalbegriff niemandem, und in Tutorials werden sie selten wörtlich zitiert.

---

## 0 Werkzeugleiste, Menüleiste, Fensterrahmen

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 0.1 | Toolbar **„Open"** / **„Save"** (Projekt) | Views/ContentView.swift:51, :55 | Heißen genauso wie Schritt 1 „Open" und Schritt 7 „Save". Ein Einsteiger erwartet „Open" = Aufnahme öffnen und „Save" = Bild speichern. Tatsächlich ist es die Projektdatei. | „Open Project…" / „Save Project…" (oder Icon + Label „Project ▾"-Menü) | umbenennen | hoch |
| 0.2 | Menü File **„Open…"** (⌘O) | SidereaPlanetsApp.swift:43 | Öffnet eine Aufnahme, heißt aber anders als der Toolbar-Button „Select Recording…" und kollidiert mit „Open Project…" direkt darunter. | „Open Recording…" (oder „Select Recording…" wie Toolbar) | umbenennen | mittel |
| 0.3 | **„Open Recent"** / **„Recent Recordings"** / **„Recent"** | SidereaPlanetsApp.swift:46, ContentView.swift:46, StepPanel.swift:186 | Drei Namen für dieselbe Liste. „Open Recent" ist der macOS-Standard, also okay. Panel-Überschrift „Recent" angleichen. | Panel: „Recent Recordings" | umbenennen | niedrig |
| 0.4 | **„No recent files"** | ContentView.swift:76, StepPanel.swift:188 | Spricht von „files", sonst überall „recordings". | „No recent recordings" | umbenennen | niedrig |
| 0.5 | Menü **„Process"** (Cancel, Pause, Go to Step) | SidereaPlanetsApp.swift:62 | Gleicher Name wie der Button „Process" im Wavelets-Schritt, der etwas ganz anderes tut. | „Run" oder „Calculation" | umbenennen | mittel |
| 0.6 | **„Save Image…"** (Menü) vs. **„Save image"** (Panel) | SidereaPlanetsApp.swift:54, SaveViews.swift:42, StepLogic.swift:42 | Schreibweise uneinheitlich. | Überall „Save Image…" (Title Case + Ellipse, siehe G.1/G.2) | umbenennen | niedrig |
| 0.7 | Menü „Save Project…" steht **vor** „Open Project…" | SidereaPlanetsApp.swift:57–59 | Unübliche Reihenfolge. Öffnen gehört nach oben in die Open-Gruppe. | Open Project… in die Gruppe zu Open/Open Recent verschieben | Struktur | niedrig |
| 0.8 | **„Fit Image"** (Menü) vs. **„Fit"** (Zoomleiste) | SidereaPlanetsApp.swift:79, ImageArea.swift:160 | Leichte Abweichung, sonst unproblematisch. | Menü „Zoom to Fit" (macOS-Konvention), Leiste „Fit" | umbenennen | niedrig |
| 0.9 | Pause-Tooltip „Hold the calculation; press again to go on" | ContentView.swift:64 | Verständlich. „Hold" ist mit dem Link „Hold" (Contrast/Brightness) belegt. | „Pause the calculation; press again to resume" | umbenennen | niedrig |
| 0.10 | Fenstertitel während Rechnung: „Siderea Planets · **Wavelet** 40%" | AppModel+Wavelets.swift:281, :302 | Job heißt „Wavelet", der Button aber „Do All" bzw. „Process". | Jobnamen = Buttonname: „Do All", „Process" | umbenennen | niedrig |
| 0.11 | NSOpenPanel-Button **„Select"** | Model/RecordingPicker.swift:17 | Generisch. | „Open" (Systemstandard) | umbenennen | niedrig |
| 0.12 | Fehler „The calculation core did not start: …" | SidereaPlanetsApp.swift:31 | fatalError, ist für den Nutzer unsichtbar. | — | — | niedrig |

## Schrittleiste (alle Schritte) und Panel-Fuß

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| S.1 | Schrittname **„Alignpoints"** | Model/StepLogic.swift:24 | Zusammengeschriebenes Kunstwort. Ein Einsteiger weiß nicht, dass es „Ankerpunkte für die Ausrichtung" sind. | Behalten (Tutorial-Begriff). Tooltip auf der Pille: „Alignpoints: reference marks on surface details that every frame is lined up on". Optional sichtbar „Align Points" (mit Leerzeichen). | Tooltip | hoch |
| S.2 | Schrittname **„Limit"** | StepLogic.swift:26 | Ein Verb ohne Objekt; was begrenzt wird, ist unklar. | Behalten + Tooltip „Limit: choose which frames go into the stack (the sharpest ones)". Alternativ sichtbar „Pick Frames". | Tooltip | hoch |
| S.3 | Schrittname **„Wavelets"** | StepLogic.swift:28 | Mathematischer Fachbegriff; ein Einsteiger sucht „Sharpen". | Behalten (DER the reference program-Begriff) + Tooltip „Wavelets: sharpening in layers from fine to coarse detail" | Tooltip | mittel |
| S.4 | Schrittname **„Stack"** | StepLogic.swift:27 | Für Astro-Einsteiger halbwegs bekannt, aber englisch-fachlich. | Behalten + Tooltip „Stack: average the kept frames into one low-noise image" | Tooltip | niedrig |
| S.5 | Aktion **„Set Alignpoints"** | StepLogic.swift:37 | Wie S.1. Außerdem ist unklar, dass es automatisch platziert und nicht von Hand. | Behalten + Tooltip „Place alignpoints automatically on the current frame" | Tooltip | mittel |
| S.6 | Aktion **„Do All"** | StepLogic.swift:41 | Klingt nach „alle Schritte ausführen". Tatsächlich wendet es die Schärfung aufs ganze Bild an. | Behalten + Tooltip „Do All: apply the sharpening to the whole image (the preview shows only the Processing Area)". Alternativ sichtbar „Apply to Whole Image". | beides | hoch |
| S.7 | Aktion **„Limit"**, **„Align"**, **„Stack"** = gleich wie Schrittname | StepLogic.swift:38–40 | Für Einsteiger bleibt offen, was der Klick tut. | Keine Umbenennung; Tooltips an den Buttons (z. B. Limit: „Keep the frames chosen above") | Tooltip | mittel |
| S.8 | Satz Alignpoints: „Marks the details the frames are lined up on. **A lower threshold gives more points.**" | StepLogic.swift:50 | Widerspricht der Slider-Beschriftung „weakest … strongest" (siehe 2.1). „threshold" wird nicht erklärt. | „Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer." | umbenennen | mittel |
| S.9 | Satz Wavelets: „Raise the first layers for fine detail, **then apply them to the whole image**." | StepLogic.swift:54 | Sagt nicht, mit welchem Button. | „… then press Do All to apply them to the whole image." | umbenennen | mittel |
| S.10 | Satz Open: „Choose a recording: a SER or AVI video or a series of images." | StepLogic.swift:49 | Gut. Optional: Drag & Drop erwähnen. | „… You can also drop it onto the window." | umbenennen | niedrig |
| S.11 | Sperr-Tooltip „Run **Set Alignpoints** first" / „Run **Do All** first" | StepLogic.swift:115 | Grammatisch ok. „Run Do All first" liest sich holprig. | „First press Set Alignpoints" / „Open a recording first" passt | umbenennen | niedrig |
| S.12 | „**Step 2 of 7**" | Views/StepPanel.swift:43 | Gut. | — | — | — |
| S.13 | „**‹Action› again**" (z. B. „Do All again", „Limit again") | StepPanel.swift:90 | „Limit again", „Stack again" klingen abgehackt. | „Run Limit Again" / „Run Again" | umbenennen | niedrig |
| S.14 | „Running it again discards the steps after it." | StepPanel.swift:95 | „discards the steps" ist ungenau; verworfen werden die Ergebnisse. | „Running it again discards the results of the later steps." | umbenennen | niedrig |
| S.15 | „**Continue: Limit ›**" / „**Then: Align**" | StepPanel.swift:101, :116 | Verständlich. Das „›" ist als einziges Pfeilzeichen ungewohnt, ok. | — | — | niedrig |
| S.16 | Tooltip „Comes in a later version" | StepPanel.swift:112 | Sonst heißt es überall „Not available yet". | „Not available yet" | umbenennen | niedrig |
| S.17 | „**Open another recording**" | StepPanel.swift:80 | Heißt anders als „Select Recording…". | „Select Another Recording…" | umbenennen | niedrig |
| S.18 | Ergebnis Alignpoints „**82 set**" | Model/AppModel.swift:146 | Ohne Substantiv; in der Pille steht „Alignpoints · 82 set". | „82 alignpoints" | umbenennen | niedrig |
| S.19 | Ergebnis Align „**best quality 100 %**" | AppModel.swift:147 | Die Qualität ist relativ zum besten Frame normiert (AlignLogic.swift:181–183), deshalb steht hier **immer 100 %**. Die Angabe ist also inhaltsleer und führt in die Irre. | „10 frames rated" | umbenennen | hoch |
| S.20 | Ergebnis Limit „8 of 10 kept" | AppModel.swift:148 | Gut. | „8 of 10 frames kept" | umbenennen | niedrig |
| S.21 | Ergebnis Wavelets „**applied to the whole image**" | AppModel.swift:151 | Klein geschrieben, ohne Subjekt; in der Pille: „Wavelets · applied to the whole image". | „Sharpening applied" | umbenennen | niedrig |

## 1 Open

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 1.1 | Leere Bildfläche: „Press Select Recording to open a video or a series of images" | Views/ImageArea.swift:26 | Ok. Drag & Drop fehlt. | „Select a recording or drop a video or images here" | umbenennen | niedrig |
| 1.2 | „**No image yet**" | ImageArea.swift:26 | Ohne Handlungshinweis. | „No image to show yet" | umbenennen | niedrig |
| 1.3 | Dateiinfo-Zeilen „Format · Codec · Size · **Bit depth** · Frames · **Colour** · Frame rate · Observer · Instrument · Telescope" | Model/RecordingDetails.swift:22–29 | „Codec" und „Bit depth" sind Fachbegriffe, für eine reine Info aber hinnehmbar. „Colour" steht in UK-Schreibung (siehe G.4). | Codec und Bit depth: Tooltip. „Colour" → je nach G.4 | Tooltip | niedrig |
| 1.4 | Schrittleiste rechts: „DIB · 1024×680 · 10 frames" | RecordingDetails.swift:10 | „DIB" (Codec-Kürzel) steht an prominenter Stelle und sagt einem Einsteiger nichts. | Codec weglassen oder ans Ende: „1024×680 · 10 frames · DIB" | umbenennen | niedrig |

## 2 Alignpoints

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 2.1 | Slider-Kopf „**weakest** · **Threshold** · **strongest**" | Views/AlignPanels.swift:25–31 | Ein Einsteiger weiß nicht, dass links mehr Punkte heißt (schwache Details zählen mit). „Threshold" erklärt nichts. Der Satz oben spricht von „lower threshold". | „more points · Threshold · fewer points". „weakest/strongest" in den Tooltip: „Threshold: how strong a detail must be to get a point". | beides | hoch |
| 2.2 | Nach dem Setzen: „**Alignpoints: 82**" statt „Threshold" | AlignPanels.swift:27 | Der Slider verliert seine Beschriftung; nicht mehr klar, was er regelt. | „Threshold · 82 points" | umbenennen | mittel |
| 2.3 | „**Minimum distance between**" | AlignPanels.swift:41 | Satzfragment; „between what?". Einheit (px) fehlt. | „Min. distance between points (px)" | umbenennen | hoch |
| 2.4 | „**Min distance from edge**" | AlignPanels.swift:42 | Abkürzung „Min" ohne Punkt, ungleich zu 2.3; Einheit fehlt; Rand von Bild oder ROI? | „Min. distance from image edge (px)" | umbenennen | mittel |
| 2.5 | Button „**R.o.Interest**" | AlignPanels.swift:43 | Kryptisches Kürzel. In der App gibt es 4 Schreibweisen: R.o.Interest / R.of Interest / ROI / Region of Interest (siehe G.6). | „Draw Region…" (Tooltip: „Region of Interest (ROI): only this area is used for alignment"). Sichtbar mindestens „Region of Interest". | beides | hoch |
| 2.6 | Tooltip „Draw the region that alignment is based on; press again to drop it" | AlignPanels.swift:46 | Gut. „drop" ist doppeldeutig (fallen lassen vs. entfernen). | „…; press again to remove it" | umbenennen | niedrig |
| 2.7 | Hinweis „Best frame: 7 — run Set Alignpoints and Align again to use it as **reference**." | Model/AlignLogic.swift:220 | „reference" bleibt unerklärt. | „Best frame: 7. Run Set Alignpoints and Align again to line all frames up on this one." | umbenennen | mittel |
| 2.8 | „Advanced · 5 more settings" | Views/Controls.swift:136–138 | Gut. | — | — | — |
| 2.9 | Gruppe „**Set Alignpoint parameters**" | AlignPanels.swift:52 | Singular, steif; doppelt zur Schrittüberschrift. | „Point placement" | umbenennen | niedrig |
| 2.10 | „**Intensity selection**" + Radios „Default · **3x3 area** · **Lowest pixelvalue**" | AlignPanels.swift:53–58 | Für Einsteiger unverständlich: wonach wird hier ausgewählt? „pixelvalue" ist zusammengeschrieben, „3x3" steht mit x statt ×. | Behalten (seltene Profi-Option) + Tooltips. „Lowest pixel value", „3×3 area". Tooltip Gruppe: „How the brightness at a point is measured when placing points". | beides | mittel |
| 2.11 | „**Lo**" / „**Hi**" | AlignPanels.swift:63–68 | Kürzel; Bezug (Helligkeitsbereich 0–255) unklar. | „Min. brightness" / „Max. brightness" oder Tooltip „Only pixels between Lo and Hi (0–255) get points" | beides | mittel |
| 2.12 | „Number of Alignpoints" (Read-only) | AlignPanels.swift:73 | Ok. Wirkt wie ein Eingabefeld. | — | — | niedrig |
| 2.13 | „Keep Alignpoints inside **ScanFrame**" (gesperrt) | AlignPanels.swift:77 | „ScanFrame" ist Binnenmajuskel und nirgends erklärt. | „Keep points inside the scan frame" + „Not available yet" | umbenennen | niedrig |
| 2.14 | Fehler „No alignpoints found: try other **Lo and Hi** values" | Model/AppModel+Align.swift:108 | Verweist auf versteckte Advanced-Felder; die naheliegende Lösung (Threshold nach links) fehlt. | „No alignpoints found: move the Threshold slider left, or widen Lo/Hi under Advanced" | umbenennen | hoch |
| 2.15 | Fehler „The region is too small for **Min distance from edge**" | AppModel+Align.swift:93 | Ok, solange 2.4 gleich benannt wird. | „The region is too small for the distance from the edge; draw a larger region" | umbenennen | niedrig |
| 2.16 | Status „82 alignpoints set" | AppModel+Align.swift:112 | Gut. | — | — | — |

## 3 Align

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 3.1 | Segmente „**No Align** · **Scan Frames**" + „Not available yet" | AlignPanels.swift:90–91 | Steht ganz oben, ist gesperrt und unverständlich. „No Align" wirkt sogar ausgewählt. | Gesperrt nach Advanced verschieben, oder Tooltip „Alignment modes of the reference program; not available yet" | Struktur + Tooltip | mittel |
| 3.2 | „**Alignmentbox size**" | AlignPanels.swift:92 | Zusammengeschrieben; Einheit fehlt; Bedeutung (Kantenlänge der Box um jeden Punkt) unklar. | „Box size per point (px)" + Tooltip „the reference program: Alignmentbox size" | beides | hoch |
| 3.3 | „**Max Alignpoint movement**" | AlignPanels.swift:93 | Einheit fehlt, Wirkung unklar. | „Max. point shift (px)" + Tooltip „How far a point may move between frames" | beides | mittel |
| 3.4 | Gruppe „Alignment setup" | AlignPanels.swift:97 | Ok. | — | — | niedrig |
| 3.5 | „**Show Alignment**" (aktiv, aber ohne Wirkung) | AlignPanels.swift:98, AlignLogic.swift:41 | Wird nirgends gelesen (nur in AlignLogic deklariert). Ein Häkchen, das nichts tut, verwirrt. | Sperren + „Not available yet" | Verhalten | mittel |
| 3.6 | „Align by **Centre of gravity**", „**Lum. Threshold**", „Estimate Rotation" (gesperrt) | AlignPanels.swift:100–110 | Kürzel „Lum." und UK-„Centre". Gesperrt, daher wenig schlimm. | „Brightness threshold" | umbenennen | niedrig |
| 3.7 | Fortschritt „**Loading frame: 42**" während Align | AppModel+Align.swift:50–53 (Default-Label), :164 | Beim Align steht „Loading", gemacht wird aber Ausrichten. | Label „Aligning frame" | umbenennen | mittel |
| 3.8 | Status „Alignment finished." | AppModel+Align.swift:175 | Einziger Statustext mit Punkt am Ende. | „Alignment finished" | umbenennen | niedrig |

## 4 Limit

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 4.1 | „**Lowest Quality (%)**" | Model/AlignLogic.swift:17 | Was „lowest" heißt, bleibt offen: Untergrenze oder schlechteste Frames? | „Min. quality (%)" + Tooltip „the reference program: Lowest Quality – keep frames at least this good (relative to the best)" | beides | mittel |
| 4.2 | „**Best Frames (%)**" | AlignLogic.swift:18 | Verständlich. Das Feld dazu ist das von 4.1 (geteiltes Prozentfeld) und sitzt nicht in dieser Zeile. | „Best frames (%)" | umbenennen | niedrig |
| 4.3 | „**Frames/Apoint**" | AlignLogic.swift:19 | Kryptisches Kürzel „Apoint". Bedeutung (jeder Punkt behält seine N besten Frames) nicht ableitbar. | „Best frames per point" + Tooltip „the reference program: Frames/Apoint" | beides | hoch |
| 4.4 | „**Best Frames**" (Anzahl) | AlignLogic.swift:20 | Gleich wie 4.2, nur ohne „(%)"; leicht zu verwechseln. | „Best frames (count)" bzw. „Number of best frames" | umbenennen | mittel |
| 4.5 | Felder ohne sichtbare Zuordnung (ein Prozentfeld bei Zeile 1, ein Zahlenfeld bei Zeile 3) | Views/AlignPanels.swift:151–154 | Einsteiger stellt in Zeile 2 um und sucht das Feld. | Feld je Modus in eigener Zeile oder Hinweis „uses the field above" | Struktur | mittel |
| 4.6 | Diagramm-Achse „**best frame** … **weakest**" | AlignPanels.swift:231–233 | Ungleich zu „weakest … strongest" beim Threshold: dasselbe Wort, andere Richtung und andere Bedeutung. | „sharpest … blurriest" | umbenennen | mittel |
| 4.7 | „Go to best frame" + Tooltip „Show the best frame and open Alignpoints to use it as **reference**" | AlignPanels.swift:206–209, FloatingCards.swift:82–85 | „reference" unerklärt (wie 2.7). | „… to line all frames up on it" | umbenennen | niedrig |
| 4.8 | Status „Processing limited to 8 frames" / „**Stacksize: 8  Low quality: 72.00%**" | AlignLogic.swift:213–215 | „Stacksize" zusammengeschrieben; „Low quality" klingt nach Warnung, ist aber die Qualität des schwächsten behaltenen Frames. Doppelleerzeichen; zwei Nachkommastellen. | „8 frames will be stacked · weakest kept: 72 %" | umbenennen | mittel |
| 4.9 | Barrierefreiheit „No frame is rated yet." / „Quality of n frames, best first. 8 are kept." | AlignLogic.swift:282, :339 | Gut. | — | — | — |

## Frame-Leiste, Framelist, Registrationgraph (Schritte 2–4)

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| F.1 | „**Frame (5): 5/10**" | Views/FrameBar.swift:64 | Doppelte Zahl, kryptisch. Die Bildbeschriftung sagt „Frame 5 of 10". | „Frame 5 of 10" | umbenennen | mittel |
| F.2 | „**Goto Frame**" | FrameBar.swift:33–34 | „Goto" ist Programmiersprache. | „Go to frame" | umbenennen | niedrig |
| F.3 | Status-Punkt „Loaded" / „Aligned" | FrameBar.swift:26 | Ok. | — | — | niedrig |
| F.4 | Karte „**Framelist**" | FloatingCards.swift:72 | Zusammengeschrieben. | „Frame List" (Show-Menü: „Show Frame List") | umbenennen | niedrig |
| F.5 | „Check all · Check none · **Empty List**" | FloatingCards.swift:75–77 | „Empty List" ist zweideutig (leer oder leeren?), tut nichts, ist aber **nicht gesperrt**; man erfährt es erst über die Statusleiste („Empty List: not available yet", AppModel+Align.swift:224). | Sperren + Tooltip „Not available yet"; Text „Clear List" | Verhalten + umbenennen | mittel |
| F.6 | Frame-Namen „Frame00010" | AlignLogic.swift:69 | Technisch, aber Original-getreu. | „Frame 10" | umbenennen | niedrig |
| F.7 | Karte „**Registrationgraph**" | FloatingCards.swift:147 | Fachwort, zusammengeschrieben. | „Quality Graph" + Tooltip „the reference program: Registration graph" | beides | mittel |
| F.8 | Legende „Quality" / „**Difference**" | FloatingCards.swift:154–155 | Was „Difference" misst (Verzerrung gegenüber Referenz), bleibt offen. | Tooltip „Difference: how much the frame is bent compared to the reference, higher = worse" | Tooltip | mittel |
| F.9 | „Run Align to see the graph" | FloatingCards.swift:174 | Gut. | — | — | — |

## Show-Menü am Bild (alle Schritte)

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| M.1 | „**Show Full Image**" (alle drei Varianten) | ImageArea.swift:112, :116, :121 | **Keine Wirkung** (show.fullImage / stackOptions.showFullImage / wavelet.showFullImage werden nirgends gelesen), aber nicht gesperrt. | Sperren + „Not available yet" | Verhalten | mittel |
| M.2 | „**Show Aligndata**", „**Show Prefilter**" | ImageArea.swift:124, :127 | Kryptisch und ohne Wirkung (nicht gelesen), nicht gesperrt. | Sperren + „Not available yet"; „Show Align Data" | Verhalten + umbenennen | mittel |
| M.3 | „**Show Stackgraph**", „**Show Stacklist**" | ImageArea.swift:118–119 | Zusammengeschrieben und ohne Wirkung (laut Doku „Not available yet"), im Menü aber nicht gesperrt. | Sperren; „Show Stack Graph", „Show Stack List" | Verhalten + umbenennen | mittel |
| M.4 | „**Show AlignSections**" öffnet Karte „**Alignmentpoints**" mit „Point 1…" | ImageArea.swift:117, FloatingCards.swift:119, :124 | Drei Namen für eine Sache; „AlignSections" mit Binnenmajuskel. Dass die Karte steuert, welche Punkte in den Stack gehen, ist nicht erkennbar. | Menü „Show Alignpoints", Karte „Alignpoints used for stacking" | umbenennen | mittel |
| M.5 | „Show **AlignPoints**" (Wavelets) vs. „Show **Alignpoints**" (Schritte 2–4) | ImageArea.swift:113 vs. :123 | Inkonsistente Großschreibung. | Einheitlich (siehe G.5) | umbenennen | niedrig |
| M.6 | „Show **ROI**" | ImageArea.swift:122 | Kürzel; siehe G.6. | „Show Region" | umbenennen | niedrig |
| M.7 | „Show **Processing Area**" | ImageArea.swift:114 | Unklar, dass das die Vorschau-Fläche ist. | „Show Preview Area" + Tooltip „the reference program: Processing Area" | beides | mittel |
| M.8 | Button „**Show ▾**" + Tooltip „What the image and the cards show" | ImageArea.swift:131, :144 | Ok. | — | — | niedrig |

## Zoomleiste und Bildbeschriftung

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| Z.1 | „**Before**" | ImageArea.swift:162 | Ein einzelnes Wort ohne Kontext. Dass es ein Umschalter (Vorher/Nachher) ist, sieht man nicht. | „Before/After" oder „Compare" (Tooltip: „Show the unsharpened stack") | umbenennen | mittel |
| Z.2 | „Zoom out (⌘-)", „Show at 100 % (⌘1)", „Fit the image into the area (⌘0)" | ImageArea.swift:156–160 | Gut. „100 %" mit Leerzeichen, der Prozentwert der Leiste ohne Leerzeichen (ViewTransform.swift:43); siehe G.8. | — | — | niedrig |
| Z.3 | Beschriftung „Frame 3 of 10" / „Stack of 8 frames" / „Sharpened stack" | AppModel.swift:132–134 | Gut, Vorbild für F.1. | — | — | — |

## 5 Stack

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 5.1 | „**StackSize**" + Radios „Default · Minimized · Maximized · **R.of Interest**" | Views/StackPanel.swift:12–16 | „StackSize" mit Binnenmajuskel. Gemeint ist die Größe des Ergebnisbilds; man könnte es für die Anzahl der Frames halten (vgl. Status „Stacksize: 8", 4.8, wo es genau das bedeutet!). „R.of Interest" ist eine 4. Schreibweise der ROI. | „Output size" mit „Default · Smallest common area · Largest area · Region" + Tooltip „the reference program: StackSize" | beides | hoch |
| 5.2 | „**Use Nearestby Alignpoints**" | StackPanel.swift:17 | „Nearestby" ist ein Tippfehler bzw. Kunstwort; die Zahl (3) bleibt unklar. | „Use nearest points" (Tooltip: „How many of the nearest alignpoints decide the shift of a pixel. More = smoother, fewer = more local. the reference program: Use Nearestby Alignpoints") | beides | hoch |
| 5.3 | „**Correct geometry**" | StackPanel.swift:19 | Unklar, was korrigiert wird. | Behalten + Tooltip „Shift each part of the image by its own movement; off moves the whole frame as one" | Tooltip | mittel |
| 5.4 | Gruppe „Stacking options" | StackPanel.swift:24 | Ok. | — | — | — |
| 5.5 | „**Normalization of frame intensity**" | StackPanel.swift:25 | Fachsprachlich. | „Even out frame brightness" + Tooltip „Helps when clouds pass by" | beides | mittel |
| 5.6 | „Stretch histogram after stacking", „De-rotate images" (gesperrt) | StackPanel.swift:26–27 | Fachbegriffe, aber gesperrt. | Tooltip ergänzen | Tooltip | niedrig |
| 5.7 | „**Do not stack pixels\nat the frameedge**" | StackPanel.swift:29, :31 | „frameedge" zusammengeschrieben; Einheit fehlt. | „Ignore edge pixels (px)" | umbenennen | mittel |
| 5.8 | „**Resampling**", „**Drizzling**" (gesperrt) | StackPanel.swift:35–36 | Fachbegriffe; gesperrt. | Tooltip „Enlarges the result; not available yet" | Tooltip | niedrig |
| 5.9 | Status „**Stack size (n=10) - (1024x680)**" / „**Stacking Range : 0-255**" | Model/StackLogic.swift:67 | Technisch. „Stack size" heißt hier Anzahl, in 5.1 Bildgröße. „x" statt „×", Leerzeichen vor dem Doppelpunkt, „Range" unklar. | „Stacked 10 frames · 1024×680" / „Value range 0–255" | umbenennen | mittel |
| 5.10 | Fortschritt „Stacking frame: 0" | AppModel+Stack.swift:15 | Ok. | „Stacking frame 3 of 8" | umbenennen | niedrig |
| 5.11 | Ergebnis „10 frames · 1024×680" | StackLogic.swift:72 | Gut. | — | — | — |

### 5b Create AVI (Stack › Advanced)

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 5b.1 | Gruppe „**Create AVI**" | Views/Save/SaveViews.swift:61 | Ok, aber es ist unklar, wozu man es braucht. | Tooltip „Export the aligned frames as a video, e.g. for animations or other software" | Tooltip | niedrig |
| 5b.2 | „**Codec**" (Picker DIB/cvid/i420/…) | SaveViews.swift:63–69, SaveLogic.swift:30 | **Keine Wirkung** (StackLogic.swift:21–22: Video wird immer unkomprimiert geschrieben). Die Kürzel sind kryptisch, der Picker ist trotzdem aktiv. | Sperren + „Not available yet (always uncompressed)" | Verhalten | hoch |
| 5b.3 | „**Compress Qual**" (7500) | SaveViews.swift:71 | Abgekürzt, Skala unbekannt, ohne Wirkung, nicht gesperrt. | Sperren; Text „Compression quality" | Verhalten + umbenennen | hoch |
| 5b.4 | „**Framerate**" | SaveViews.swift:72 | Einheit fehlt; zusammengeschrieben (Dateiinfo sagt „Frame rate"). | „Frame rate (fps)" | umbenennen | niedrig |
| 5b.5 | „Create aligned **S**equence" vs. „Create integrated **s**equence" | SaveViews.swift:74, :93 | Uneinheitliche Großschreibung. | „Aligned video" / „Averaged video" | umbenennen | niedrig |
| 5b.6 | „**Save Aligned**" / „**Save Integrated**" | SaveViews.swift:76, :95 | Öffnen einen Dialog, aber ohne „…". „Integrated" ist unverständlich. | „Save Aligned Video…" / „Save Averaged Video…" | umbenennen | mittel |
| 5b.7 | Radios „Original image size · **Minimum area** · **Maximum area** · Region of Interest" | Model/SaveLogic.swift:16–19 | Ohne Überschrift (laut Kommentar „Image size"). Nur „Original" wirkt (StackLogic.swift:21–22), die anderen sind aber wählbar. | Überschrift „Frame size"; nicht wirksame sperren | Verhalten + umbenennen | mittel |
| 5b.8 | „**Sort On Quality**" / „**Use Prefilters**" | SaveViews.swift:89–90 | „On" groß geschrieben; „Prefilters" unerklärt; Use Prefilters ohne Wirkung, aber nicht gesperrt. | „Sort by quality"; Use Prefilters sperren | Verhalten + umbenennen | mittel |
| 5b.9 | Tooltip „Write the frames that stay, moved onto the reference frame, as an AVI" | SaveViews.swift:79 | Verständlich. | „Save the kept frames, aligned, as an AVI video" | umbenennen | niedrig |
| 5b.10 | Tooltip „Write the moving average over **Average of n Frames** aligned frames as an AVI" | SaveViews.swift:98 | Grammatisch kaputt, weil ein Feldname eingebaut ist. | „Save a video where each frame is the average of the next n aligned frames" | umbenennen | mittel |
| 5b.11 | „**Moving average**" / „Compressed" (gesperrt) | SaveViews.swift:100–104 | Fachbegriff, für die Option aber angemessen. | Tooltip | Tooltip | niedrig |
| 5b.12 | „Average of [5] **Frames**" | SaveViews.swift:109–111 | „Frames" groß geschrieben. | „Average of 5 frames" | umbenennen | niedrig |
| 5b.13 | Status „Writing frame: 0" / „Saved x.avi (10 frames)" | AppModel+Save.swift:104, :113 | Ok. | — | — | — |

## 6 Wavelets (linkes Panel)

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 6.1 | „**LRGB**" | Views/Wavelets/WaveletsPanel.swift:22 | Kürzel aus der Mono-Kamera-Welt. Ein Einsteiger mit Farbkamera hält es für eine Kanalliste. | „Sharpen brightness only" + Tooltip „the reference program: LRGB – sharpen the luminance, keep the colour of the stack" | beides | hoch |
| 6.2 | „**Colour**" + Tooltip „Process the channels in colour; off gives **grayscale**" | WaveletsPanel.swift:17–20 | UK „Colour" neben US „grayscale" (siehe G.4). | „…off gives greyscale" (oder alles US) | umbenennen | niedrig |
| 6.3 | Tooltip „Needs a colour image" | WaveletsPanel.swift:20, :26, FunctionCards.swift:242, :284 | Gut. | — | — | — |
| 6.4 | Spalten „**Layer · Denoise · Sharpen · Preview**" | WaveletsPanel.swift:49–53 | Die Slider-Zeile hat keine Spaltenüberschrift; der eigentliche Hauptregler (Stärke) ist unbeschriftet. „Sharpen" ist ein Faktor, nicht die Stärke, und das ist irreführend. | Slider-Tooltip „Strength of layer n"; „Sharpen" → Tooltip „Scales how strongly the slider acts (0.100 = as shown)" | Tooltip | hoch |
| 6.5 | Gesperrter **„Preview"-Button zeigt den Sliderwert** (z. B. „10.0") | WaveletsPanel.swift:78–82 | Liest sich wie ein Wertefeld; Tooltip „Preview layer n: Not available yet". Ein Einsteiger versteht nicht, warum die Zahl nicht klickbar ist. | Wert als Text neben dem Slider; Preview-Button separat und gesperrt | Struktur | mittel |
| 6.6 | Layer-Nummern „1…6" ohne Größenangabe | WaveletsPanel.swift:65 | Unklar, dass 1 = feinstes Detail ist. | Kopfzeile „Layer (1 = finest)" oder Tooltip | Tooltip | mittel |
| 6.7 | „Reset Wavelets" | WaveletsPanel.swift:33 | Ok. | — | — | niedrig |
| 6.8 | „**Automatic**" | WaveletsPanel.swift:104 | Löst laut Doku nach 0,5 s selbst Do All aus. Am Namen ist das nicht zu erkennen. | „Run Do All automatically" | umbenennen | hoch |
| 6.9 | „**Hold Wavelet Setting**" | WaveletsPanel.swift:105 | Singular, unklar wann gehalten wird. | „Keep layer values for a new stack" | beides | mittel |
| 6.10 | „**Waveletscheme**" + „Dyadic (2^n) · Linear" | WaveletsPanel.swift:106–109 | Zusammengeschrieben; „Dyadic" ist Mathe-Jargon. | „Wavelet scheme"; Tooltip „Linear: layers grow evenly. Dyadic: each layer doubles in size" | beides | mittel |
| 6.11 | „**Initial Layer**" / „**Step Incr.**" | WaveletsPanel.swift:112–115 | Abkürzung „Incr."; Bedeutung (Größe der ersten Ebene / Zuwachs) unklar. | „First layer size" / „Size step" + Tooltip mit the reference program-Namen | beides | mittel |
| 6.12 | „Wavelet filter" „Default · Gaussian" | WaveletsPanel.swift:118–121 | „Default" sagt nichts über den Filter aus. | Tooltip „Gaussian: smoother; Default: the reference program's original filter" | Tooltip | niedrig |
| 6.13 | „**Use Linked Wavelets**" | WaveletsPanel.swift:123 | Unklar. | „Move all sliders together" + Tooltip „the reference program: Linked Wavelets" | beides | mittel |
| 6.14 | „Available schemes" (gesperrt, leerer Picker) | WaveletsPanel.swift:126–131 | „scheme" ist doppelt belegt: Wavelet-Schema (6.10) und gespeicherte Voreinstellung. | „Saved presets" | umbenennen | mittel |
| 6.15 | „**Load Scheme**" / „**Save Scheme**" | WaveletsPanel.swift:133–136 | Öffnen Dialoge, haben aber keine Ellipse (in Settings mit „…"). „Scheme" wie 6.14. | „Load Preset…" / „Save Preset…" (the reference program: Scheme) | umbenennen | mittel |
| 6.16 | Dialogtext „the reference program Wavelet scheme settings" | WaveletsPanel.swift:159 | Ok. | — | — | niedrig |
| 6.17 | „**Realign with Processed**" (gesperrt) / „**Stack Again**" (gesperrt, **ohne** „Not available yet") | WaveletsPanel.swift:143–151 | Stack Again hat keinen Hinweis und keinen Tooltip. | Stack Again: Tooltip „Not available yet" | Tooltip | niedrig |
| 6.18 | Fuß „**Process**" + Tooltip „Calculate the preview area with the current settings" | Views/StepPanel.swift:129–132 | „Process" ist generisch; dass es nur die Vorschau rechnet, steht nur im Tooltip. Kollidiert mit dem Menü „Process" (0.5). | „Update Preview" (Tooltip „the reference program: Process") | beides | hoch |
| 6.19 | Status „**layer setting changed**" | AppModel+Wavelets.swift:217 | Klein geschrieben, vage. | „Updating preview…" | umbenennen | niedrig |
| 6.20 | Status „**Preview: 85 ms**" / „**Calculating wavelets done**" | AppModel+Wavelets.swift:266–267 | „Calculating … done" ist holprig. | „Preview updated (85 ms)" | umbenennen | niedrig |
| 6.21 | Status „**Do_all processing**" | AppModel+Wavelets.swift:303 | Unterstrich (Delphi-Relikt). | „Applying to whole image…" | umbenennen | mittel |
| 6.22 | Status „Do All finished" / „Do All: the stack changed" | AppModel+Wavelets.swift:309, :317 | Ok; Letzteres bleibt kryptisch. | „Do All stopped: the stack changed, run it again" | umbenennen | niedrig |
| 6.23 | Status „Scheme loaded: **/full/path/…**" | AppModel+Wavelets.swift:350, :359 | Voller Pfad in schmalem Feld. | Nur Dateiname | umbenennen | niedrig |
| 6.24 | Fehler „Error: no stacked image" | AppModel+Wavelets.swift:247, :282 | Ok. | „Stack first" | umbenennen | niedrig |

## 6b Functions-Panel (rechts) und Karten

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 6b.1 | „**Functions »**" | ImageArea.swift:96 | Sagt nicht, was drin ist (Farbe/Tonwert). „»" deutet eine Navigation an. | „Adjust ›" oder „Colour & Tone" (Tooltip „the reference program: Functions") | beides | mittel |
| 6b.2 | 15 Buttons, davon **11 ohne Funktion, aber klickbar** (Colour Mixing, View Zoomed, View Compare, View Stacksize, Flip and Rotate, Resize Image, Denoise/ Deringing, Wavelet Filter, Masking, Show Linegraph, Cropping Area) | Model/WaveletLogic.swift:153–157, FunctionsPanel.swift:128 | Sehen genauso aus wie die funktionierenden. Erst der Klick meldet „…: not available yet" in der Statusleiste (AppModel+Wavelets.swift:376). | Optisch dimmen (wie LockedOption) | Verhalten | hoch |
| 6b.3 | „**Denoise/ Deringing**" | WaveletLogic.swift:156 | Leerzeichen nach „/". | „Denoise/Deringing" | umbenennen | niedrig |
| 6b.4 | „**View Stacksize**", „**Show Linegraph**" | WaveletLogic.swift:154, :157 | Zusammengeschrieben. | „View Stack Size", „Show Line Graph" | umbenennen | niedrig |
| 6b.5 | „**Wavelet Filter**" (Functions) vs. „Wavelet filter" (Advanced) | WaveletLogic.swift:156, WaveletsPanel.swift:118 | Gleicher Name für zwei verschiedene Dinge. | Tooltip am Function-Button | Tooltip | niedrig |
| 6b.6 | „Contrast/Brightness" mit Links „**Hold**" · „Reset" | FunctionsPanel.swift:49–52 | „Hold" ohne Erklärung; Umschalter als Textlink. | Tooltip „Keep these values for the next image" | Tooltip | mittel |
| 6b.7 | Zwei Slider ohne eigene Beschriftung, darunter die Felder Contrast/Brightness | FunctionsPanel.swift:54–80 | Welcher Slider was regelt, erkennt man nur an der Reihenfolge. | Label direkt über jedem Slider | Struktur | mittel |
| 6b.8 | „**Copy To** · **Load to** · Difference" | FunctionsPanel.swift:91–93 | „Copy To" ohne Objekt; „Load to" mit anderer Großschreibung; Ziel ist ein interner Zwischenspeicher, nicht die Systemzwischenablage. | „Remember · Restore · Difference" oder „Copy Image · Paste Image · Show Difference" + Tooltip „the reference program: Copy To / Load to" | beides | hoch |
| 6b.9 | „**Toggle**" + „Current Image · **Clipboard Image**" | FunctionsPanel.swift:95–98 | „Toggle" als Überschrift ist ein Verb ohne Inhalt; „Clipboard" suggeriert die macOS-Zwischenablage. | „Show:" · „Current" · „Copied" | umbenennen | mittel |
| 6b.10 | Status „Image copied", „Load to: nothing copied yet", „Difference: the images differ in size" | AppModel+Wavelets.swift:530–565 | Ok, sobald 6b.8 umbenannt ist, mitziehen. | — | umbenennen | niedrig |
| 6b.11 | Histogramm: „**Input · Result**", „Black · White", „**Auto**", „**Clip %**" | Views/Wavelets/FunctionCards.swift:61–79 | „Clip %" unklar (Anteil der Pixel, die bei Auto abgeschnitten werden). | „Clip %" + Tooltip „Share of darkest/brightest pixels Auto may clip"; „Black point"/„White point" | beides | mittel |
| 6b.12 | Histogramm-Tooltip „Drag the triangles to set the black and white point" | FunctionCards.swift:134 | Gut. | — | — | — |
| 6b.13 | Gamma „darker · brighter" | FunctionCards.swift:182–184 | Gut. | — | — | — |
| 6b.14 | RGB Align: „X · Y", „**Max shift**", „Auto moves Red and Blue onto Green" | FunctionCards.swift:222–237 | Einheit (px) fehlt bei X/Y und Max shift. | „Max shift (px)", Kopf „X (px) · Y (px)" | umbenennen | niedrig |
| 6b.15 | Status „RGB Align: shifts found" | AppModel+Wavelets.swift:499 | Ok. | — | — | — |
| 6b.16 | RGB Balance „Red/Green/Blue" ohne „%" | FunctionCards.swift:277–298 | Einheit fehlt sichtbar (Feld zeigt „100"). | „%" hinter das Feld | umbenennen | niedrig |

## 7 Save

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| 7.1 | „The sharpened stack with the colour and tone settings." | Views/Save/SaveViews.swift:14 | Ohne Verb. | „Saves the sharpened stack with your colour and tone settings." | umbenennen | niedrig |
| 7.2 | „**Do All** runs first with the current values." | SaveViews.swift:18 | Setzt Wissen über Do All voraus. | „Your latest changes are applied to the whole image first (Do All)." | umbenennen | mittel |
| 7.3 | Menü „Save image ▾" → Einträge „**‹Label›  .ext**" | SaveViews.swift:36, :42 | Doppelleerzeichen; die Einträge öffnen einen Dialog, haben aber kein „…". | „TIFF 16-bit (.tif)…" | umbenennen | niedrig |
| 7.4 | Tooltip „Save the image as a file; choose the file type first" | SaveViews.swift:49 | Ok. | — | — | — |
| 7.5 | Status „Save image: ‹error›" / „Save cancelled" / „Saved x.tif" | AppModel+Save.swift:64–83 | Ok. | — | — | — |
| 7.6 | Status „Save project: no recording is open" / „Open project: the project has no recording" / „Open project: file not found: …" | AppModel+Save.swift:145–185 | Ok. „file not found" erklärt nicht, dass die Aufnahme verschoben wurde. | „Open project: the recording was not found at ‹path›" | umbenennen | niedrig |
| 7.7 | Projekt-Dialog „Open a project: the recording and the settings of the steps." | Model/SavePanels.swift:29 | Ok. | — | — | — |

## Statusleiste allgemein

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| T.1 | „Time used 00:00:12 · Time left 00:01:05" (immer sichtbar, auch bei 0) | Views/StatusBar.swift:28 | Steht im Leerlauf als „00:00:00" da. | Nur während einer Rechnung zeigen | Verhalten | niedrig |
| T.2 | „**Error: ‹Kerntext›**" | AppModel.swift:177, :258 u. a. | Kernmeldungen sind englisch, klein geschrieben und technisch (z. B. SessionBackend.swift:34 „the frame could not be turned into an image"). | Präfix je Schritt („Align failed: …") | umbenennen | mittel |
| T.3 | „Cancelled" / „Save cancelled" | AppModel+Align.swift:33 | Ok. | — | — | — |

## Settings (⌘,)

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| E.1 | Label „**CPUs**", Text „The number of **threads** …" | Settings/SettingsView.swift:18, :29 | Label und Erklärung sprechen von verschiedenen Dingen. | „CPU cores to use" | umbenennen | niedrig |
| E.2 | „**Processing Area**" „512 **x** 512" + „The square in the middle of the image the live preview of the Wavelets step calculates." | SettingsView.swift:40–43 | „x" statt „×"; der Satz ist verschachtelt. | „Preview area" (Tooltip the reference program-Name); „512 × 512 px"; „Size of the square in the middle of the image that the Wavelets preview calculates. Smaller = faster." | beides | mittel |
| E.3 | „Schemes" + „Load Scheme… / Save Scheme…" + „… Initial Layer, **Step Incr.** and the six layers …" | SettingsView.swift:47–54 | Kürzel „Step Incr."; siehe 6.14/6.15. | Mitziehen | umbenennen | niedrig |

## Help-Fenster und About

| # | Aktuell | Ort | Problem | Vorschlag | Art | Schwere |
|---|---|---|---|---|---|---|
| H.1 | „Search the documentation", „No page matches." | Help/HelpView.swift:38, :49 | Gut. | — | — | — |
| H.2 | Ladefehler „The documentation is not part of this build." | Help/HelpModel.swift:25 | Ok, Entwicklerfall. | — | — | niedrig |
| H.3 | About: „A rebuild of the workflow of the reference program for stacking and sharpening planetary and lunar recordings. Not affiliated with the original." | Commands/HelpCommands.swift:21–22 | Gut. | — | — | — |
| H.4 | Kein Glossar | — | Bei „behalten + Tooltip" fehlt eine zentrale Stelle, an der die the reference program-Begriffe erklärt sind. | Doku-Seite „Glossary" (Alignpoint, ROI, Limit, Stack, Wavelet layer, Do All, LRGB, Processing Area, Registration graph) und verlinken | neu | mittel |

---

## Globale Vorschläge

| # | Thema | Befund | Vorschlag | Schwere |
|---|---|---|---|---|
| G.1 | **Groß-/Kleinschreibung** | Gemischt: „Save image" / „Save Image…", „Sort On Quality", „Go to best frame", „Check all", „Create aligned Sequence", „Load to" / „Copy To", „Frames". | macOS HIG: **Title Case** für Buttons, Menüeinträge, Gruppenüberschriften und Show-Toggles („Go to Best Frame", „Check All"); **Sentence case** für Feldbeschriftungen, Häkchen, Radios, Hinweise und Tooltips. Einmal festlegen und in docs/design.md notieren. | mittel |
| G.2 | **Ellipse „…"** | Fehlt bei Aktionen mit Dialog: Load Scheme / Save Scheme (Panel), Save Aligned, Save Integrated, R.o.Interest (startet einen Zeichenmodus, kein Dialog, also ohne), die Formateinträge von Save image. Vorhanden bei Select Recording…, Open…, Save Image… (Menü), Settings-Buttons. | HIG: „…" immer dann, wenn vor der Ausführung weitere Eingaben (Dialog) kommen. | niedrig |
| G.3 | **Zusammengeschriebene Kunstwörter** (Delphi-Erbe) | Alignpoints, Alignmentbox, Alignmentpoints, AlignSections, Aligndata, Framelist, Registrationgraph, Stackgraph, Stacklist, StackSize, Stacksize, Waveletscheme, frameedge, pixelvalue, Linegraph, Framerate, ScanFrame, Nearestby, Apoint, Do_all. | Getrennt schreiben. „Alignpoints" als bewusst behaltenen Markenbegriff überall gleich schreiben (siehe G.5). | mittel |
| G.4 | **UK/US-Englisch gemischt** | „Colour", „Centre" (UK) neben „grayscale" (US); macOS-Systemmenüs sind US. | Entscheiden. Empfehlung: **US** (Color, Center, gray), passend zum System. Alternativ konsequent UK inkl. „greyscale". | niedrig |
| G.5 | **Ein Begriff = eine Schreibweise** | Alignpoints / AlignPoints / Alignpoint / alignpoints / Alignmentpoints / Apoint / AlignSections / „Point n". | Eine Form festlegen, z. B. „alignpoint(s)" im Satz, „Alignpoints" im Titel. | mittel |
| G.6 | **ROI in vier Schreibweisen** | R.o.Interest (2.5), R.of Interest (5.1), ROI (M.6), Region of Interest (5b.7). | Überall „Region" (kurz) mit Tooltip „Region of Interest (ROI)". | hoch |
| G.7 | **Einheiten fehlen** | Abstände, Box-Größe, Max movement, Edge pixels, Max shift, RGB-Shift (px); Frame rate (fps); RGB Balance (%); Compress Qual (Skala unbekannt). | Einheit in Klammern ans Label oder als Suffix hinters Feld. | mittel |
| G.8 | **Zahlformat** | „100 %" (Tooltip, Limit-Diagramm) vs. „100%" (Zoom, Statusleiste, Framelist); „1024x680" vs. „1024×680"; „72.00%" mit 2, Framelist mit 1 Nachkommastelle. | Einheitlich: „×" für Maße; „%" ohne Leerzeichen (US) oder mit, aber gleich; Qualität mit 0–1 Nachkommastelle. | niedrig |
| G.9 | **Bedienelemente ohne Wirkung, aber aktiv** | Show Full Image (3×), Show Aligndata, Show Prefilter, Show Stackgraph, Show Stacklist, Show Alignment, Codec, Compress Qual, Use Prefilters, Frame size außer Original, Empty List, 11 Functions-Buttons. | Gleich behandeln wie LockedOption: gedimmt, Tooltip „Not available yet". Für Einsteiger ist ein Schalter ohne Wirkung schlimmer als ein unbekanntes Wort. | hoch |
| G.10 | **Tooltip-Abdeckung** | Felder in Align/Stack/Wavelets haben fast keine Tooltips (nur R.o.Interest, Colour, LRGB, Process, Save). | Jedes Feld mit Fachbegriff bekommt einen Ein-Satz-Tooltip. Wo umbenannt wird, schließt er mit „the reference program: ‹Originalname›", damit Tutorials auffindbar bleiben. Optional ein „?"-Link je Gruppe auf die Doku-Seite. | hoch |
| G.11 | **Statustexte** | Mal mit Punkt, mal ohne; mal klein („layer setting changed"); Delphi-Relikte („Do_all", „Stacking Range : 0-255", „Stack size (n=10) - (…)"). | Stil: Großer Anfangsbuchstabe, kein Punkt, ein kurzer Satz, Einheiten mit „×"/„%". | mittel |
| G.12 | **„reference"** | Wird in Hinweis/Tooltip zu Best frame benutzt, aber nie erklärt. | Einmal erklären („the frame all others are lined up on") oder umschreiben. | mittel |
| G.13 | **Doku nachziehen** | `app/src/docs/pages/*.md` zitiert die heutigen Begriffe wörtlich (z. B. image-area.md:16–18, step-stack.md:56). | Jede Umbenennung im selben Commit in den Doku-Seiten und in docs/ui.md (Spalte „Mac-Text" neben „the reference program-Begriff") nachziehen. | mittel |

---

## Zählung

| Schwere | Anzahl |
|---|---|
| hoch | 24 |
| mittel | 64 |
| niedrig | 77 |
| **gesamt** (ohne die 19 „—"-Zeilen = in Ordnung) | **165** |

Davon global: G.6, G.9, G.10 hoch · G.1, G.3, G.5, G.7, G.11, G.12, G.13 mittel · G.2, G.4, G.8 niedrig.

---

## Umsetzung und Abweichungen (2026-11-12)

Alle hohen und mittleren Befunde sind umgesetzt, mit diesen Abweichungen von den Vorschlägen:

- **Kein „the reference program: …“-Tooltip-Zusatz** (Philipp): überall weggelassen (G.10, 3.2, 4.1, 4.3, 5.1, 5.2, 6.1, 6.11, 6.13, 6.15, 6b.8, E.2 u. a.).
- **Schritt 4 „Limit“ → „Best Frames“, Knopf „Keep Best Frames“** (spätere Entscheidung): S.2, S.7, 4.1–4.4 und 4.8 folgen dem, nicht dem Vorschlag
  „Min. quality (%)“ usw.; Modi „At least % of the best quality“, „Best % of frames“, „Best number of frames“, „Best frames per point“.
- **0.2**: Menü File heißt „Select Recording…“ wie die Werkzeugleiste (statt „Open Recording…“), damit eine Aktion einen Namen hat.
- **0.5**: Menü „Process“ heißt „Run“.
- **S.18**: Ergebniszeile „82 points“ statt „82 alignpoints“ (die Pille nennt „Alignpoints“ schon, sonst steht das Wort doppelt).
- **2.1**: „more points · fewer points“ stehen unter dem Regler, der Titel „Threshold · 78 points“ darüber (in einer Zeile fehlte die Breite).
- **2.14**: Fehlertext „lower the distances or widen Min./Max. brightness under Advanced“ statt „move the Threshold slider left“: bei „keine Punkte
  gefunden“ ist der Regler noch gesperrt und der Threshold kann nichts bewirken.
- **2.10/2.11**: „Intensity selection“ → „Brightness measure“; Lo/Hi → „Min. brightness (0–255)“ / „Max. brightness (0–255)“ als eigene Zeilen.
- **3.1**: die gesperrten Segmente „No Align · Scan Frames“ stehen jetzt in Advanced („Alignment mode“, 5 statt 4 weitere Einstellungen).
- **6.10/6.14/6.15**: „Waveletscheme“ (Dyadic/Linear) heißt „Layer spacing“, damit „scheme“ nur die Datei meint (Dialog „Wavelet scheme“, Philipp);
  „Available schemes“ → „Saved Schemes“, die Knöpfe bleiben „Load Scheme…“ / „Save Scheme…“ statt „Preset“.
- **6.5**: der gesperrte Preview-Knopf zeigt ein Auge-Symbol, der Sliderwert steht als Text rechts neben dem Regler.
- **6b.1**: „Functions »“ → „Adjust ›“, Panel „Adjustments“. **6b.2**: die elf Knöpfe ohne Karte sind gesperrt. **6b.8/6b.9**: „Remember · Restore ·
  Difference“, „Show: Current image · Remembered image“. **6b.6**: Link „Hold“ → „Keep“, gesperrt, weil er keine Wirkung hat.
- **5b.2/5b.3/5b.8**: Codec, Compression quality, die Frame-size-Radios außer Original size und Use prefilters sind gesperrt.
- **7.3**: Dateityp-Einträge „TIFF (16 bit) · .tif…“ (Mittelpunkt statt Doppelleerzeichen, weil der Name schon eine Klammer hat).
- **T.2**: Präfix je Schritt („Align failed: …“); der Kerntext selbst bleibt unverändert (klein geschrieben).
- **F.5**: „Clear List“ gesperrt, die Funktion `emptyList()` samt Test entfernt.

Bewusst nicht umgesetzt:

- **0.12** (Absturztext, für Nutzer unsichtbar), **S.12, S.15, 2.8, 2.12, 2.16, 3.4, 4.9, 6.3, 6.7, 6.16, 7.4, 7.5, 7.7, 8.x** (in Ordnung oder kosmetisch ohne Nutzen).
- **4.5** (Feld je Modus in eigener Zeile): Das Feld steht schon in der Zeile seiner Modi und ist beim anderen Modus gesperrt; ein Umbau der Zeilen
  brächte keine Klarheit, der Tooltip der Radios nennt das Feld.
- **5.10** („Stacking frame 3 of 8“): niedrig, betrifft weder Tippfehler noch Schreibweise; Zähler bleiben „Aligning frame: 10“ usw.
- **T.1** (Zeit nur während einer Rechnung zeigen): niedrig; die zuletzt benötigte Zeit nach dem Lauf ist nützlich.
- **S.11** („Run Do All first“): bleibt, die Sperr-Tooltips nennen den Knopf (jetzt „Run Keep Best Frames first“).
- **6.6** („Layer (1 = finest)“): als Tooltip der Spaltenüberschrift „Layer“ umgesetzt, nicht als Text der Überschrift (Platz).

