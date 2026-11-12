# RK-014 Geführter Ablauf: Schrittleiste statt Reiter und Aktionszeile

- Status: erledigt
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
- [x] Nach jedem Lauf öffnet sich der nächste Schritt selbst; ein erneutes Set Alignpoints, Align oder Limit verwirft
  wie in RK-005 festgelegt alles danach und die Leiste fällt auf den passenden Schritt zurück
  - Umgesetzt (`app/src/steps.ts`). Im echten Fenster am 2026-11-12 mit einer Aufnahme durchgeklickt: Select → Align →
    Limit → Stack → Wavelets → Do All → Save, Auto-Weiter und Rückfall bei Cancel gesehen. Als Test: `flow.test.ts`,
    Abschnitt „The panel follows the runs“ (siehe Stand).

Tests und Doku
- [x] `app/src/logic/workflow.ts` bildet die sieben Schritte ab (offen, erledigt, gesperrt, nächster Schritt), mit
  Tests, die zuerst rot waren
  - Tests in `workflow.test.ts`, `qualityChart.test.ts`, `flow.test.ts` und `align-pages.test.ts`. Weil die Testpause sie
    nie rot sehen ließ, ist der Nachweis nachgeholt, siehe Stand.
- [x] Anwenderdoku in `app/src/docs/pages/` nachgezogen: „window-at-a-glance“, „align-tab“, „align-your-frames“ und
  jede Seite, die Reiter oder Aktionszeile nennt

## Stand
Rot-Nachweis am 2026-11-12, nachträglich: Die Quelle wurde je Test vorübergehend mutiert (eine Änderung je Lauf), nur die
betroffene Testdatei lief, die Quelle wurde danach unverändert wiederhergestellt (`git diff` leer). Alle Mutationen unten
wurden an einer Assertion rot (`AssertionError`), nie an einem Import- oder Setup-Fehler.

- `workflow.test.ts` (Quelle `logic/workflow.ts`, 43 Mutationen): rot sind u. a. Sperrbedingung je Schritt (`requires` für
  Align, Stack, Wavelets, Save) → „follow the table row by row“; Save ohne Wavelets erledigt, Wavelets ohne Stack erledigt →
  „take the check mark …“ und „ignore a stale Do All“; nächster Schritt am Ende `"open"` statt `"save"` → „open the next step
  after each run“; Wortlaut und Auswahl der Sperr-Tooltips; `againDiscards` für Do All; Bildart, Bildwahl und Bildunterschrift
  je Schritt; Ergebnistexte; Fenstertitel; Zeitanzeige; `stepAfter`/`stepBefore`; Beschriftungen und Aktionen. Einzige
  überlebende Mutation: `againDiscards` für Save image warnen zu lassen, sie ist wirkungslos (Save ist der letzte Schritt, nach
  ihm gibt es nichts zu verwerfen).
- `qualityChart.test.ts` (Quelle `logic/qualityChart.ts`, 24 Mutationen): Balkenreihenfolge umgedreht oder nicht sortiert →
  „sorts the bars best first“ rot. Fünf Mutationen blieben grün und führten zu neuen Tests, die danach rot wurden: Frame genau
  auf der Grenze bei Lowest Quality (`>` statt `>=`), leere Liste, Balken bei einem Bestwert ungleich 100 %, mittlere Höhe eines
  zusammengefassten Balkens, Gleichstand „halb behalten“ in einem zusammengefassten Balken.
- `flow.test.ts` (Quellen `steps.ts`, `align.ts`, `stack.ts`, `actions.ts`, `saving.ts`, `recording.ts`, 17 Mutationen): Das
  Weiterschalten nach Stack, Limit, Align und Do All war abgedeckt (`advance()` entfernt → rot). Nicht abgedeckt waren und jetzt
  im neuen Abschnitt „The panel follows the runs“ stehen: Select öffnet Alignpoints, Set Alignpoints öffnet Align (ohne
  `advance()` nach Set Alignpoints war der Test grün), Save image aus dem Schritt Wavelets öffnet Save (ohne `advance()` in
  `saving.ts` grün), Rückfall bei erneutem Limit, Align und Set Alignpoints mit den Zuständen der Pillen, Cancel von Stack und
  Do All lässt den Schritt offen, Wavelets und Save verlieren den Haken, wenn nach Do All und Save image eine Ebene geändert wird, und
  kommen mit einem neuen Do All zurück. Alle diese Mutationen sind jetzt rot. Zwei Mutationen sind wirkungslos und bleiben
  grün: `advance()` auch nach einem abgebrochenen Do All (der nächste offene Schritt ist dann ohnehin Wavelets) und
  `waveletsDone` ohne die Prüfung auf den Schritt Stack (`doneMap` prüft sie schon).
- `align-pages.test.ts` (Quellen `docs/pages/*.md`, `docs/docs.ts`, rund 35 Mutationen): Gruppenname, Reihenfolge der Seiten, Link auf
  eine Seite, die es nicht gibt, Anker-Link, „Not available yet“ auf der falschen Seite, jeder Feldname aus der Liste (an allen
  Stellen entfernt), Zeilen der Limit-Tabelle, die Zeilen „Scan Frames“, „Empty List“ und „No Align“ der Übersichtsseite, der
  Seitenname der Übersicht (Datei umbenannt) und die Volltextsuche → jeweils rot.
- **Show Advanced Controls** (2026-11-12, Fund aus dem Durchlauf der Release-App): Das Häkchen im Settings-Menü war wirkungslos. Entscheidung
  Philipps: Beim Setzen klappt es die Advanced-Bereiche aller Schritte auf (`app/src/steps.ts`), danach lässt sich jeder einzeln weiter
  auf- und zuklappen; Entfernen des Häkchens lässt sie, wie sie sind. Test: `app/src/flow.test.ts` („Settings > Show Advanced Controls“),
  rot an Assertionen gesehen, dann grün.

## Offen
- Texte der sieben Erklärsätze: der Entwurf enthält einen ersten Stand, Philipp nimmt sie ab.
- Welche Einstellungen je Schritt als „wichtig“ gelten und welche unter „Advanced“ liegen: der Entwurf macht einen
  Vorschlag, entschieden wird bei der Umsetzung des Designs.
