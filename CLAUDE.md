# SidereaPlanets

Prototyp: Nachbau von **the reference program** (Planeten-/Mond-Stacking und Wavelet-Schärfung für die Astrofotografie)
in **Rust**. Das Original (Delphi, Windows) ist kostenlos, wird aber seit 2011 nicht mehr gepflegt.
Ziel ist ein möglichst genauer Klon in **Bedienung und Ergebnis** — gleicher Workflow, gleiche Begriffe,
vergleichbare Resultate. Es ist zunächst nur ein Prototyp, um zu prüfen, ob das Vorhaben überhaupt aufgeht.

## Status

Frühe Phase, noch kein Code. Philipp installiert das Original (the reference program) lokal; daraus entsteht per
Reverse Engineering die Spezifikation (Verhalten, Oberfläche, Dateiformate, Algorithmen).

## Tech Stack

- Rust (stable, Edition 2021+), Cargo-Workspace — Crates werden mit den Tickets angelegt
- GUI-Toolkit: offen, wird in RK-002 entschieden
- Bildverarbeitung: eigene Implementierung der Algorithmen, keine Delphi-Quellen

## Dev-Commands

Noch keine. Sobald das Workspace-Gerüst steht (RK-001), hier ergänzen:
`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`.

## Verzeichnisstruktur

```
tickets/   Aufgaben als Markdown-Dateien (RK-NNN-kurzer-titel.md), kein Jira
docs/      Spezifikation, die beim Reverse Engineering entsteht (Workflow, UI, Algorithmen, Formate)
re-local/  Lokal, nicht in Git: Original-Binaries, Beispieldaten, Screenshots, Mitschnitte
.claude/   Hooks, Skills und Einstellungen aus `npx github:PKuhlmay/claude-kit install` (nicht von Hand ändern)
```

## Tickets

- Nur im Repo unter `tickets/`, **kein Jira**. Dateiname `RK-NNN-kurzer-titel.md`, fortlaufend nummeriert.
- Kopf: Status (`offen` · `in Arbeit` · `erledigt`), Abhängigkeiten, dann Ziel und Akzeptanzkriterien.
- Status-Änderung direkt in der Datei und im selben Commit wie die Umsetzung.
- Commits: `[TYPE] RK-NNN - Message in English` (Types wie in den globalen Regeln).

## Reverse Engineering

- Quelle ist das **beobachtbare Verhalten** des installierten Originals: Oberfläche, Ein-/Ausgaben,
  Dateiformate, Vergleich von Ergebnisbildern. Erkenntnisse landen als Beschreibung in `docs/`.
- Original-Binaries, Ressourcen, Icons und Texte werden **nicht** ins Repo übernommen oder kopiert
  (`re-local/` ist gitignored). Der Klon bekommt eigene Implementierung und eigene Assets.
- Algorithmen, die nur geraten sind, in `docs/` als **Vermutung** markieren, bis ein Vergleich mit dem
  Original sie bestätigt.

## Konventionen

- Bildverarbeitung prüfen wir **gegen das Original**: gleiche Eingabe, Ergebnis vergleichen
  (Differenzbild, PSNR). Referenzfälle liegen in `docs/` beschrieben, Daten in `re-local/`.
- Rechenkern (Stacking, Wavelets) ist GUI-frei in eigenen Crates und per Unit-Test prüfbar.
- Performance zählt: Videos mit tausenden Frames, deshalb Streaming statt alles im RAM, Parallelität via `rayon`.
- `clippy` ohne Warnungen, `cargo fmt`; kein `unsafe` ohne Begründung im Kommentar.
