# Querencia Rechner

Klassischer Taschenrechner für Querencia Desktop nach dem Vorbild von
gcalctool im einfachen Modus (GNOME 2.30). Rust + GTK 4 ohne libadwaita,
Server-Fensterrahmen von labwc (Thema Querencia-Klassisch), deutsche
Oberfläche, App-ID `org.querencia.Rechner`.

**Umfang 0.1.0:** Ziffern und Komma, `+ − × ÷` mit Punkt vor Strich und
Klammern, Prozent, Quadratwurzel, Quadrat, Vorzeichenwechsel, Kehrwert,
Speicher (M+, M−, MR, MC mit Marke „M“), Rücktaste, C, AC, vollständige
Tastaturbedienung mit Ziffernblock, Kopieren/Einfügen (Strg+C/Strg+V), Menü
„Rechner“ und „Hilfe → Info“. Fehler (Division durch Null, Überlauf, Wurzel
aus negativer Zahl) erscheinen als Text in der Anzeige.

**Nicht enthalten:** wissenschaftlicher und Programmierer-Modus, Einheiten,
Verlauf.

| Datei | Inhalt |
|---|---|
| `src/rechenkern.rs` | Parser, Auswertung, Rundung, Formatierung; Prozentsemantik dokumentiert; Unit-Tests |
| `src/modell.rs` | Tasten und Zustand (Eingabezeile, Ergebnis, Speicher); Unit-Tests |
| `src/fenster.rs` | Fenster, Menüleiste, Tastenfeld, Tastatur, Zwischenablage |
| `src/dialoge.rs` | Info-Dialog, Fenstersymbol |
| `data/` | Desktop-Datei, Symbol (nur SVG), Logo C2, eingebettete Ressourcen |
| `werkzeug/` | isolierte GUI-Prüfung (siehe TESTING.md) |
| `integration/` | vorbereitetes Integrationsskript (siehe BUILDING.md) |

Lizenz: GNU GPL, Version 3 oder neuer.
