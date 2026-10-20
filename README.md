# Querencia Rechner

Klassischer Taschenrechner für Querencia Desktop nach dem Vorbild von
gcalctool im einfachen Modus (GNOME 2.30). Rust + GTK 4 ohne libadwaita,
Server-Fensterrahmen von labwc (Thema Querencia-Klassisch), Oberfläche
deutsch und englisch (gettext), App-ID `org.querencia.Rechner`.

**Umfang 0.1.0:** Ziffern und Komma, `+ − × ÷` mit Punkt vor Strich und
Klammern, Prozent, Quadratwurzel, Quadrat, Vorzeichenwechsel, Kehrwert,
Speicher (M+, M−, MR, MC mit Marke „M“), Rücktaste, C, AC, vollständige
Tastaturbedienung mit Ziffernblock, Kopieren/Einfügen (Strg+C/Strg+V), Menü
„Rechner“ und „Hilfe → Info“. Fehler (Division durch Null, Überlauf, Wurzel
aus negativer Zahl) erscheinen als Text in der Anzeige.

**0.1.1:** intern volle `f64`-Genauigkeit, gerundet wird nur für Anzeige und
Zwischenablage (`2 √ x²` = 2, `1 ÷ 3 × 3` = 1); siehe `CHANGELOG.md`.

**0.1.2:** Englisch und Deutsch über GNU gettext (siehe „Sprachen“).

**Nicht enthalten:** wissenschaftlicher und Programmierer-Modus, Einheiten,
Verlauf.

| Datei | Inhalt |
|---|---|
| `src/rechenkern.rs` | Parser, Auswertung, Anzeigerundung, Formatierung; Prozentsemantik und Genauigkeit dokumentiert; Unit-Tests |
| `src/modell.rs` | Tasten und Zustand (Eingabezeile, Ergebnis, Speicher); Unit-Tests |
| `src/fenster.rs` | Fenster, Menüleiste, Tastenfeld, Tastatur, Zwischenablage |
| `src/dialoge.rs` | Info-Dialog, Fenstersymbol |
| `data/` | Desktop-Datei, Symbol (nur SVG), Logo C2, eingebettete Ressourcen |
| `src/i18n.rs` | gettext: Textdomäne, Katalogsuche, Platzhalter |
| `po/` | Übersetzungsvorlage und deutscher Katalog |
| `werkzeug/` | isolierte GUI-Prüfung (siehe [docs/testing.md](docs/testing.md)), `i18n.py` für die po-Pflege |
| `integration/` | Integration auf der Entwicklungs-VM (siehe [docs/building.md](docs/building.md)) |

## Sprachen

Seit 0.1.2 sind die Texte englisch im Code (msgid) und deutsch in
`po/de.po` (GNU gettext, Muster Querencia Editor 0.2.0); der deutsche
Wortlaut ist der bisherige. Übersetzt sind Menüs (nach gcalctool 5.30),
Tooltips, Fenstertitel, Info-Dialog, Fehlertexte der Anzeige und die
Desktop-Datei (`data/org.querencia.Rechner.desktop.in` → `…desktop`). Die
Sprache folgt `LANGUAGE`/`LANG`; ohne deutschen Katalog erscheint Englisch.
Der Dezimaltrenner bleibt in beiden Sprachen das Komma.

## Lizenz

GNU GPL, Version 3 oder neuer, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors. Änderungen: [CHANGELOG.md](CHANGELOG.md).
Eigener Vermerk: `werkzeug/virtual-keyboard-unstable-v1.xml` (gekürzte Kopie
des Wayland-Protokolls, Copyright Kristian Høgsberg u. a., MIT-artige
Lizenz im Dateikopf; nur für das Prüfwerkzeug).

