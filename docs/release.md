# Release

Eingabe für das CI-Release-Ticket PLANETS-12. Der Abschnitt zu macOS beschreibt, wie die Tauri-App signiert, notarisiert und als
Installer ausgeliefert wird. Windows: `scripts/publish-windows.sh --installer` (Inno Setup), siehe `CLAUDE.md`.

## macOS: Signieren und Notarisieren

Ausgeliefert wird ein PKG-Installer wie bei Siderea, mit einem Bündel für Intel und Apple Silicon.

**Kennungen**

| Was | Wert |
|---|---|
| Apple-Team | `75QWYYP7DB` |
| Bundle-ID | `de.kuhlmay.siderea.planets` (auch die Kennung des Projektdatei-Typs `de.kuhlmay.siderea.planets.project`) |
| Zertifikat App | `Developer ID Application: <Name> (75QWYYP7DB)` |
| Zertifikat Installer | `Developer ID Installer: <Name> (75QWYYP7DB)` |
| `notarytool`-Profil (Schlüsselbund) | `siderea-notary`, dasselbe wie bei Siderea |
| Bündel | `Siderea Planets.app` (mit Leerzeichen); Dateinamen ohne Leerzeichen: `Siderea-Planets-<version>-OSX-Universal.pkg` / `.zip`, dazu `Siderea-Planets-<version>-OSX-SHA256SUMS.txt` |

Das Profil entsteht einmal je Mac mit
`xcrun notarytool store-credentials siderea-notary --apple-id <Apple-ID> --team-id 75QWYYP7DB` (App-spezifisches Kennwort, nicht das
Apple-ID-Kennwort). Beide Zertifikate liegen im Anmelde-Schlüsselbund. Im CI kommen sie stattdessen aus Secrets (Zertifikate als
`.p12` plus Kennwort, Apple-ID, App-spezifisches Kennwort, Team-ID); nichts davon gehört ins Repo.

**Ablauf**

1. **Vorab prüfen, vor dem langen Bau:** beide Zertifikate (`security find-identity -v -p basic`), das `notarytool`-Profil
   (`xcrun notarytool history --keychain-profile siderea-notary`), das Tag `v<version>`, kein schon gebautes PKG derselben Version.
2. **Universal bauen:** ein Bündel mit `arm64` und `x86_64` (`lipo -archs` muss beide nennen, sonst abbrechen). Für die Tauri-App:
   `pnpm tauri build --target universal-apple-darwin` mit beiden Rust-Targets (`aarch64-apple-darwin`, `x86_64-apple-darwin`);
   `rust-toolchain.toml` führt sie bisher nicht auf.
3. **App signieren:** `codesign --force --deep --options runtime --timestamp --sign "Developer ID Application: …" <App>`, dann
   `codesign --verify --deep --strict`. Hardened Runtime (`--options runtime`) und Zeitstempel sind für die Notarisierung Pflicht. Die
   App wird ohne Sandbox ausgeliefert (Download außerhalb des App Stores); zu prüfen ist, ob die WebView unter Hardened Runtime
   Entitlements braucht (Tauri bringt die nötigen mit, wenn es selbst signiert).
4. **App notarisieren:** `ditto -c -k --keepParent <App> app.zip`, dann
   `xcrun notarytool submit app.zip --keychain-profile siderea-notary --wait`. **`notarytool` endet auch bei einer Ablehnung mit Exit 0,**
   deshalb die Ausgabe auf `status: Accepted` prüfen. Danach `xcrun stapler staple <App>`.
5. **Installer bauen:** App mit `ditto` in ein leeres Stammverzeichnis kopieren, dann
   `pkgbuild --analyze --root <root> components.plist`, in der Plist `BundleIsRelocatable` auf `false` setzen
   (`PlistBuddy -c 'Add :0:BundleIsRelocatable bool false'`; `--analyze` schreibt den Schlüssel nicht), dann
   `pkgbuild --root <root> --component-plist components.plist --identifier de.kuhlmay.siderea.planets --version <version> --install-location /Applications component.pkg`.
   Ohne `BundleIsRelocatable false` aktualisiert der Installer irgendeine andere Kopie der App auf der Platte (Build-Ordner, Downloads)
   statt nach `/Applications` zu installieren. Dann
   `productbuild --package component.pkg --sign "Developer ID Installer: …" Siderea-Planets-<version>-OSX-Universal.pkg`.
6. **PKG notarisieren und stapeln:** wie die App (`notarytool submit … --wait`, auf `Accepted` prüfen, `stapler staple`).
7. **Gatekeeper prüfen, das letzte Wort:** `spctl -a -vv -t exec <App>` und `spctl -a -vv -t install <PKG>` müssen beide als
   notarisiert (Developer ID) durchgehen.
8. **Ausliefern:** PKG, ZIP der gestapelten App (`ditto -c -k --keepParent`, nicht `zip`: erhält Signatur und Dateiattribute) und
   `shasum -a 256` beider Dateien als `…-OSX-SHA256SUMS.txt`.

**Stolpersteine**

- Gebaut wird aus dem Tag in einem Wegwerf-Worktree, damit Arbeitsstand und Build-Ordner unberührt bleiben; im CI entspricht das dem
  frischen Checkout.
- Die Version im Bündel (`CFBundleShortVersionString`) muss zum Tag passen; vor dem Notarisieren vergleichen.
- Zwei Notarisierungen (App, PKG) dauern zusammen mit dem Bau rund 10 Minuten.
- Ein ad hoc signiertes Bündel (`codesign --sign -`) läuft nur auf dem eigenen Mac; auf anderen blockiert Gatekeeper es.
- Die Bundle-ID bleibt `de.kuhlmay.siderea.planets`: Installer und Dateizuordnung tragen sie, und ein Wechsel würde
  Einstellungen (WebView-Speicher unter dieser Kennung) zurücksetzen.
