# MAC-005 App-Gerüst, Fenster, Bildbereich und Schritt Open

- Status: offen
- Abhängig von: MAC-004
- Teil von: MAC-001, Kriterien „Fenster …“, „Schritt Open“, „Bildbereich“

## Ziel

Die SwiftUI-App steht und deployt: `mac/project.yml` (xcodegen, macOS 14+), Hauptfenster nach `docs/design.md` mit Werkzeugleiste,
Schrittleiste (Open · Alignpoints · Align · Limit · Stack · Wavelets · Save), Schrittpanel, Bildbereich, Frame-Leiste, Statusleiste;
Farben und Abstände als Swift-Tokens aus `docs/design/tokens.css` (dunkel und hell); Bedienelemente nach Mac-Art.

## Akzeptanzkriterien

- [ ] `mac/project.yml`, `mac/App/…`, Bau per `xcodegen generate` + `xcodebuild`; `scripts/deploy-mac.sh` startet die App
- [ ] Tokens aus `docs/design/tokens.css` als Swift-Farben/Abstände, hell und dunkel, folgen der macOS-Darstellung
- [ ] Fensteraufbau und Schrittzustände wie in `docs/design.md` (gesperrte Schritte, aktiver Schritt, erledigte Schritte)
- [ ] Open: Aufnahme öffnen (SER, AVI, Bildserien, FITS) per nativem Dialog und ⌘O, MRU (Menü „Open Recent“), Dateiangaben im Panel
- [ ] Bildbereich: Frame anzeigen (CGImage), Zoom um den Zeiger (Trackpad, Mausrad), Pan, 100 %-Klick, Einpassen bei Schrittwechsel;
      Frame-Leiste blättert durch die Frames
- [ ] Statusleiste mit den Angaben aus `docs/ui.md`
- [ ] Bewusste Mac-Abweichungen in `docs/design.md` eingetragen
