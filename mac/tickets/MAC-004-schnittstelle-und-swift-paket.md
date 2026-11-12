# MAC-004 Rust-Schnittstelle (UniFFI), XCFramework und Swift-Paket

- Status: erledigt
- Abhängig von: MAC-003
- Teil von: MAC-001, Kriterium „`crates/ffi` mit Swift-Bindings und XCFramework“

## Ziel

`crates/ffi` macht die Session aus `crates/session` über **UniFFI** für Swift verfügbar. Ein Skript baut die statische Bibliothek,
erzeugt die Bindings und bündelt beides zu `SidereaPlanetsFFI.xcframework`, das `mac/Packages/SidereaPlanetsKit` einbindet.

## Entscheidungen

- UniFFI erzeugt `unsafe`-Code. Der Workspace verbietet `unsafe_code` per `forbid`; `crates/ffi` erbt die Workspace-Lints deshalb
  nicht und erlaubt `unsafe` nur dort, begründet im Kommentar. Alle anderen Crates bleiben bei `forbid`.
- Debug-Bau nur `aarch64-apple-darwin`, Release universal (`lipo` mit `x86_64-apple-darwin`).
- Bilder gehen als `Vec<u8>` (RGBA8, Breite, Höhe) bzw. 16-Bit-Puffer über die Grenze; ein Vorgang mit Fortschritt nutzt ein
  Callback-Interface und `async` auf Swift-Seite.

## Akzeptanzkriterien

- [x] `crates/ffi` mit UniFFI, `scripts/build-ffi.sh [--release]` erzeugt Bindings und XCFramework unter `mac/Packages/SidereaPlanetsKit`
      (erzeugte Artefakte gitignored, nur Quellen in Git)
- [x] `mac/Packages/SidereaPlanetsKit`: Swift-Paket mit dünner Schicht (typisierte Aufrufe, `async`, Fortschritt, Abbruch) über den Bindings
- [x] `swift test` im Paket: öffnet `JUPITER.AVI` (wenn vorhanden, sonst übersprungen), liest Frame 1 (1024×680), setzt Alignpoints
      (78 ± 10 % nach Threshold wie in `docs/testdaten.md`), richtet aus und stapelt; Test war einmal rot
- [x] `cargo test --workspace` und `clippy` grün

## Umsetzung

- UniFFI 0.32.2 (Proc-Macros, kein UDL). `crates/ffi`: Objekt `CoreSession` (hält die `Session`), Records und Enums spiegeln die
  Typen des Kerns, `PlanetsError` hat je `SessionError`-Art eine Variante mit `message`, Fortschritt über `ProgressListener`.
  `Operation` heißt in Swift `SessionOperation` (Namenskollision mit Foundation), Zählwerte sind `u64`, Projektabschnitte reisen als
  JSON-Text. Ein `uniffi-bindgen`-Binary im Crate erzeugt die Bindings mit derselben Version.
- Die Swift-Schicht (`PlanetsSession`) führt blockierende Aufrufe auf einer `DispatchQueue` aus (nicht im kooperativen Pool, weil
  sie bis zum Ende oder einer Pause blockieren), `Task`-Abbruch löst `cancel` aus.
- Der Swift-Test war rot, bevor er grün wurde: erst mit der falschen Erwartung `info.width == 1025` (Fehlschlag an der Assertion),
  dann korrigiert auf 1024.
