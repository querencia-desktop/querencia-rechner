# Testen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline                 # Rechenkern und Modell, ohne Display
python3 werkzeug/i18n.py pruefen              # Katalog vollständig, .pot aktuell
```

GUI nur isoliert (eigener headless labwc 0.9.8, eigener D-Bus, eigenes
XDG-Profil, virtuelle Eingaben; nie die laufende Sitzung):

```bash
bash werkzeug/bauen.sh                        # einmalig: qeingabe, qausgabe
bash werkzeug/gui-pruefung.sh 1024x768        # Release-Binary, sonst Debug
bash werkzeug/gui-pruefung.sh 1920x1080
QR_LANGUAGE=en bash werkzeug/gui-pruefung.sh 1024x768 <binary> szenario-sprache.sh
QR_LANGUAGE=it_IT QR_AUSGABE=<ordner> bash werkzeug/gui-pruefung.sh 1024x768 <binary> szenario-sprache.sh
```

Die Prüfung läuft wie die Sitzung mit `LANG=en_US.UTF-8` und `LANGUAGE=de`
(`QR_LANGUAGE=en` für Englisch, `it_IT`, `es_ES`, `fr_FR` für Italienisch,
Spanisch, Französisch; `QR_AUSGABE` legt die Ergebnisse in einen anderen
Ordner statt `nachweis/gui/<Auflösung>/`). Die isolierte Sitzung startet mit
bereinigter Umgebung (`env -i`, HOME im Profil, `GIO_USE_VFS=local`); http(s)
öffnet dort kein Browser, sondern ein Schein-Programm, das die Adresse in
`browser-aufrufe.txt` schreibt. `szenario-sprache.sh` prüft Fehlertext,
Tooltip, Menüs und Info-Dialog der Sprache und fotografiert die längsten
Fehlertexte (Überlauf, Wurzel aus negativer Zahl). Ab 0.1.2 braucht die geprüfte Binary den
Katalog in `<Ordner der Binary>/share/locale`, sonst ist sie englisch und die
deutschen Anzeigeprüfungen (z. B. „Division durch Null“) scheitern.

Ergebnis je Auflösung in `nachweis/gui/<Auflösung>/`: `ergebnis.txt`
(PASS/FAIL/BLOCKED), Bilder `01-…png` bis `18-…png`, `rechner.log`
(Debug-Ausgabe des Programms, daraus werden die Anzeigen geprüft),
`rechner.err` (GTK-Meldungen), `lauf-*.log` (labwc).

Das Szenario (`werkzeug/szenario.sh`) prüft: Start, Kalibrierung der
Tastenpositionen über eine Zeigermeldung, Grundrechnung per Maus und per
Tastatur (Haupttastatur und Ziffernblock), Klammern, Prozent, Funktionen,
Speicher, Genauigkeit (0.1.1: `2 √ x²` = 2, `1/x` = 0,5, `0,5 × 30` = 15,
`1 ÷ 3 × 3` = 1, `0,1 + 0,2 − 0,3` = 0, Kopieren wie die Anzeige, Speicher
mit vollem Wert; Bilder 16–18), Division durch Null und Überlauf, Kopieren/Einfügen (Strg+C/V,
`wl-paste`/`wl-copy`, Menü), Info-Dialog öffnen und mit Tastaturfokus
schließen (Eingabetaste und Escape, danach Tasten im Hauptfenster), zweite
Instanz, Beenden mit Strg+Q.
