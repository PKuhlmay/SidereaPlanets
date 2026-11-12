# RK-014 Geführter Ablauf: Schrittleiste statt Reiter und Aktionszeile

- Status: in Arbeit
- Abhängig von: RK-010, RK-012

## Ziel
Die Oberfläche führt durch den Ablauf, statt ihn wie das Original über Reiter, Aktionszeile und Panel zu verteilen
(Entscheidung von Philipp, 2026-11-11, Variante A aus `docs/design/vorschlag-gefuehrter-ablauf.html`). Vorbild ist die
Schrittleiste von Siderea.

Aus den drei Reitern werden sieben Schritte: **Open · Alignpoints · Align · Limit · Stack · Wavelets · Save**. Das linke
Panel zeigt nur den offenen Schritt: ein Satz Erklärung, die wichtigen Einstellungen, unten die eine gefüllte
Schaltfläche. Nach dem Lauf öffnet sich der nächste Schritt von selbst.

Die englischen Beschriftungen des Originals bleiben (Set Alignpoints, Align, Limit, Stack, Do All, Save image und alle
Feldnamen aus `docs/ui.md`). Sperrlogik, erneutes Ausrichten (RK-005) und der Rechenkern ändern sich nicht. Das ist eine
bewusste Abweichung vom Original: Grundsatz 1 in `docs/design.md` gilt danach nur noch für die Namen, nicht mehr für
die Orte.

## Akzeptanzkriterien

Design
- [x] `docs/design.md` nachgezogen: Grundsatz 1 („gleicher Ort, gleicher Name“ wird „gleicher Name“), Fensteraufbau ohne
  Reiterleiste und Aktionszeile, neue Komponente Schrittleiste, Abschnitte Zustände und Bildschirme je Schritt, Eintrag
  unter „Bewusste Abweichungen vom Original“
- [x] Klickbarer Entwurf `docs/design/oberflaeche-entwurf.html`, Soll-Bilder und die FigCraft-Datei zeigen die sieben
  Schritte, dunkel und hell; `vorschlag-gefuehrter-ablauf.html` ist danach gelöscht oder als überholt gekennzeichnet

Schrittleiste
- [x] Eine Zeile unter der Werkzeugleiste mit den sieben Schritten als Pillen mit Nummer im Kreis: offen in
  `--accent-soft` mit Kreis in `--accent`, erledigt mit Haken in `--ok`, noch nicht möglich in `--off` und nicht
  anklickbar, Tooltip nennt den fehlenden Schritt („Run Align first“)
- [x] Erledigte Schritte lassen sich öffnen; ihr Ergebnis steht im Tooltip der Pille
- [x] Bei der Mindestbreite von 1120 px läuft die Zeile nicht über

Panel je Schritt
- [x] Kopf mit „Step n of 7“, Name und einem Satz, was der Schritt tut
- [x] Darüber eine Zeile in `--ok` mit dem Ergebnis des vorigen Schritts (z. B. „Alignpoints · 82 set“)
- [x] Nur die Einstellungen dieses Schritts; selten gebrauchte liegen eingeklappt unter „Advanced“ und sind dort
  vollständig, kein Feld aus `docs/ui.md` geht verloren
- [x] Unten im Panel genau eine gefüllte Schaltfläche mit der Beschriftung des Originals, darunter „Then: ‹nächster
  Schritt›“. Pro Fenster ist außerhalb einer Berechnung nur diese eine Schaltfläche gefüllt
- [x] Ein erledigter, wieder geöffneter Schritt zeigt sein Ergebnis, „‹Action› again“ als Standard-Schaltfläche mit dem
  Hinweis, dass ein neuer Lauf die Schritte danach verwirft, und gefüllt „Continue: ‹nächster offener Schritt›“
- [x] Limit zeigt die Qualität der Frames als Balken, bestes Frame links, mit der Grenze als Linie

Übrige Oberfläche
- [x] Die Häkchen der bisherigen Aktionszeile (Show Framelist, Show Registrationgraph, Show Alignpoints …) liegen in
  einem Menü „Show“ oben rechts am Bild; die schwebenden Karten verhalten sich wie bisher
- [x] Functions bleibt im Schritt Wavelets rechts neben dem Bild, Process bleibt erreichbar
- [x] Frame-Leiste in den Schritten Alignpoints, Align und Limit; Statusleiste, Cancel und Pause unverändert
- [ ] Nach jedem Lauf öffnet sich der nächste Schritt selbst; ein erneutes Set Alignpoints, Align oder Limit verwirft
  wie in RK-005 festgelegt alles danach und die Leiste fällt auf den passenden Schritt zurück
  - Umgesetzt (`app/src/steps.ts`), aber noch nicht in der laufenden App mit einer echten Aufnahme durchgeklickt.

Tests und Doku
- [ ] `app/src/logic/workflow.ts` bildet die sieben Schritte ab (offen, erledigt, gesperrt, nächster Schritt), mit
  Tests, die zuerst rot waren
  - Umgesetzt mit Tests in `workflow.test.ts`, `qualityChart.test.ts` und `flow.test.ts`; nicht gelaufen, solange die
    Testläufe pausiert sind (Philipp, 2026-11-10), also auch nicht rot gesehen.
- [x] Anwenderdoku in `app/src/docs/pages/` nachgezogen: „window-at-a-glance“, „align-tab“, „align-your-frames“ und
  jede Seite, die Reiter oder Aktionszeile nennt

## Offen
- Texte der sieben Erklärsätze: der Entwurf enthält einen ersten Stand, Philipp nimmt sie ab.
- Welche Einstellungen je Schritt als „wichtig“ gelten und welche unter „Advanced“ liegen: der Entwurf macht einen
  Vorschlag, entschieden wird bei der Umsetzung des Designs.
