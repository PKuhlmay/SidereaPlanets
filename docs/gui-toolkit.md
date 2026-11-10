# GUI-Toolkit: Tauri 2 + Vue 3

Entscheidung vom 2026-11-10 (RK-002), getroffen mit Philipp.

## Anforderungen
- Modernes Design, von separaten Agenten gebaut
- Windows, macOS und Linux
- Prototyp zuerst

## Entscheidung
**Tauri 2** mit Rust-Kern und Web-Oberfläche (Vue 3, TypeScript, Vite).

- Gestaltungsfreiheit wie im Web; die Design-Skills im Repo (`.claude/design`) arbeiten mit HTML/CSS-Tokens.
- Der Rechenkern bleibt reines Rust (`crates/core`, `crates/io`) und hängt nicht an Tauri.
- Tauri baut für alle drei Plattformen (Windows: WebView2, macOS: WKWebView, Linux: WebKitGTK).

## Verworfen
- **Slint**: native und schnell, aber die vorhandenen Web-Design-Skills greifen nicht.
- **egui**: schneller Einstieg, wirkt aber wie ein Entwicklerwerkzeug.

## Offene Risiken
- Bilddaten laufen zwischen Rust und Webview. Für die Live-Vorschau der Wavelet-Regler (RK-007) muss das gemessen werden; Plan: RGBA-Puffer in Vorschaugröße über ein eigenes Protokoll oder `Channel`, Anzeige per `<canvas>`.
- Linux-Builds brauchen WebKitGTK auf dem Zielsystem.
