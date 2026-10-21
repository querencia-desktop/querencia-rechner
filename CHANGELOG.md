# Änderungen

## 0.1.5 – 2026-10-21

Lesbare Speichermarke im hellen Modus (Kontrast-Sammelauftrag hell, QD-089):

- Die Speichermarke „M“ in der Anzeige schreibt hell `#b33c0b` (Verweisfarbe
  von Querencia-Klassisch) statt `#f07746`: 5,9:1 auf Weiß statt 2,8:1
  (seit 0.1.0 unter AA).
- Dunkel unverändert `#f07746` auf `#2b2a27` (5,1:1); die Farbe steht jetzt
  ausdrücklich im Block `@media (prefers-color-scheme: dark)`.
- Sonst keine Änderung: übrige Farben, Texte, Katalog, Symbol und Rechnen wie
  0.1.4.

## 0.1.4 – 2026-10-21

Dunkle Darstellung (CSS-Welle dunkel, QD-081; für Querencia-Klassisch 0.2):

- Der Rechner folgt „Dunkle Darstellung bevorzugen“ beim Start und beim
  Wechsel im Lauf. Beim Start meldet er `gtk-interface-color-scheme` einmal
  neu (GTK 4.22 wendet ein schon dunkles Farbschema sonst nicht auf das Thema
  an).
- Menüleiste und Menüs ohne eigene Farben: Grund, Rand und Schrift kommen
  vom Thema (hell dieselben Werte wie bisher).
- Anzeige und Löschtasten haben eine dunkle Fassung im Block
  `@media (prefers-color-scheme: dark)`: Anzeige `#2b2a27` mit Rand
  `#23221f`, Ergebnis `#f2f1f0`, Eingabezeile `#a8a29d`, Fehlertext
  `#ed8b85`, Löschtasten `#f9c3be` (alle mindestens 4,5:1). Dafür ist der
  eigene CSS-Provider an `gtk-interface-color-scheme` gebunden (ohne Bindung
  wirkt der Block nie, isoliert gemessen). Die Anzeige wechselt als Ganzes.
- Hell unverändert; Texte, Katalog und Rechnen unverändert.

## 0.1.3 – 2026-10-20

Dunkle Auswahlschrift (CSS-Welle, passend zum GTK-Thema Querencia-Klassisch,
Entscheidung 20.10.):

- Menüleiste, Menüeinträge und Taste „=“ schreiben `#1a1a1a` statt `#ffffff`
  auf den orangen Verlauf. Kontrast 6,2:1 (unten, `#f07746`) bis 8,5:1 (oben,
  `#f6a179`), bei „=“ unter dem Zeiger 6,8:1, gedrückt 5,1:1; mit Weiß waren
  es 1,8:1 bis 3,4:1. Verläufe und alles Übrige unverändert; Texte und
  Katalog unverändert.

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
