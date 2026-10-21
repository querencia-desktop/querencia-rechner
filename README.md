# Querencia Rechner

Klassischer Taschenrechner im einfachen Modus für alle, die Querencia Desktop
benutzen. Teil von
[Querencia Desktop](https://github.com/querencia-desktop/querencia) – einer
klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust, GTK 4).

*English summary:* A classic basic-mode calculator modelled on GNOME 2's
gcalctool. Part of Querencia Desktop, a classic GNOME-2-style desktop for
Wayland. Alpha software; documentation is in German.

> **Alpha.** Nur der einfache Modus ist da; wissenschaftlicher und
> Programmierer-Modus fehlen. Keine Sicherheits- oder Datenverlustgarantie.

## Zweck

- Grundrechenarten `+ − × ÷` mit Punkt vor Strich und Klammern, Ziffern und
  Komma, Prozent, Quadratwurzel, Quadrat, Vorzeichenwechsel, Kehrwert.
- Speicher (M+, M−, MR, MC mit Marke „M“), Rücktaste, C, AC, vollständige
  Tastaturbedienung mit Ziffernblock, Kopieren/Einfügen (Strg+C/Strg+V),
  Menü „Rechner“ und „Hilfe → Info“. Fehler (Division durch Null, Überlauf,
  Wurzel aus negativer Zahl) erscheinen als Text in der Anzeige.
- Intern volle `f64`-Genauigkeit; gerundet wird nur für Anzeige und
  Zwischenablage (12 Stellen), z. B. `2 √ x²` = 2 und `1 ÷ 3 × 3` = 1.
- Bewusst **nicht** Teil dieses Programms: wissenschaftlicher und
  Programmierer-Modus, Einheiten, Verlauf.

Anwendungskennung (App-ID): `org.querencia.Rechner` · Binary:
`querencia-rechner` · Debian-Paket: `querencia-rechner`

| Datei | Inhalt |
|---|---|
| `src/rechenkern.rs` | Parser, Auswertung, Anzeigerundung, Formatierung; Prozentsemantik und Genauigkeit dokumentiert; Unit-Tests |
| `src/modell.rs` | Tasten und Zustand (Eingabezeile, Ergebnis, Speicher); Unit-Tests |
| `src/fenster.rs` | Fenster, Menüleiste, Tastenfeld, Tastatur, Zwischenablage |
| `src/dialoge.rs` | Info-Dialog, Fenstersymbol |
| `data/` | Desktop-Datei, Symbol (nur SVG), Logo, eingebettete Ressourcen |
| `werkzeug/` | isolierte GUI-Prüfung (siehe docs/testing.md) |

## Stand

| | |
|---|---|
| Version | 0.1.5 (Tag `rechner/0.1.5`; dunkle Darstellung seit 0.1.4, hell lesbare Speichermarke) |
| Letzte Prüfung | 21.10.2026: fmt und Clippy sauber, 37 Unit-Tests, Sprache Deutsch und Englisch; isolierte GUI-Prüfung hell und dunkel (1024×768, 1920×1080), Wechsel hell/dunkel 48 Fälle, Kontrast dunkel 112 Paare WCAG AA |
| Bekannte Lücken | siehe „Zweck“ (nur einfacher Modus) |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, Rust 1.93 |
| Nicht getestet | andere Distributionen, HiDPI, Mehrmonitor |

Änderungen: [CHANGELOG.md](CHANGELOG.md).

## Bauen

Voraussetzungen (Ubuntu 26.04):

```bash
sudo apt install build-essential pkg-config rustc cargo libgtk-4-dev
```

```bash
cargo build --release --locked
```

Das Ergebnis liegt unter `target/release/querencia-rechner`. Ausführliche
Hinweise (Toolchain-Version, Offline-Bau, Symbol): [docs/building.md](docs/building.md).

## Prüfen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 werkzeug/i18n.py pruefen
```

Diese Befehle laufen ohne Bildschirm; die ersten drei sind auch die CI
(`.github/workflows/ci.yml`), die po-Prüfung braucht gettext und läuft lokal. Die **GUI-Prüfung** läuft in einer
abgeschotteten Sitzung (eigener labwc ohne Bildschirm, eigenes `HOME` und
`XDG_RUNTIME_DIR`), niemals in der laufenden Desktopsitzung:

```bash
bash werkzeug/bauen.sh
bash werkzeug/gui-pruefung.sh 1024x768
```

Fälle, Ablauf und Belege: [docs/testing.md](docs/testing.md). Arbeitsweise insgesamt:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Sprachen

Seit 0.1.2 sind die Texte englisch im Code (msgid) und deutsch in
`po/de.po` (GNU gettext, Muster Querencia Editor 0.2.0); der deutsche
Wortlaut ist der bisherige. Übersetzt sind Menüs (nach gcalctool 5.30),
Tooltips, Fenstertitel, Info-Dialog, Fehlertexte der Anzeige und die
Desktop-Datei (`data/org.querencia.Rechner.desktop.in` → `…desktop`). Die
Sprache folgt `LANGUAGE`/`LANG`; ohne deutschen Katalog erscheint Englisch.
Der Dezimaltrenner bleibt in beiden Sprachen das Komma.

## Vorbild und Abgrenzung

Vorbild für Bedienung und Aussehen ist **gcalctool 5.30 aus GNOME 2.30 im
einfachen Modus**. Querencia übernimmt davon die Bedienideen (Menüs, Kürzel,
Anordnung), **keinen Quellcode, keine Texte, keine Grafiken**. Das Projekt ist
mit gcalctool, GNOME und deren Autoren weder verbunden noch von ihnen
unterstützt.

Abweichungen vom Vorbild und Dinge, die bewusst fehlen:

- GTK 4 statt GTK 2, Fensterrahmen vom Compositor (labwc), kein libadwaita.
- Nur der einfache Modus; kein Verlauf, keine Einheiten.

Fremdbestandteile: nur Rust-Abhängigkeiten (gtk4-rs und Umfeld) unter MIT,
MIT OR Apache-2.0, Apache-2.0 WITH LLVM-exception, Unicode-3.0 und Unlicense
OR MIT; Liste mit `cargo metadata --locked`. Dynamisch gelinkt: GTK 4 und
seine Systembibliotheken. Für die GUI-Prüfung liegt in `werkzeug/` eine
gekürzte Fassung des wlroots-Protokolls `virtual-keyboard-unstable-v1.xml`
mit eigenem Vermerk (© 2008–2018 Kristian Høgsberg, Intel Corporation,
Collabora, Ltd., Purism SPC; MIT, siehe Dateikopf).

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors; Urheberhinweis in [COPYRIGHT](COPYRIGHT). Die
Protokolldatei `werkzeug/virtual-keyboard-unstable-v1.xml` behält ihren eigenen
Vermerk (siehe oben).

Name und Logo „Querencia“ sind keine freien Inhalte: [MARKE.md](MARKE.md).

## Fehler melden

Fehler, Wünsche und Fragen zu Querencia gehen an die gemeinsame Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.
Sicherheitslücken bitte nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).
