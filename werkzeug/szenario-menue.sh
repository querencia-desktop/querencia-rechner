#!/bin/bash
# Diagnose: Tastaturnavigation in den Menüs (läuft in isoliert.py).
set -u
. "$HMP04_PROFILE/env.sh"
mkdir -p "$OUT"; E=$OUT/ergebnis.txt; : >"$E"
. "$(dirname "$0")/eingabe.sh"
shot() { grim "$OUT/$1.png"; }
"$QA" "$R" >/dev/null 2>&1; W=${R%x*}; H=${R#*x}
q "size $W $H" "numlock on" "wait 500"
QUERENCIA_RECHNER_DEBUG=1 "$BIN" >"$OUT/rechner.log" 2>"$OUT/rechner.err" &
PID=$!; sleep 4
OX=61; OY=87
q "type 7-10=" "wait 300"
# A: F10, Down, Return (erstes Menü: Kopieren) -> Zwischenablage -3?
printf 'leer' | wl-copy
q "key F10" "wait 1000" "key Down" "wait 500" "key Return" "wait 1000"; shot g1-f10-down-return
echo "A F10 Down Return: wl-paste=[$(wl-paste --no-newline)]" >>"$E"
q "key Escape" "wait 500"
# B: Alt+H, Down, Return (Hilfe: Info)
q "chord alt h" "wait 1000" "key Down" "wait 500" "key Return" "wait 1200"; shot g2-alt-h-return
q "key Escape" "wait 800"
# C: Maus auf Hilfe, Down, Return
q "move $((OX+104)) $((OY+13))" "click" "wait 1000" "key Down" "wait 500" "key Return" "wait 1200"; shot g3-maus-down-return
q "key Escape" "wait 800"
# D: Maus auf Rechner, Down, Return
printf 'leer' | wl-copy
q "move $((OX+40)) $((OY+13))" "click" "wait 1000" "key Down" "wait 500" "key Return" "wait 1000"; shot g4-maus-rechner-return
echo "D Maus Rechner Down Return: wl-paste=[$(wl-paste --no-newline)]" >>"$E"
q "key Escape" "wait 800"
grep '^taste' "$OUT/rechner.log" >>"$E"
q "chord ctrl q" "wait 1000"
touch "$HMP04_PROFILE/logs/FERTIG"
