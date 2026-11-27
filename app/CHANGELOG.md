# Changelog — Siderea Planets

## Unreleased

### English

- The app has a new internal identifier (de.kuhlmay.siderea.planets). Once after this update the list of recent recordings, the theme and the panels you had opened start over; your recordings, project files and results are not affected.
- Mac: the app has a menu bar (File, Edit, View, Run, Window, Help) and uses Cmd instead of Ctrl for its shortcuts: Cmd+O select a recording, Cmd+S save the image, Cmd+. cancel, Cmd+, settings, Option+Cmd+1 to 7 open a step. The window follows the light or dark appearance of macOS, a double-click on a project file in Finder opens it, and a pinch on the trackpad zooms the image.
- Windows and Linux: Ctrl+O, Ctrl+S, Ctrl+. and Ctrl+, do the same; Ctrl+plus, Ctrl+minus, Ctrl+0 and Ctrl+1 zoom the image; Alt+1 to Alt+7 still open the steps.
- Siderea Planets is now free software under the GPL-3.0-or-later license. About shows the version, the license and a link to the source code; Third-party licenses lists the licenses of all components the program contains.
- Project files now end in .sidereaplanets. To open a project saved with an earlier build, rename its file to this ending.
- In the installed app a right-click no longer opens the web view's browser menu (Reload, Inspect) and the keys that reload the page are switched off: F5, Ctrl+R and Ctrl+Shift+R, on the Mac Cmd+R and Cmd+Shift+R. Text fields keep their menu for Copy and Paste.
- Updates: the app looks for a new version at the start, at most once a day (Settings > Check for updates automatically), and on request with Check for Updates… (Mac: menu Siderea Planets; Windows and Linux: Settings menu). A dialog shows what is new; Install and Restart, Later or Skip This Version. The install waits while a calculation runs. The Linux .deb package does not update itself, the AppImage does.
- Wavelets: the live preview does less work while a slider moves, which should help most on Windows. It now redraws only the preview area instead of the whole image, and for a recording without colour (or with Color off) it sends a quarter of the data.

### Deutsch

- Die App hat eine neue interne Kennung (de.kuhlmay.siderea.planets). Einmalig nach diesem Update beginnen die Liste der zuletzt geöffneten Aufnahmen, das Thema und die geöffneten Panels von vorn; Aufnahmen, Projektdateien und Ergebnisse sind nicht betroffen.
- Mac: Die App hat eine Menüleiste (Ablage, Bearbeiten, Darstellung, Run, Fenster, Hilfe) und nutzt Cmd statt Ctrl: Cmd+O Aufnahme wählen, Cmd+S Bild speichern, Cmd+. abbrechen, Cmd+, Einstellungen, Option+Cmd+1 bis 7 öffnen einen Schritt. Das Fenster folgt dem hellen oder dunklen Erscheinungsbild von macOS, ein Doppelklick auf eine Projektdatei im Finder öffnet sie, und ein Pinch auf dem Trackpad zoomt das Bild.
- Windows und Linux: Strg+O, Strg+S, Strg+. und Strg+, tun dasselbe; Strg+Plus, Strg+Minus, Strg+0 und Strg+1 zoomen das Bild; Alt+1 bis Alt+7 öffnen weiter die Schritte.
- Siderea Planets ist jetzt freie Software unter der Lizenz GPL-3.0-or-later. About zeigt die Version, die Lizenz und einen Link zum Quellcode; Third-party licenses führt die Lizenzen aller enthaltenen Komponenten auf.
- Projektdateien enden jetzt auf .sidereaplanets. Ein mit einem früheren Stand gespeichertes Projekt öffnet sich, nachdem die Datei auf diese Endung umbenannt wurde.
- In der installierten App öffnet ein Rechtsklick nicht mehr das Browsermenü der Webansicht (Neu laden, Untersuchen), und die Tasten, die die Seite neu laden, sind abgeschaltet: F5, Strg+R und Strg+Umschalt+R, am Mac Cmd+R und Cmd+Umschalt+R. Textfelder behalten ihr Menü für Kopieren und Einfügen.
- Updates: Die App sucht beim Start nach einer neuen Version, höchstens einmal am Tag (Settings > Check for updates automatically), und auf Wunsch mit Check for Updates… (Mac: Menü Siderea Planets; Windows und Linux: Menü Settings). Ein Dialog zeigt, was neu ist; Install and Restart, Later oder Skip This Version. Während einer Berechnung ist das Installieren gesperrt. Das Linux-Paket .deb aktualisiert sich nicht selbst, das AppImage schon.
- Wavelets: Die Live-Vorschau läuft beim Ziehen eines Reglers flüssiger, vor allem unter Windows. Sie zeichnet nur noch den Vorschaubereich statt des ganzen Bildes neu, und bei einer Aufnahme ohne Farbe (oder mit Color aus) schickt sie ein Viertel der Daten.

## 0.1.0 — 2026-11-13

### English

- First version: stacking and sharpening of planetary and lunar recordings in seven steps — Open, Alignpoints, Align, Best Frames, Stack, Wavelets and Save.
- Opens SER and AVI videos and series of TIFF, PNG, BMP, JPEG and FITS images; drop a recording onto the window or pick it under Recent Recordings. The step Open lists format, codec, size, bit depth, frames, color and frame rate.
- Set Alignpoints places the marks automatically; Draw Region limits them to a part of the image. Align can follow the planet by its centre of gravity (Align by center of gravity), however far it jumps between frames.
- Best Frames keeps the sharpest frames by quality, share or count, shows the quality of all frames and takes you to the sharpest one with Go to Best Frame.
- Stack with Correct geometry, Even out frame brightness and De-rotate images for Jupiter, Saturn and Mars; Save Aligned Video… and Save Averaged Video… write AVI videos.
- Wavelets with six layers, a live preview and Do All; Load Scheme… and Save Scheme… read and write wavelet scheme files (.rwv).
- Adjustments: Histogram, Gamma, Color Mixing, RGB Align, RGB Balance, Denoise/Deringing, Masking, View Compare (before and after at a divider), Crop (Free, 1:1, 3:2, 4:3), Flip and Rotate and Resize Image; Remember, Restore and Difference compare two versions.
- Save Image writes TIFF, PNG, JPEG, BMP and FITS; Open Project… and Save Project… keep a session.
- Clear names and a tooltip on every control; controls that do not work yet are dimmed and say "Not available yet".
- Keyboard: left and right arrow keys page through the frames, Alt+1 to Alt+7 open the steps.
- The status bar shows the CPU load and the memory of the app (Settings > Show CPU and Memory Usage) and the version.
- Help with a page for every step and a glossary; dark and light theme; CPU cores to use in the toolbar.
- Installer for the current user or for all users; it installs the Microsoft Edge WebView2 runtime where it is missing.

### Deutsch

- Erste Version: Stacken und Schärfen von Planeten- und Mondaufnahmen in sieben Schritten — Open, Alignpoints, Align, Best Frames, Stack, Wavelets und Save.
- Öffnet SER- und AVI-Videos sowie Bildserien aus TIFF, PNG, BMP, JPEG und FITS; eine Aufnahme aufs Fenster ziehen oder unter Recent Recordings wählen. Der Schritt Open zeigt Format, Codec, Größe, Bittiefe, Frames, Farbe und Bildrate.
- Set Alignpoints setzt die Marken selbst; Draw Region beschränkt sie auf einen Teil des Bildes. Align kann dem Planeten über seinen Helligkeitsschwerpunkt folgen (Align by center of gravity), wie weit er auch springt.
- Best Frames behält die schärfsten Frames nach Qualität, Anteil oder Anzahl, zeigt die Qualität aller Frames und springt mit Go to Best Frame zum schärfsten.
- Stack mit Correct geometry, Even out frame brightness und De-rotate images für Jupiter, Saturn und Mars; Save Aligned Video… und Save Averaged Video… schreiben AVI-Videos.
- Wavelets mit sechs Ebenen, Live-Vorschau und Do All; Load Scheme… und Save Scheme… lesen und schreiben Wavelet-Schemata (.rwv).
- Adjustments: Histogram, Gamma, Color Mixing, RGB Align, RGB Balance, Denoise/Deringing, Masking, View Compare (vorher und nachher an einer Trennlinie), Crop (Free, 1:1, 3:2, 4:3), Flip and Rotate und Resize Image; Remember, Restore und Difference vergleichen zwei Stände.
- Save Image schreibt TIFF, PNG, JPEG, BMP und FITS; Open Project… und Save Project… sichern eine Sitzung.
- Klare Namen und ein Tooltip an jedem Bedienelement; was noch nicht funktioniert, ist gedimmt und sagt „Not available yet“.
- Tastatur: Pfeil links und rechts blättern durch die Frames, Alt+1 bis Alt+7 öffnen die Schritte.
- Die Statusleiste zeigt CPU-Last und Speicher der App (Settings > Show CPU and Memory Usage) und die Version.
- Hilfe mit einer Seite je Schritt und einem Glossar; dunkles und helles Design; CPU cores to use in der Werkzeugleiste.
- Installer für den eigenen Benutzer oder für alle; er installiert die Microsoft-Edge-WebView2-Laufzeit, wo sie fehlt.
