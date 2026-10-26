#!/bin/bash
# Läuft IN der isolierten labwc-Sitzung (isoliert.py). Sprache von Querencia Rechner 0.1.2 (G2-AH, ab 0.1.7 auch it/es/fr): Fehlertext, Menüs, Info-Dialog, Tooltip in LANGUAGE (de, en, it, es, fr; gui-pruefung.sh QR_LANGUAGE), Katalog in /proc/<pid>/maps. Kopf wie szenario.sh; prüft Querencia Rechner
# mit virtuellen Eingaben (Maus und Tastatur) und Bildschirmfotos; die
# Anzeige wird aus der Debug-Ausgabe des Programms (QUERENCIA_RECHNER_DEBUG)
# gelesen, nicht aus den Bildern.
set -u
. "$HMP04_PROFILE/env.sh"
mkdir -p "$OUT"
E=$OUT/ergebnis.txt; : >"$E"
LOG=$OUT/rechner.log
ok() { echo "PASS $*" | tee -a "$E"; }
nok() { echo "FAIL $*" | tee -a "$E"; }
blocked() { echo "BLOCKED $*" | tee -a "$E"; }
. "$(dirname "$0")/eingabe.sh"
shot() { grim "$OUT/$1.png"; }
running() { kill -0 "$PID" 2>/dev/null; }
W=${R%x*}; H=${R#*x}

# Auflösung des headless-Ausgangs setzen; Zeigerkoordinaten in Pixeln.
"$QA" "$R" >"$OUT/ausgabe.log" 2>&1 && ok "Auflösung $R gesetzt" || blocked "Auflösung $R nicht setzbar"
q "size $W $H" "numlock on" "wait 500"

# ---------------------------------------------------------------- Start
QUERENCIA_RECHNER_DEBUG=1 "$BIN" >"$LOG" 2>"$OUT/rechner.err" &
PID=$!
sleep 3
running && ok "Start ($R)" || { nok "Start"; touch "$HMP04_PROFILE/logs/FERTIG"; exit 0; }
grep -q '^fenster' "$LOG" || sleep 2
shot "01-start"

# Letzte Anzeigezeile: "anzeige<TAB>eingabezeile<TAB>ergebnis<TAB>M|-"
anzeige() { grep '^anzeige' "$LOG" | tail -1 | cut -f2-; }
# erwartet "<eingabezeile>" "<ergebnis>" ["M"]
erwartet() {
    local soll ist
    soll=$(printf '%s\t%s\t%s' "$1" "$2" "${3:--}")
    ist=$(anzeige)
    if [ "$ist" = "$soll" ]; then return 0; fi
    echo "   erwartet: [$soll]" >>"$E"; echo "   ist:      [$ist]" >>"$E"
    return 1
}
pruefe() { # pruefe "<Name>" "<eingabezeile>" "<ergebnis>" ["M"]
    local name=$1; shift
    if erwartet "$@"; then ok "$name"; else nok "$name"; fi
}

# ------------------------------------------------------- Kalibrierung
# Fensterlage aus einer Zeigerbewegung: das Programm meldet, wo der Zeiger
# relativ zum Fenster steht. Fenster laut rc.xml bei (60, 60) plus Rahmen.
q "move 200 200" "wait 300" "move 220 210" "wait 400"
read -r ZX ZY < <(grep '^zeiger' "$LOG" | tail -1 | cut -f2,3)
if [ -n "${ZX:-}" ] && [ -n "${ZY:-}" ]; then
    OX=$((220 - ZX)); OY=$((210 - ZY))
    ok "Kalibrierung: Fensterinhalt beginnt bei ($OX, $OY)"
else
    OX=60; OY=84; blocked "Kalibrierung: keine Zeigermeldung, Annahme ($OX, $OY)"
fi
# Tastenmitte aus der Debug-Ausgabe: knopf <Text> x y w h
mitte() {
    awk -F'\t' -v t="$1" '$1=="knopf" && $2==t {printf "%d %d\n", $3+$5/2, $4+$6/2; exit}' "$LOG"
}
menue_mitte() {
    awk -F'\t' -v i="$1" '$1=="menue" && $2==i {printf "%d %d\n", $3+$5/2, $4+$6/2; exit}' "$LOG"
}
# Erster Eintrag des aufgeklappten Menüs i: unter dem Menükopf (Popover-Rand ~9 px, Eintrag ~24 px hoch)
menue_eintrag1() {
    awk -F'\t' -v i="$1" '$1=="menue" && $2==i {printf "%d %d\n", $3+30, $4+$6+21; exit}' "$LOG"
}
klick() { # klick <Taste> [<Taste> …] – Maus auf die Taste und klicken
    local t x y
    for t in "$@"; do
        read -r x y < <(mitte "$t")
        [ -n "${x:-}" ] || { echo "   keine Position für Taste $t" >>"$E"; continue; }
        q "move $((OX + x)) $((OY + y))" "click" "wait 120"
    done
    q "wait 200"
}

SPRACHE=${LANGUAGE%%:*}
SPRACHE=${SPRACHE%%_*}
case $SPRACHE in
de) FEHLER="Division durch Null"; SCHLIESSEN=s; INFO=i ;;
it) FEHLER="Divisione per zero"; SCHLIESSEN=c; INFO=i ;;
es) FEHLER="División por cero"; SCHLIESSEN=c; INFO=a ;;
fr) FEHLER="Division par zéro"; SCHLIESSEN=f; INFO=p ;;
*) FEHLER="Division by zero"; SCHLIESSEN=c; INFO=a ;;
esac
echo "INFO LANG=$LANG LANGUAGE=$LANGUAGE" >>"$E"
KAT=$(grep -o '[^ ]*querencia-rechner.mo' "/proc/$PID/maps" | head -1)
echo "INFO Katalog: ${KAT:-keiner}" >>"$E"
case $SPRACHE in
de|it|es|fr) [ -n "$KAT" ] && ok "$SPRACHE: Katalog eingeblendet" || nok "$SPRACHE: kein Katalog eingeblendet" ;;
*) [ -z "$KAT" ] && ok "$SPRACHE: kein Katalog eingeblendet" || nok "$SPRACHE: Katalog eingeblendet" ;;
esac
grep -q "^menue" "$LOG" && ok "Menüleiste gemeldet" || nok "keine Menüleiste"

# Fehlertext in der Sprache
q "key Escape" "type 5/0=" "wait 400"
pruefe "Fehlertext: $FEHLER" "5 ÷ 0" "$FEHLER"
shot "s1-fehler-$SPRACHE"

# Längste Fehlertexte (ab 0.1.7): Anzeige auf den Fotos prüfen (die Anzeige
# kürzt zu lange Texte vorne mit drei Punkten)
q "key Escape" "type 1000000000000qqqqq" "wait 400"; shot "s1b-ueberlauf-$SPRACHE"
echo "INFO Anzeige Überlauf: $(anzeige)" >>"$E"
q "key Escape" "type 4vs" "wait 400"; shot "s1c-wurzel-$SPRACHE"
echo "INFO Anzeige Wurzel: $(anzeige)" >>"$E"
q "key Escape" "type (1+2=" "wait 400"; shot "s1d-unvollstaendig-$SPRACHE"
echo "INFO Anzeige unvollständig: $(anzeige)" >>"$E"
q "key Escape" "wait 300"

# Tooltip der Taste √
read -r x y < <(mitte "√")
q "move $((OX + x)) $((OY + y))" "wait 1800"
shot "s2-tooltip-$SPRACHE"
q "move $((OX + 5)) $((OY + 5))" "wait 400"

# Menü 0 (Rechner/Calculator) und 1 (Hilfe/Help) aufklappen
for i in 0 1; do
    read -r MX MY < <(menue_mitte $i)
    q "move $((OX + MX)) $((OY + MY))" "click" "wait 1000"
    shot "s3-menue-$i-$SPRACHE"
    q "key Escape" "wait 500"
done
running && ok "Menüs geöffnet (Bilder s3)" || nok "Absturz beim Öffnen der Menüs"

# Info-Dialog über Hilfe, Zugriffstaste von Info (seit 0.1.6 ist der erste
# Eintrag Online-Hilfe, der den Browser öffnen würde), schließen mit
# Alt+<Zugriffstaste der Sprache>
read -r MX MY < <(menue_mitte 1)
q "move $((OX + MX)) $((OY + MY))" "click" "wait 1000" "key $INFO" "wait 1200"
shot "s4-info-$SPRACHE"
q "chord alt $SCHLIESSEN" "wait 800" "type 3+3=" "wait 400"
if running && erwartet "3 + 3 =" "6"; then ok "Info-Dialog per Alt+$SCHLIESSEN geschlossen ($SPRACHE), Tastatur danach im Hauptfenster"
else nok "Info-Dialog Alt+$SCHLIESSEN"; fi

q "chord ctrl q" "wait 1500"
if running; then nok "Strg+Q beendet nicht"; kill "$PID"; else ok "Strg+Q beendet das Programm"; fi
echo "Ergebnis $R $SPRACHE: $(grep -c '^PASS' "$E") PASS, $(grep -c '^FAIL' "$E") FAIL, $(grep -c '^BLOCKED' "$E") BLOCKED" | tee -a "$E"
touch "$HMP04_PROFILE/logs/FERTIG"
