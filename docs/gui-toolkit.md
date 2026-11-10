# GUI-Toolkit: Tauri 2 + Vue 3

Entscheidung vom 2026-11-10 (RK-002), getroffen mit Philipp. Messung am selben Tag.

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

Die beiden wurden **nicht gebaut und nicht gemessen**. Der Grund war das Design (Web-Tooling), nicht die Leistung.

## Spike: Wie schnell ist Tauri für unseren Anwendungsfall?

Gemessen wurde nur das gewählte Toolkit, weil dort das Risiko liegt: Bilddaten müssen aus Rust in die Webview.
Code und Rohdaten liegen auf dem Branch `spike/rk-002-tauri` (`docs/rk-002-spike-result.json`).

**Messaufbau:** Release-Build, Windows 11, WebView2 (Chromium 157), Monitor mit 60 Hz (16,7 ms je Bild),
ein Lauf, synthetisches Graustufenbild. Je Größe 3 Aufwärm- und 15 Messläufe, angegeben ist der Median.
„Gesamt“ reicht vom Aufruf bis zum nächsten angezeigten Bild.

| Bildgröße | RGBA-Daten | Rust-Umwandlung | Übertragung (IPC) | Zeichnen | Gesamt |
|---|---|---|---|---|---|
| 1024 × 680 | 2,8 MB | 7,6 ms | 34,8 ms | 0,2 ms | **50 ms** |
| 2048 × 2048 | 16,8 MB | 43,4 ms | 150,8 ms | 1,1 ms | **191 ms** |
| 4096 × 4096 | 67,1 MB | 173,9 ms | 568,1 ms | 4,2 ms | **751 ms** |

**Regler ziehen simuliert** (Bild anfordern, zeichnen, wiederholen, 3 Sekunden):
- 1024 × 680: **20,3 Bilder/s**
- 2048 × 2048: **5,3 Bilder/s**

**Oberfläche selbst:**
- Zoom und Pan eines 2048-²-Bildes: 16,7 ms je Bild, also 60 Hz ohne Ruckler, sowohl per CSS-Transform als auch
  per Neuzeichnen auf ein Canvas (einmal 33,5 ms beim ersten Bild der CSS-Variante).
- 60 Regler, jedes Bild alle Werte geändert: 16,7 ms, größter Wert 16,9 ms. Kein Problem.

### Bewertung
- **Zoom, Pan und viele Regler sind kein Risiko.**
- **Der Engpass ist die Übertragung Rust → Webview**, mit etwa 80–120 MB/s, dazu die Umwandlung in 8-Bit-RGBA.
  Die 16-Bit-Quelle spielt keine Rolle, weil die Vorschau immer 8 Bit ist; gerechnet wird in Rust.
- Für einen Prototyp reicht das: bei Vorschaugröße um 1000 Pixel sind es rund 20 Bilder/s.
  Ganze große Bilder (2048² und mehr) lassen sich so nicht flüssig live bewegen.

### Maßnahmen für die Live-Vorschau (gehören zu RK-007)
- Vorschau auf die Größe des sichtbaren Ausschnitts begrenzen (herunterskaliert oder nur der Bildausschnitt).
- Umwandlung in RGBA parallelisieren (`rayon`); sie ist bisher einfach und einthreadig.
- Frames statt per `invoke` über ein eigenes URI-Schema oder `Channel` holen. **Nicht gemessen**, möglicherweise schneller.
- Neu messen, sobald die echte Wavelet-Rechnung dabei ist; ihre Zeit steckt in den obigen Zahlen nicht drin.

### Grenzen der Messung
- Eine Maschine, ein Lauf, ein synthetisches Bild; keine echte Verarbeitung.
- macOS (WKWebView) und Linux (WebKitGTK) nicht gemessen, dort kann die Übertragung anders aussehen.
- Vergleich mit anderen Toolkits fehlt. Native Toolkits müssten die Bilder nicht in eine Webview kopieren und wären
  dort voraussichtlich schneller, das ist aber eine Vermutung.

## Reproduzieren
Auf dem Branch `spike/rk-002-tauri`:
`cd app && pnpm tauri build --no-bundle`, dann `target/release/planets-app.exe` mit gesetzten Umgebungsvariablen
`PLANETS_SPIKE=1` und `PLANETS_SPIKE_OUT=<Pfad zur JSON-Datei>` starten. Die App misst, schreibt die Datei und beendet sich.

## Weitere Risiken
- Linux-Builds brauchen WebKitGTK auf dem Zielsystem.
