# MAC-004 Rust-Schnittstelle (UniFFI), XCFramework und Swift-Paket

- Status: offen
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

- [ ] `crates/ffi` mit UniFFI, `scripts/build-ffi.sh [--release]` erzeugt Bindings und XCFramework unter `mac/Packages/SidereaPlanetsKit`
      (erzeugte Artefakte gitignored, nur Quellen in Git)
- [ ] `mac/Packages/SidereaPlanetsKit`: Swift-Paket mit dünner Schicht (typisierte Aufrufe, `async`, Fortschritt, Abbruch) über den Bindings
- [ ] `swift test` im Paket: öffnet `JUPITER.AVI` (wenn vorhanden, sonst übersprungen), liest Frame 1 (1024×680), setzt Alignpoints
      (78 ± 10 % nach Threshold wie in `docs/testdaten.md`), richtet aus und stapelt; Test war einmal rot
- [ ] `cargo test --workspace` und `clippy` grün
