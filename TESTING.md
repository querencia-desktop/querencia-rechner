# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline                 # Rechenkern und Modell, ohne Display
```

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, virtuelle Eingaben; nie die laufende Sitzung):

```bash
bash werkzeug/bauen.sh                        # einmalig: qeingabe, qausgabe
bash werkzeug/gui-pruefung.sh 1024x768        # Release-Binary, sonst Debug
bash werkzeug/gui-pruefung.sh 1920x1080
```

Ergebnis je Auflösung in `nachweis/gui/<Auflösung>/`: `ergebnis.txt`
(PASS/FAIL/BLOCKED), Bilder `01-…png` bis `14-…png`, `rechner.log`
(Debug-Ausgabe des Programms, daraus werden die Anzeigen geprüft),
`rechner.err` (GTK-Meldungen), `lauf-*.log` (labwc).

Das Szenario (`werkzeug/szenario.sh`) prüft: Start, Kalibrierung der
Tastenpositionen über eine Zeigermeldung, Grundrechnung per Maus und per
Tastatur (Haupttastatur und Ziffernblock), Klammern, Prozent, Funktionen,
Speicher, Division durch Null und Überlauf, Kopieren/Einfügen (Strg+C/V,
`wl-paste`/`wl-copy`, Menü), Info-Dialog öffnen und mit Tastaturfokus
schließen (Eingabetaste und Escape, danach Tasten im Hauptfenster), zweite
Instanz, Beenden mit Strg+Q.
