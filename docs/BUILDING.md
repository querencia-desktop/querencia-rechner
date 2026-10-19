# Bauen

Ubuntu 26.04: `cargo`, `rustc` (≥ 1.92, Ubuntu 1.93.1), `pkgconf`,
`libgtk-4-dev`. Das `Cargo.lock` stammt vom Querencia Editor (gtk4 0.11.5,
glib 0.22.10), damit der Bau aus dem lokalen Crate-Cache gelingt:

```bash
cargo build --release --locked --offline
```

Das Release-Binary liegt unter `target/release/querencia-rechner`; seine
Prüfsumme steht in `nachweis/release/SHA256SUMS`.

**Symbol:** nur `data/icons/hicolor/scalable/apps/org.querencia.Rechner.svg`,
bewusst keine PNG-Stufen. Liegt ein Symbol sowohl in `48x48` als auch in einem
Scalable-Verzeichnis, stürzt GTK 4.22 beim Schließen fokussierter Dialoge ab
. Das Fenstersymbol ist
zusätzlich unter dem Namen `org.querencia.Rechner-fenster` als Ressource
eingebettet (`werkzeug/ressourcen.sh` erzeugt `data/resource/rechner.gresource`).

**Integration 0.1.1 (Tausch 0.1.0 → 0.1.1, nur nach Freigabe über die
Status-Session):** `integration/werkzeug/rechner-integrieren.sh pruefen|aktivieren|zurueck`
legt `~/opt/querencia/rechner/<sha16>/` an, stellt den Link
`~/opt/querencia/bin/querencia-rechner` darauf um und ersetzt den Starter
`~/.local/share/applications/org.querencia.Rechner.desktop` (Vorlagen in
`integration/vorschlag/`, Prüfsummen in `integration/vorschlag/SHA256SUMS`).
`zurueck` stellt Link und Starter von 0.1.0 (`e151ed18bd549976`) wieder her;
der Starter von 0.1.0 liegt dafür in `integration/vorschlag/alt/`. Nach einer
Quelländerung: `cargo build --release --locked --offline`, dann
`bash integration/werkzeug/vorschlag-erzeugen.sh`.
