# MAC-012 App-Icon für Mac und Windows

- Status: erledigt
- Abhängig von: MAC-005
- Teil von: MAC-001 (Aussehen); Familie mit Siderea

## Ziel

Ein eigenes App-Icon für SidereaPlanets im Stil von Siderea (gleicher Hintergrund, drei gestapelte Rahmen mit Orange-Kontur), das
das Thema zeigt: ein Planet, links unscharf, rechts scharf mit Bändern, dazwischen eine dünne Trennlinie, dahinter ein oranger
Schein („unscharf → scharf“). Dieselbe Gestaltung für die Mac-App (Icon-Composer-Dokument) und die Windows-App (Tauri-Icon-Satz),
die das Tauri-Standardlogo ersetzt. Der Debug-Build der Mac-App trägt ein DEV-Abzeichen.

## Akzeptanzkriterien

- [x] `docs/design/icon/AppIcon.icon` im Icon-Composer-Format wie bei Siderea: Hintergrundverlauf, Ebenengruppen mit Schatten,
      Ebenen `frame-back`, `frame-middle`, `frame-front` und das Planetenmotiv (Schein, unscharfer Planet, scharfe Hälfte mit
      Bändern und Trennlinie) auf dem 824-Raster
- [x] `docs/design/icon/AppIconDev.icon`: dieselben Ebenen plus DEV-Abzeichen in der Akzentfarbe
- [x] `mac/project.yml` nimmt beide `.icon`-Ordner als Ressourcen auf, `ASSETCATALOG_COMPILER_APPICON_NAME` ist `AppIcon`, im
      Debug `AppIconDev`; das gebaute Bündel enthält das Icon (`Assets.car`, `AppIconDev.icns`, `CFBundleIconName`)
- [x] Motiv ist bei 128 px klar lesbar; bei 32 und 16 px bleiben Rahmen und Planet erkennbar (Trennlinie und Bänder etwas kräftiger
      als im Entwurf)
- [x] Windows: 1024-px-Master `docs/design/icon/AppIcon-1024.png` und Tauri-Icon-Satz in `app/src-tauri/icons/` (ersetzt das
      Standardlogo), `bundle.icon` in `tauri.conf.json` passt; 16 bis 32 px der `icon.ico` aus der vereinfachten Fassung
- [x] `scripts/make-icons.sh` baut Master und Windows-Satz aus den Ebenen neu
- [x] `docs/design.md` Abschnitt „App-Icon“, `mac/README.md` Zeile und Statustabelle

## Hinweise

- Icon Composer ignoriert SVG-Filter (`feGaussianBlur`): die Unschärfe ist ein PNG (`planet-blur.png`), gerechnet aus
  `docs/design/icon/src/planet-blur.svg`.
- Die Windows-Icon wird im selben Commit ersetzt wie das Mac-Icon; ein Nachzug-Eintrag ist deshalb nicht nötig.
- Nicht geprüft: Aussehen auf einem echten Windows-Desktop (Taskleiste, Explorer) und der NSIS-Installer mit dem neuen Icon.
