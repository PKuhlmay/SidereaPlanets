# MAC-005 App-Gerüst, Fenster, Bildbereich und Schritt Open

- Status: erledigt
- Abhängig von: MAC-004
- Teil von: MAC-001, Kriterien „Fenster …“, „Schritt Open“, „Bildbereich“

## Ziel

Die SwiftUI-App steht und deployt: `mac/project.yml` (xcodegen, macOS 14+), Hauptfenster nach `docs/design.md` mit Werkzeugleiste,
Schrittleiste (Open · Alignpoints · Align · Limit · Stack · Wavelets · Save), Schrittpanel, Bildbereich, Frame-Leiste, Statusleiste;
Farben und Abstände als Swift-Tokens aus `docs/design/tokens.css` (dunkel und hell); Bedienelemente nach Mac-Art.

## Akzeptanzkriterien

- [x] `mac/project.yml`, `mac/App/…`, Bau per `xcodegen generate` + `xcodebuild`; `scripts/deploy-mac.sh` startet die App
- [x] Tokens aus `docs/design/tokens.css` als Swift-Farben/Abstände, hell und dunkel, folgen der macOS-Darstellung
- [x] Fensteraufbau und Schrittzustände wie in `docs/design.md` (gesperrte Schritte, aktiver Schritt, erledigte Schritte)
- [x] Open: Aufnahme öffnen (SER, AVI, Bildserien, FITS) per nativem Dialog und ⌘O, MRU (Menü „Open Recent“), Dateiangaben im Panel
- [x] Bildbereich: Frame anzeigen (CGImage), Zoom um den Zeiger (Trackpad, Mausrad), Pan, 100 %-Klick, Einpassen bei Schrittwechsel;
      Frame-Leiste blättert durch die Frames
- [x] Statusleiste mit den Angaben aus `docs/ui.md`
- [x] Bewusste Mac-Abweichungen in `docs/design.md` eingetragen

## Umsetzung

- Gerüst, Dateien und Entscheidungen: `mac/README.md` („Aufbau“, „Bauen und veröffentlichen“); Abweichungen vom Entwurf:
  `docs/design.md`, Abschnitt „Mac-App: bewusste Abweichungen“.
- Geprüft am laufenden Fenster mit JUPITER.AVI (dunkel und hell, Schrittzustände Open und Alignpoints, Zoom über Menü, Scrollen mit ⌥
  und Ziehen, Frames blättern); Screenshots gegen `docs/design/screenshots/01-open.png`, `02-alignpoints.png`, `15-limit-light.png`.
  Nicht am Gerät geprüft (Handprobe, `docs/parity/Handprobe.md`): Pinch und Zwei-Finger-Doppeltipp, SER, Bildserien und FITS über die
  Oberfläche, ⌘-Kürzel per Tastatur.
- Schritte ab Alignpoints zeigen nur ihr Panel mit „Comes in a later version“ und gesperrter Aktion; MAC-006 bis MAC-010 ersetzen das
  je Schritt. Cancel und Pause in der Werkzeugleiste sind gesperrt, bis MAC-006 Align anschließt.
- Bewusst nicht dabei: die Menüs Flat/Dark/Reference, Tools und Settings, Help, About und CPUs der Werkzeugleiste (MAC-010), Open und
  Save project (MAC-009), Show-Menü und schwebende Karten (MAC-006 ff.).
