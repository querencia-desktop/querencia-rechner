# Bauen

Ubuntu 26.04: `cargo`, `rustc` (≥ 1.92, Ubuntu 1.93.1), `pkgconf`,
`libgtk-4-dev`. Das `Cargo.lock` stammt vom Querencia Editor (gtk4 0.11.5,
glib 0.22.10), damit der Bau aus dem lokalen Crate-Cache gelingt:

```bash
cargo build --release --locked --offline
```

Das Release-Binary liegt unter `target/release/querencia-rechner`. Für
Pakete und Anhänge mit `RUSTFLAGS="--remap-path-prefix=$HOME=~"` bauen,
damit keine Pfade des Bauordners in der Binary stehen.

**Übersetzungen (ab 0.1.2):** Die Binary sucht `<lang>/LC_MESSAGES/querencia-rechner.mo`
in `<Ordner der Binary>/share/locale`, dann `<Ordner der Binary>/../share/locale`,
zuletzt `/usr/share/locale`. Den Katalog erzeugt GNU gettext oder das
Werkzeug des Repos:

```bash
msgfmt -c -o share/locale/de/LC_MESSAGES/querencia-rechner.mo po/de.po
python3 werkzeug/i18n.py mo <ziel>          # <ziel>/de/LC_MESSAGES/querencia-rechner.mo
```

Ohne Katalog ist die Oberfläche englisch.

**Symbol:** nur `data/icons/hicolor/scalable/apps/org.querencia.Rechner.svg`,
bewusst keine PNG-Stufen. Liegt ein Symbol sowohl in `48x48` als auch in einem
Scalable-Verzeichnis, stürzt GTK 4.22 beim Schließen fokussierter Dialoge ab
(GTK-Symbolabsturz, geprüft im Oktober 2026). Das Fenstersymbol ist
zusätzlich unter dem Namen `org.querencia.Rechner-fenster` als Ressource
eingebettet (`werkzeug/ressourcen.sh` erzeugt `data/resource/rechner.gresource`).

**Integration (nur in der Entwicklungsumgebung, nicht Teil der Veröffentlichung):**
Die Live-Integration in die Querencia-Sitzung der Entwicklungsumgebung erledigen
Skripte unter `integration/` (`pruefen|aktivieren|zurueck`, unveränderliche
Ablage `~/opt/querencia/rechner/<sha16>/`, Link `~/opt/querencia/bin/querencia-rechner`,
Starter). Der Ordner enthält feste Pfade der VM und wird vor der
Veröffentlichung herausgefiltert.
