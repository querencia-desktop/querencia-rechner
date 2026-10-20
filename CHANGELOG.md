# Änderungen

## 0.1.2 – 2026-10-20

Englisch und Deutsch über GNU gettext (G2-AH, Muster Querencia Editor 0.2.0):

- Texte englisch im Code, Deutsch aus `po/de.po` mit dem bisherigen
  Wortlaut: Menüs (`_Calculator`/`_Rechner`, `_Help`/`_Hilfe`), Tooltips,
  Fenstertitel („Calculator“/„Taschenrechner“), Info-Dialog, Fehlertexte der
  Anzeige, Desktop-Datei (englischer Name jetzt „Calculator“).
- Rechnen, Anzeige, Tastatur und Zwischenablage unverändert; der
  Dezimaltrenner bleibt das Komma.
- `werkzeug/i18n.py` für die po-Pflege; GUI-Prüfung mit `QR_LANGUAGE` und
  neuem Szenario `szenario-sprache.sh`.
- Doku nach ENTSCHEIDUNGEN Nr. 17 (`docs/building.md`, `docs/testing.md`,
  Arbeitsregeln), Lizenzkopf (Nr. 14 Q5).

## 0.1.1 – 2026-10-19

Volle Genauigkeit (G2-AB, Werkstatt-Abnahme QREL-05, FAIL R.3):

- Der Rechenkern rechnet intern mit voller `f64`-Genauigkeit weiter. Bisher
  wurden Ergebnisse und die Funktionen √, x², 1/x auf 12 Stellen gerundet und
  mit dem gerundeten Wert weitergerechnet (`2 √ x²` = 1,99999999999, danach
  `1/x` = 0,500000000003). Jetzt: `2 √ x²` = 2, `1/x` = 0,5, `1 ÷ 3 × 3` = 1,
  auch beim Weiterrechnen mit einem Ergebnis, aus dem Speicher (MR) und mit
  eingefügten Zahlen.
- Gerundet wird nur noch für die Anzeige und die Zwischenablage (12 Stellen,
  wie bisher).
- Heben sich Summe oder Differenz bis auf einen Binärrest auf, ist das
  Ergebnis 0 (`0,1 + 0,2 − 0,3` = 0 statt 5,55111512313e-17; gilt auch für
  M+/M−).
- Unverändert: Prozent, Speicher, Vorrang, Klammern, Bedienung. Eine Ziffer
  direkt nach einem Funktionsergebnis ergänzt weiterhin „ד (Vorbild
  gcalctool 5.30 beginnt nur nach Gleich neu).

## 0.1.0 – 2026-10-18

Erste Fassung (G2-H, AP-10a): einfacher Modus nach gcalctool 5.30 – Grundrechenarten mit
Punkt vor Strich und Klammern, Prozent, Wurzel, Quadrat, Vorzeichen, Kehrwert,
Speicher, Tastaturbedienung mit Ziffernblock, Zwischenablage, Info-Dialog.
