# Release

Ein Tag `v<version>` baut Siderea Planets für macOS, Windows und Linux, signiert und notarisiert die Mac-App und legt alles als
**Entwurf** (Draft) in GitHub Releases ab. Philipp prüft den Entwurf und veröffentlicht ihn. Der Ablauf steht in
`.github/workflows/release.yml`; geprüft wird bei jedem Push von `.github/workflows/ci.yml`. Es gibt eine Version für alle drei
Plattformen und nur Tags `v<version>` (die früheren `windows-v<version>`-Tags entfallen).

## Was ein Release liefert

| Plattform | Dateien | Gebaut auf |
|---|---|---|
| macOS | `.dmg` (universal, Apple Silicon und Intel), signiert und notarisiert; dazu `.app.tar.gz` und `.sig` für den Updater | `macos-latest` |
| Windows | `…-setup.exe` (NSIS), `…_x64-portable.zip` (die bloße Exe mit `CHANGELOG.md`) | `windows-latest` |
| Linux | `.AppImage` und `.deb` | `ubuntu-22.04` |
| alle | `latest.json` (Updater), `SHA256SUMS.txt` über alle Dateien | |

GitHub ersetzt Leerzeichen in Dateinamen durch Punkte (`Siderea.Planets_1.0.0_universal.dmg`). Linux baut auf 22.04, weil AppImage
und `.deb` gegen die glibc der Build-Maschine linken: ein Bau auf 22.04 (glibc 2.35) läuft auf mehr Distributionen als einer auf 24.04
(glibc 2.39). Windows-Exe und Installer sind **nicht signiert** (SmartScreen warnt beim ersten Start); eine Windows-Signatur gibt es
noch nicht.

## Einmal einrichten: GitHub-Secrets

Repository, Settings, Secrets and variables, Actions, „New repository secret“ (oder `gh secret set <NAME>`). Ohne diese Secrets bricht
der Release-Lauf vor dem langen Bau mit einer Liste der fehlenden ab. Nichts davon gehört ins Repo.

| Secret | Inhalt |
|---|---|
| `APPLE_CERTIFICATE` | Zertifikat „Developer ID Application“ mit privatem Schlüssel als `.p12`, base64-kodiert |
| `APPLE_CERTIFICATE_PASSWORD` | Kennwort, mit dem die `.p12` beim Export geschützt wurde |
| `APPLE_SIGNING_IDENTITY` | Name des Zertifikats: `Developer ID Application: <Name> (75QWYYP7DB)` |
| `APPLE_API_KEY` | Schlüssel-ID des App-Store-Connect-API-Schlüssels (zehn Zeichen) |
| `APPLE_API_ISSUER` | Issuer-ID desselben Schlüssels (UUID) |
| `APPLE_API_KEY_CONTENT` | Inhalt der `AuthKey_<ID>.p8`, samt `-----BEGIN PRIVATE KEY-----`-Zeilen |
| `TAURI_SIGNING_PRIVATE_KEY` | privater Updater-Schlüssel (Inhalt der Schlüsseldatei) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Kennwort des Updater-Schlüssels |

**Zertifikat exportieren.** Schlüsselbundverwaltung, Anmeldung, „Meine Zertifikate“: das Zertifikat „Developer ID Application: … (75QWYYP7DB)“
aufklappen (der private Schlüssel muss darunter hängen), Rechtsklick, „Exportieren“, Format `.p12`, ein Kennwort vergeben. Dann:

```bash
base64 -i Zertifikat.p12 | gh secret set APPLE_CERTIFICATE
gh secret set APPLE_CERTIFICATE_PASSWORD        # fragt nach dem Wert
security find-identity -v -p codesigning        # zeigt den Namen für APPLE_SIGNING_IDENTITY
```

Danach die `.p12` löschen. Das Zertifikat „Developer ID Installer“ aus der PKG-Zeit wird nicht mehr gebraucht: ein `.dmg` braucht nur
das Application-Zertifikat.

**API-Schlüssel für die Notarisierung.** appstoreconnect.apple.com, Benutzer und Zugriff, Integrationen, App-Store-Connect-API,
Team-Schlüssel, „Schlüssel generieren“ mit der Rolle „Developer“. Die Schlüssel-ID steht in der Liste, die Issuer-ID darüber. Die
`.p8` lässt sich nur **einmal** laden; aufbewahren (Passwortverwaltung). Der Schlüssel ersetzt Apple-ID und App-spezifisches Kennwort.

```bash
gh secret set APPLE_API_KEY                      # Schlüssel-ID
gh secret set APPLE_API_ISSUER                   # Issuer-ID
gh secret set APPLE_API_KEY_CONTENT < AuthKey_XXXXXXXXXX.p8
```

**Updater-Schlüssel (PLANETS-13).** Einmal erzeugen; `tauri.conf.json` trägt bis dahin einen Platzhalter, und ohne echten öffentlichen
Schlüssel lehnt jede gebaute App jedes Update ab:

```bash
cd app && pnpm tauri signer generate -w ~/.tauri/siderea-planets.key    # fragt nach einem Kennwort
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/siderea-planets.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

1. Den Inhalt von `~/.tauri/siderea-planets.key.pub` in `app/src-tauri/tauri.conf.json` bei `plugins.updater.pubkey` eintragen (ersetzt
   den Platzhalter) und committen. Der öffentliche Schlüssel ist nicht geheim.
2. Den Inhalt der Schlüsseldatei (ohne `.pub`) als Secret `TAURI_SIGNING_PRIVATE_KEY` und das Kennwort als
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` setzen (die beiden `gh secret set` oben).

**Den privaten Updater-Schlüssel sichern.** Geht er verloren, kann keine installierte App mehr ein Update annehmen; Nutzer müssten von
Hand neu installieren.

## Veröffentlichen

`/siderea-release <version>` erledigt die Schritte 1 bis 4; von Hand:

1. **Version setzen** in der `Cargo.toml` (`[workspace.package]`), `app/package.json` und `app/src-tauri/tauri.conf.json`, überall
   dieselbe Nummer. Der Release-Lauf bricht ab, wenn der Tag nicht zu allen dreien passt.
2. **Changelog:** in `app/CHANGELOG.md` den Abschnitt `## Unreleased` zu `## <version> — <JJJJ-MM-TT>` machen, mit `### English` und
   `### Deutsch`. Genau dieser Abschnitt wird der Text des Releases (`scripts/changelog-section.sh <version>` zeigt ihn). Eine
   Vorabversion wie `1.0.0-rc.1` nimmt den Abschnitt `## 1.0.0-rc.1` oder, wenn es ihn nicht gibt, den von `1.0.0`.
3. **Prüfen**, wie die CI es tut: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cd app && pnpm build && pnpm test`.
4. **Committen und taggen:** `[TASK] PLANETS-N - Release <version>`, dann `git tag -a v<version> -m "Siderea Planets <version>"`.
5. **Pushen, nur nach Philipps OK:** `git push origin main v<version>`. Der Tag startet den Lauf (Actions, „Release“, rund 30 bis 40 Minuten,
   die Notarisierung dauert davon einige Minuten).
6. **Entwurf prüfen** (Releases, Entwurf `Siderea Planets <version>`): alle Dateien der Tabelle oben da, Text aus dem Changelog,
   `SHA256SUMS.txt` vorhanden. Das `.dmg` auf einem Mac laden und öffnen: Gatekeeper darf nicht warnen
   (`spctl -a -vv -t exec "/Applications/Siderea Planets.app"` nennt „Notarized Developer ID“), die Version in der Statusleiste muss
   stimmen. Den Installer unter Windows und das AppImage unter Linux zumindest starten.
7. **Veröffentlichen:** „Publish release“. Erst jetzt sehen Nutzer den Release und der Updater `latest.json`: Der Updater fragt
   `releases/latest/download/latest.json` ab, und ein Entwurf ist für diese Adresse unsichtbar.

Probelauf mit einer Vorabversion: dasselbe mit `1.0.0-rc.1`; der Release wird dann als Vorabversion markiert. Nach dem Test den
Entwurf und den Tag löschen (`gh release delete v1.0.0-rc.1 --cleanup-tag`).

**Updater testen.** GitHub zählt eine Vorabversion nie als „latest“, `releases/latest/download/latest.json` findet sie also nicht.
Für den Test den rc-Release nach dem Veröffentlichen von Hand zum neuesten machen (Releases, „Edit“, „Set as the latest release“, oder
`gh release edit v1.0.0-rc.1 --latest`). Dann eine ältere Version der App installieren und „Check for Updates…“ wählen.

**Der Lauf ist rot.** Die Jobs sind unabhängig; „Re-run failed jobs“ wiederholt nur die roten und nutzt denselben Entwurf (Dateien mit
gleichem Namen werden ersetzt). Meldet der Job „Checksums and completeness“ eine fehlende Plattform in `latest.json`, haben sich zwei
Build-Jobs beim Schreiben überholt: die Build-Jobs noch einmal laufen lassen. Wurde die Notarisierung abgelehnt, steht die
Submission-ID im Protokoll; `xcrun notarytool log <id> --key … --key-id … --issuer …` nennt den Grund. Soll der Tag neu gesetzt werden:
Entwurf löschen, `git push origin :refs/tags/v<version>`, lokal `git tag -d v<version>`, neu taggen.

## Wie der Lauf gebaut ist

- **Drei Stufen:** `create-release` prüft Tag gegen Versionsdateien, liest den Changelog-Abschnitt und legt den Entwurf einmal an;
  `build` (Matrix macOS, Windows, Linux) baut mit `tauri-apps/tauri-action` und lädt in diesen Entwurf; `checksums` lädt alle Dateien
  herunter, prüft, dass jede Plattform geliefert hat, und hängt `SHA256SUMS.txt` an. Wer den Entwurf nur einmal anlegt, vermeidet,
  dass drei Jobs gleichzeitig drei Entwürfe erzeugen.
- **Version in der Statusleiste:** `app/vite.config.ts` liest Version aus `app/package.json`, Commit mit `git rev-parse`, Datum
  aus der Uhr des Baus. Der normale Checkout enthält `.git`, das genügt; ein flacher Checkout reicht für den Commit-Hash.
- **macOS:** `--target universal-apple-darwin` baut beide Rust-Targets (`aarch64-apple-darwin`, `x86_64-apple-darwin`) und fügt sie
  zusammen. Tauri importiert `APPLE_CERTIFICATE` selbst in einen temporären Schlüsselbund, signiert mit Hardened Runtime und
  Zeitstempel und notarisiert mit dem API-Schlüssel (`APPLE_API_KEY_PATH` zeigt auf die aus dem Secret geschriebene `.p8`); der Lauf
  wartet auf das Ergebnis und heftet das Ticket an. Entitlements braucht die App nicht: die WebView (WKWebView) läuft in einem
  Apple-Prozess, im eigenen Prozess gibt es kein JIT, und sie ist nicht sandboxed (Download außerhalb des App Stores). Ändert sich
  das, kommt eine `Entitlements.plist` unter `app/src-tauri/` und `bundle.macOS.entitlements` in die Konfiguration.
- **Windows:** `tauri.windows.conf.json` beschränkt die Bündel auf `nsis`; die ZIP-Datei baut der Workflow aus `target/release/planets-app.exe`.
- **Linux:** `--bundles appimage,deb` (kein `.rpm`).
- **Updater:** `TAURI_SIGNING_PRIVATE_KEY` signiert die Updater-Dateien (`.sig`), `tauri-action` lädt `latest.json` hoch
  (`uploadUpdaterJson`). `bundle.createUpdaterArtifacts` steht **nur** in `app/src-tauri/tauri.release.conf.json`; der Release-Lauf
  gibt sie mit `--config src-tauri/tauri.release.conf.json` an `tauri build` (Pfad relativ zu `app`, wo die CLI läuft). Lokale und
  CI-Builds (`ci.yml`) brauchen deshalb den privaten Schlüssel nicht. `plugins.updater` (Endpunkt und `pubkey`) steht in
  `tauri.conf.json`. Der Job „Checksums and completeness“ prüft `latest.json`: Version, und für `darwin-aarch64`, `darwin-x86_64`
  (beide vom Universal-Bau), `windows-x86_64` und `linux-x86_64` je eine URL und eine Signatur. Das `.deb` aktualisiert sich nicht
  selbst, das AppImage schon.
- **Kosten:** In einem privaten Repository zählen macOS-Minuten zehnfach gegen das Kontingent; in einem öffentlichen sind die
  Runner kostenlos.

## Hintergrund: von Hand signieren und notarisieren

Das Folgende hat beim Mac-Release 0.1.0 (PKG-Installer, Tag `v0.1.0`) von Hand funktioniert. Die Skripte dazu
(`package-mac.sh`, `build-mac-release.sh`) sind mit PLANETS-4 entfallen und stehen in der Git-Historie. Heute macht Tauri die
Schritte 3, 4 und 7 im CI; das Wissen hilft, wenn ein Lauf scheitert oder ein Bündel lokal geprüft werden soll.

**Kennungen**

| Was | Wert |
|---|---|
| Apple-Team | `75QWYYP7DB` |
| Bundle-ID | `de.kuhlmay.siderea.planets` (auch die Kennung des Projektdatei-Typs `de.kuhlmay.siderea.planets.project`) |
| Zertifikat App | `Developer ID Application: <Name> (75QWYYP7DB)` |
| `notarytool`-Profil (Schlüsselbund, nur lokal) | `siderea-notary`, dasselbe wie bei Siderea |
| Bündel | `Siderea Planets.app` (mit Leerzeichen) |

Das lokale Profil entsteht einmal je Mac mit
`xcrun notarytool store-credentials siderea-notary --apple-id <Apple-ID> --team-id 75QWYYP7DB` (App-spezifisches Kennwort, nicht das
Apple-ID-Kennwort). Im CI ersetzt es der API-Schlüssel.

**Ablauf von Hand**

1. **Universal bauen:** ein Bündel mit `arm64` und `x86_64` (`lipo -archs` muss beide nennen). Für die Tauri-App
   `pnpm tauri build --target universal-apple-darwin` mit beiden Rust-Targets.
2. **App signieren:** `codesign --force --deep --options runtime --timestamp --sign "Developer ID Application: …" <App>`, dann
   `codesign --verify --deep --strict`. Hardened Runtime (`--options runtime`) und Zeitstempel sind für die Notarisierung Pflicht.
3. **App notarisieren:** `ditto -c -k --keepParent <App> app.zip`, dann
   `xcrun notarytool submit app.zip --keychain-profile siderea-notary --wait`. **`notarytool` endet auch bei einer Ablehnung mit Exit 0,**
   deshalb die Ausgabe auf `status: Accepted` prüfen. Danach `xcrun stapler staple <App>`.
4. **Gatekeeper prüfen, das letzte Wort:** `spctl -a -vv -t exec <App>` muss als notarisiert (Developer ID) durchgehen.
5. **Ausliefern:** ZIP der gestapelten App mit `ditto -c -k --keepParent` (nicht `zip`: erhält Signatur und Dateiattribute) und
   `shasum -a 256` der Dateien.

Beim früheren PKG-Installer (nur noch Geschichte): `pkgbuild` mit einer Komponenten-Plist, in der `BundleIsRelocatable` auf `false`
steht (`--analyze` schreibt den Schlüssel nicht), sonst aktualisiert der Installer irgendeine andere Kopie der App auf der Platte statt
nach `/Applications` zu installieren; dann `productbuild --sign "Developer ID Installer: …"` und die PKG ein zweites Mal notarisieren.

**Stolpersteine**

- Ein ad hoc signiertes Bündel (`codesign --sign -`) läuft nur auf dem eigenen Mac; auf anderen blockiert Gatekeeper es. Baut Tauri
  ohne `APPLE_CERTIFICATE`, ist das Ergebnis so ein Bündel; deshalb prüft der Workflow die Secrets vor dem Bau.
- Die Version im Bündel (`CFBundleShortVersionString`) muss zum Tag passen. Eine Vorabversion mit Suffix (`1.0.0-rc.1`) ist für
  die Notarisierung egal, würde aber im App Store abgelehnt; wir liefern dort nicht aus.
- Die Bundle-ID bleibt `de.kuhlmay.siderea.planets`: Das Zertifikat und die Dateizuordnung der veröffentlichten Mac-Version 0.1.0
  tragen sie, und ein Wechsel würde Einstellungen (WebView-Speicher unter dieser Kennung) zurücksetzen.
