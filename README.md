# Querencia Rechner

Klassischer Taschenrechner für Querencia Desktop nach dem Vorbild von
gcalctool im einfachen Modus (GNOME 2.30). Rust + GTK 4 ohne libadwaita,
Server-Fensterrahmen von labwc (Thema Querencia-Klassisch), Oberfläche
deutsch und englisch (gettext), App-ID `org.querencia.Rechner`.

> **Alpha.** Nur der einfache Modus; keine Sicherheits- oder
> Datenverlustgarantie.

## Stand

| | |
|---|---|
| Version | 0.1.5 (Zweig `v0.1.5`; Speichermarke hell lesbar, sonst wie 0.1.4 mit dunkler Darstellung für Querencia-Klassisch 0.2) |
| Auf der Entwicklungs-VM | 0.1.4-2 live seit 21.10.2026 10:26; 0.1.5 nicht aktiv (Freigabe durch die Status-Session) |
| Letzte Prüfung | 21.10.2026 (0.1.5): fmt und Clippy sauber, 37/37 Unit-Tests, i18n PASS; isoliert gegen 0.1.4-2 (de 1024×768, en 1920×1080, Thema 0.2): dunkel 22/22 Bilder pixelgleich, hell 22/22 gleich oder nur die Speichermarke, Wechsel 12/12; Speichermarke hell 5,9:1, dunkel 5,1:1 |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22 |

Änderungen je Version: [CHANGELOG.md](CHANGELOG.md).

## Umfang

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

## Bauen und Prüfen

`cargo build --release --locked`, danach `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` und `python3 werkzeug/i18n.py pruefen`.
Voraussetzungen, Katalog, Symbol, Integration und die isolierte GUI-Prüfung: [docs/building.md](docs/building.md), [docs/testing.md](docs/testing.md).

## Sprachen

Seit 0.1.2 sind die Texte englisch im Code (msgid) und deutsch in
`po/de.po` (GNU gettext, Muster Querencia Editor 0.2.0); der deutsche
Wortlaut ist der bisherige. Übersetzt sind Menüs (nach gcalctool 5.30),
Tooltips, Fenstertitel, Info-Dialog, Fehlertexte der Anzeige und die
Desktop-Datei (`data/org.querencia.Rechner.desktop.in` → `…desktop`). Die
Sprache folgt `LANGUAGE`/`LANG`; ohne deutschen Katalog erscheint Englisch.
Der Dezimaltrenner bleibt in beiden Sprachen das Komma.

## Lizenz

GNU GPL, Version 3 oder neuer, siehe [LICENSE](LICENSE) (reiner Lizenztext).
Copyright 2026 Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT).
Eigener Vermerk: `werkzeug/virtual-keyboard-unstable-v1.xml` (gekürzte Kopie
des Wayland-Protokolls, Copyright Kristian Høgsberg u. a., MIT-artige
Lizenz im Dateikopf; nur für das Prüfwerkzeug).

