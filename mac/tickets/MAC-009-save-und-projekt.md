# MAC-009 Schritt Save, Export und Projektdatei

- Status: offen
- Abhängig von: MAC-008
- Teil von: MAC-001, Kriterium „Schritt Save“

## Akzeptanzkriterien

- [ ] Save image (⌘S) in allen Formaten der Windows-App (`export_formats`), mit Farbanpassungen, nativer Dialog
- [ ] Save Aligned und Save Integrated mit Fortschritt
- [ ] Save project und Open project (gleiches Format wie Windows, `crates/io/src/project.rs`)
- [ ] Nach erneutem Align ist Save image gesperrt, bis neu gestapelt ist (Session)
- [ ] Ganzer Ablauf mit JUPITER.AVI bis TIFF 16 Bit läuft durch (Test oder dokumentierter Lauf)
