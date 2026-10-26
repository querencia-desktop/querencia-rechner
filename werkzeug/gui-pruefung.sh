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
# QR_AUSGABE: anderer Ergebnisordner (z. B. für Läufe je Sprache außerhalb des Repos)
OUT=${QR_AUSGABE:-$E/nachweis/gui/$R}
rm -rf "$OUT"; mkdir -p "$PROFIL/data/applications" "$PROFIL/data/icons/hicolor/scalable/apps" "$PROFIL/config/labwc" "$OUT"
# Starter und Symbol wie nach der Integration (Starter zeigt auf das Binary).
sed "s|^Exec=.*|Exec=$BIN|; s|^TryExec=.*|TryExec=$BIN|" "$E/data/org.querencia.Rechner.desktop" >"$PROFIL/data/applications/org.querencia.Rechner.desktop"
cp "$E/data/icons/hicolor/scalable/apps/org.querencia.Rechner.svg" "$PROFIL/data/icons/hicolor/scalable/apps/"
# Kein echter Browser in der Prüfung: http(s) öffnet ein Schein-Programm, das
# nur die Adresse protokolliert (Online-Hilfe, Fehler melden).
cat >"$PROFIL/data/applications/qr-kein-browser.desktop" <<X
[Desktop Entry]
Type=Application
Name=kein Browser
Exec=sh -c 'echo "\$1" >>"$OUT/browser-aufrufe.txt"' qr-kein-browser %u
MimeType=x-scheme-handler/http;x-scheme-handler/https;
NoDisplay=true
X
mkdir -p "$PROFIL/config"
printf '[Default Applications]\nx-scheme-handler/http=qr-kein-browser.desktop\nx-scheme-handler/https=qr-kein-browser.desktop\n' >"$PROFIL/config/mimeapps.list"
# Serverdekoration wie live; das Rechnerfenster liegt fest bei (60, 60).
cat >"$PROFIL/config/labwc/rc.xml" <<X
<?xml version="1.0"?>
<labwc_config>
  <core><decoration>server</decoration></core>
  <windowRules>
    <windowRule identifier="org.querencia.Rechner" title="Taschenrechner">
      <action name="MoveTo" x="60" y="60" />
    </windowRule>
    <windowRule identifier="org.querencia.Rechner" title="Calculator">
      <action name="MoveTo" x="60" y="60" />
    </windowRule>
    <windowRule identifier="org.querencia.Rechner" title="Calcolatrice">
      <action name="MoveTo" x="60" y="60" />
    </windowRule>
    <windowRule identifier="org.querencia.Rechner" title="Calculadora">
      <action name="MoveTo" x="60" y="60" />
    </windowRule>
    <windowRule identifier="org.querencia.Rechner" title="Calculatrice">
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
# Sprache wie die Sitzung (LANG=en_US.UTF-8, LANGUAGE=de); QR_LANGUAGE=en
# prüft die englische Oberfläche. Ab 0.1.2 braucht Deutsch den Katalog
# <Ordner der Binary>/share/locale (oder QUERENCIA_RECHNER_LOCALEDIR).
echo "Sprache: LANGUAGE=${QR_LANGUAGE:-de}, Katalog: $( [ -d "$(dirname "$BIN")/share/locale" ] && echo "neben der Binary" || echo "${QUERENCIA_RECHNER_LOCALEDIR:-keiner}")" >"$OUT/sprache.txt"
# Schrift der Sitzung nur lesen und ins Profil schreiben (PT-1); danach läuft
# die Prüfung mit bereinigter Umgebung (env -i): eigenes HOME im Profil, keine
# QUERENCIA_*-Variablen der Sitzung, GIO_USE_VFS=local (kein gvfs/wsdd).
SCHRIFT=$(gsettings get org.gnome.desktop.interface font-name 2>/dev/null | tr -d "'")
mkdir -p "$PROFIL/home" "$PROFIL/config/gtk-4.0"
[ -n "$SCHRIFT" ] && printf '[Settings]\ngtk-font-name=%s\n' "$SCHRIFT" >"$PROFIL/config/gtk-4.0/settings.ini"
LABWC_BIN=${QUERENCIA_LABWC:-$HOME/opt/labwc-0.9.8/bin/labwc}
[ -x "$LABWC_BIN" ] || LABWC_BIN=$(command -v labwc)
env -i HOME="$PROFIL/home" PATH="$PATH" LANG=en_US.UTF-8 LANGUAGE="${QR_LANGUAGE:-de}" \
    GIO_USE_VFS=local QUERENCIA_LABWC="$LABWC_BIN" \
    python3 -I "$E/werkzeug/isoliert.py" "$PROFIL" "$E/werkzeug/$SZ" 300 || true
cp "$PROFIL"/logs/*.log "$OUT/" 2>/dev/null || true
cat "$OUT/ergebnis.txt"
