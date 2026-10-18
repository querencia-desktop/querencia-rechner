#!/bin/bash
# Isolierte GUI-Prüfung von Querencia Rechner (G2-H). Nur Wegwerfprofil,
# eigener headless labwc, eigener D-Bus; die laufende Sitzung bleibt unberührt.
#
#   bash werkzeug/gui-pruefung.sh [BREITExHÖHE] [Binary]
#
# Standard: 1024x768 und das Release-Binary target/release/querencia-rechner
# (sonst das Debug-Binary). Ergebnis: nachweis/gui/<Auflösung>/ergebnis.txt,
# Bilder, rechner.log (Debug-Ausgabe des Programms), lauf-*.log.
set -euo pipefail
E=$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)
R=${1:-1024x768}
case $R in *x*) ;; *) echo "Auflösung als BREITExHÖHE" >&2; exit 2 ;; esac
BIN=${2:-}
[ -n "$BIN" ] && BIN=$(readlink -f "$BIN")
if [ -z "$BIN" ]; then
    if [ -x "$E/target/release/querencia-rechner" ]; then BIN=$E/target/release/querencia-rechner
    else BIN=$E/target/debug/querencia-rechner; fi
fi
[ -x "$BIN" ] || { echo "kein Binary: $BIN" >&2; exit 2; }
[ -x "$E/werkzeug/bin/qeingabe" ] || bash "$E/werkzeug/bauen.sh"
PROFIL=$E/gui-test/profil-$R
[ -d "$E/gui-test" ] && [ "$(dirname "$PROFIL")" = "$E/gui-test" ] && rm -rf "$PROFIL"
OUT=$E/nachweis/gui/$R
rm -rf "$OUT"; mkdir -p "$PROFIL/data/applications" "$PROFIL/data/icons/hicolor/scalable/apps" "$PROFIL/config/labwc" "$OUT"
# Starter und Symbol wie nach der Integration (Starter zeigt auf das Binary).
sed "s|^Exec=.*|Exec=$BIN|; s|^TryExec=.*|TryExec=$BIN|" "$E/data/org.querencia.Rechner.desktop" >"$PROFIL/data/applications/org.querencia.Rechner.desktop"
cp "$E/data/icons/hicolor/scalable/apps/org.querencia.Rechner.svg" "$PROFIL/data/icons/hicolor/scalable/apps/"
# Serverdekoration wie live; das Rechnerfenster liegt fest bei (60, 60).
cat >"$PROFIL/config/labwc/rc.xml" <<X
<?xml version="1.0"?>
<labwc_config>
  <core><decoration>server</decoration></core>
  <windowRules>
    <windowRule identifier="org.querencia.Rechner" title="Taschenrechner">
      <action name="MoveTo" x="60" y="60" />
    </windowRule>
  </windowRules>
</labwc_config>
X
cat >"$PROFIL/env.sh" <<X
BIN=$BIN
Q=$E/werkzeug/bin/qeingabe
QA=$E/werkzeug/bin/qausgabe
OUT=$OUT
R=$R
X
sha256sum "$BIN" >"$OUT/binary.sha256"
SZ=${3:-szenario.sh}
python3 -I "$E/werkzeug/isoliert.py" "$PROFIL" "$E/werkzeug/$SZ" 300 || true
cp "$PROFIL"/logs/*.log "$OUT/" 2>/dev/null || true
cat "$OUT/ergebnis.txt"
