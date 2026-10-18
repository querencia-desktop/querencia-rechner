#!/bin/bash
# Läuft IN der isolierten labwc-Sitzung (isoliert.py). Prüft Querencia Rechner
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
q "size $W $H" "wait 500"

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
klick() { # klick <Taste> [<Taste> …] – Maus auf die Taste und klicken
    local t x y
    for t in "$@"; do
        read -r x y < <(mitte "$t")
        [ -n "${x:-}" ] || { echo "   keine Position für Taste $t" >>"$E"; continue; }
        q "move $((OX + x)) $((OY + y))" "click" "wait 120"
    done
    q "wait 200"
}
klick 7
pruefe "Maus: Taste 7 trifft (Kalibrierung bestätigt)" "7" "7"
[ "$(grep -c '^knopf' "$LOG")" = 30 ] && ok "30 Tasten gemeldet" || nok "Tastenzahl: $(grep -c '^knopf' "$LOG")"

# --------------------------------------------- Grundrechnung per Maus
klick AC 1 2 + 7 × "(" 3 − 1 ")"
pruefe "Maus: Ausdruck 12 + 7 × (3 − 1)" "12 + 7 × (3 − 1)" "1"
shot "02-maus-ausdruck"
klick =
pruefe "Maus: 12 + 7 × (3 − 1) = 26 (Punkt vor Strich, Klammern)" "12 + 7 × (3 − 1) =" "26"
shot "03-maus-ergebnis"
klick AC 1 "," 5 × 4 =
pruefe "Maus: 1,5 × 4 = 6 (Komma)" "1,5 × 4 =" "6"
klick AC 8 1 √
pruefe "Maus: √81 = 9" "9" "9"
klick AC 1 2 x²
pruefe "Maus: 12² = 144" "144" "144"
klick AC 5 ±
pruefe "Maus: ±5 = −5" "−5" "−5"
klick AC 4 1/x
pruefe "Maus: 1/4 = 0,25" "0,25" "0,25"
klick AC 1 2 3 ←
pruefe "Maus: Rücktaste 123 → 12" "12" "12"
klick AC 1 2 + 7 C
pruefe "Maus: C löscht die aktuelle Zahl" "12 +" "12"
klick AC
pruefe "Maus: AC löscht alles" "" "0"

# ----------------------------------------- Grundrechnung per Tastatur
q "key Escape" "type 6*7" "key Return" "wait 300"
pruefe "Tastatur: 6*7 Enter = 42" "6 × 7 =" "42"
shot "04-tastatur-ergebnis"
q "key Escape" "type 7-10=" "wait 300"
pruefe "Tastatur: 7-10= = −3 (Gleichtaste)" "7 − 10 =" "−3"
q "key Escape" "type 2.5+0,5=" "wait 300"
pruefe "Tastatur: Punkt und Komma als Dezimaltrenner" "2,5 + 0,5 =" "3"
q "key Escape" "key KP_1" "key KP_0" "key KP_0" "key KP_Divide" "key KP_8" "key KP_Enter" "wait 300"
pruefe "Ziffernblock: 100 ÷ 8 Enter = 12,5" "100 ÷ 8 =" "12,5"
q "key Escape" "key KP_1" "key KP_Decimal" "key KP_5" "key KP_Multiply" "key KP_2" "key KP_Subtract" "key KP_1" "key KP_Add" "key KP_4" "key KP_Enter" "wait 300"
pruefe "Ziffernblock: 1,5 × 2 − 1 + 4 = 6 (KP_Decimal, alle Operatoren)" "1,5 × 2 − 1 + 4 =" "6"
q "key Escape" "type 123" "key BackSpace" "wait 200"
pruefe "Tastatur: Rücktaste" "12" "12"
q "key Delete" "wait 200"
pruefe "Tastatur: Entf = C" "" "0"
q "type 9+9" "key Escape" "wait 200"
pruefe "Tastatur: Escape = AC" "" "0"
q "type 81s" "wait 200"
pruefe "Tastatur: s = Wurzel (√81 = 9)" "9" "9"
q "key Escape" "type 12q" "wait 200"
pruefe "Tastatur: q = Quadrat" "144" "144"
q "key Escape" "type 4r" "wait 200"
pruefe "Tastatur: r = Kehrwert" "0,25" "0,25"
q "key Escape" "type 5v" "wait 200"
pruefe "Tastatur: v = Vorzeichen" "−5" "−5"

# ------------------------------------------------------------ Klammern
q "key Escape" "type (2+3)*4=" "wait 300"
pruefe "Klammern: (2+3)*4 = 20" "(2 + 3) × 4 =" "20"
q "key Escape" "type 2*(3+4=" "wait 300"
pruefe "Klammern: offene Klammer wird bei = geschlossen" "2 × (3 + 4) =" "14"
q "key Escape" "type ((1+2)*(3+4))=" "wait 300"
pruefe "Klammern: verschachtelt = 21" "((1 + 2) × (3 + 4)) =" "21"
shot "05-klammern"

# ------------------------------------------------------------- Prozent
q "key Escape" "type 200+10%=" "wait 300"
pruefe "Prozent: 200 + 10 % = 220" "200 + 10% =" "220"
q "key Escape" "type 200*10%=" "wait 300"
pruefe "Prozent: 200 × 10 % = 20" "200 × 10% =" "20"
klick AC 5 0 % =
pruefe "Prozent per Maus: 50 % = 0,5" "50% =" "0,5"
shot "06-prozent"

# ------------------------------------------------------------ Speicher
klick AC 6 × 7 = M+
pruefe "Speicher: M+ nach 6 × 7 zeigt Marke M" "6 × 7 =" "42" "M"
shot "07-speicher-belegt"
klick AC 2 × MR =
pruefe "Speicher: 2 × MR = 84" "2 × 42 =" "84" "M"
klick AC 1 0 M− AC MR
pruefe "Speicher: M− 10, MR = 32" "32" "32" "M"
klick MC
pruefe "Speicher: MC löscht die Marke" "32" "32"
klick AC MR
pruefe "Speicher: MR ohne Speicher tut nichts" "" "0"

# -------------------------------------------------- Division durch Null
q "key Escape" "type 5/0=" "wait 300"
pruefe "Division durch Null als Text" "5 ÷ 0 =" "Division durch Null"
shot "08-division-durch-null"
running && ok "Programm läuft nach Division durch Null weiter" || nok "Absturz bei Division durch Null"
q "type 7" "wait 200"
pruefe "Nach dem Fehler beginnt eine Ziffer neu" "7" "7"
q "key Escape" "type 1000000000000qqqqq" "wait 300"
pruefe "Überlauf als Text (x² wiederholt)" "1e+12" "Überlauf"
shot "09-ueberlauf"
klick AC 4 ± √
pruefe "Wurzel aus negativer Zahl als Text" "−4" "Wurzel aus negativer Zahl"
klick AC 0 1/x
pruefe "Kehrwert von 0 als Text" "0" "Division durch Null"
running && ok "Programm läuft nach allen Fehlern weiter" || nok "Absturz nach Fehlern"

# --------------------------------------------------- Kopieren/Einfügen
q "key Escape" "type 6*7=" "chord ctrl c" "wait 600"
grep -q "^kopiert	42$" "$LOG" && ok "Strg+C: Ergebnis 42 kopiert (Programm)" || nok "Strg+C ohne Kopiermeldung"
ZA=$(wl-paste --no-newline 2>/dev/null || echo "?")
[ "$ZA" = "42" ] && ok "Zwischenablage enthält 42 (wl-paste)" || nok "Zwischenablage: [$ZA]"
q "key Escape" "type 3*" "chord ctrl v" "wait 800"
pruefe "Strg+V: 42 eingefügt nach 3 ×" "3 × 42" "42"
q "key Return" "wait 300"
pruefe "Eingefügte Zahl rechnet: 3 × 42 = 126" "3 × 42 =" "126"
shot "10-einfuegen"
printf '12,5' | wl-copy 2>/dev/null
q "key Escape" "chord ctrl v" "wait 800"
pruefe "Strg+V: Fremdtext „12,5“ mit Komma" "12,5" "12,5"
printf '0.75' | wl-copy 2>/dev/null
q "key Escape" "chord ctrl v" "wait 800"
pruefe "Strg+V: Fremdtext „0.75“ mit Punkt" "0,75" "0,75"
printf 'Hallo' | wl-copy 2>/dev/null
q "key Escape" "type 5" "chord ctrl v" "wait 800"
grep -q "^eingefuegt	nein	Hallo$" "$LOG" && erwartet "5" "5" && ok "Strg+V: Text ohne Zahl wird ignoriert" || nok "Strg+V mit Text ohne Zahl"
# Menü Rechner → Kopieren per Maus (Menükopf 0), Eintrag per Tastatur
q "key Escape" "type 7-10=" "wait 200"
read -r MX MY < <(menue_mitte 0)
if [ -n "${MX:-}" ]; then
    q "move $((OX + MX)) $((OY + MY))" "click" "wait 1000"
    shot "11-menue-rechner"
    q "key Down" "wait 200" "key Return" "wait 600"
    ZA=$(wl-paste --no-newline 2>/dev/null || echo "?")
    [ "$ZA" = "-3" ] && ok "Menü Rechner → Kopieren: −3 als „-3“ in der Zwischenablage" || nok "Menü Kopieren: [$ZA]"
else
    blocked "Menüposition nicht gemeldet"
fi
q "key Escape" "wait 300"

# ---------------------------------------------- Info-Dialog mit Fokus
read -r MX MY < <(menue_mitte 1)
if [ -n "${MX:-}" ]; then
    q "move $((OX + MX)) $((OY + MY))" "click" "wait 1000"
    shot "12-menue-hilfe"
    q "key Down" "wait 200" "key Return" "wait 1200"
    shot "13-info-dialog"
    running && ok "Info-Dialog geöffnet (Bild 13)" || nok "Absturz beim Öffnen des Info-Dialogs"
    # Schließen-Knopf hat den Fokus: Eingabetaste schließt; danach Tasten im Hauptfenster
    q "key Return" "wait 800" "type 1+1=" "wait 400"
    sleep 0.5
    if running && erwartet "1 + 1 =" "2"; then ok "Info-Dialog per Eingabetaste (Fokus) geschlossen, danach Tastatur ohne Absturz"
    else nok "Info-Dialog schließen per Eingabetaste / Folgetasten"; fi
    shot "14-nach-info"
    # Zweiter Durchgang mit Escape
    q "move $((OX + MX)) $((OY + MY))" "click" "wait 900" "key Down" "wait 200" "key Return" "wait 1000"
    q "key Escape" "wait 800" "type 2+2=" "wait 400"
    sleep 0.5
    if running && erwartet "2 + 2 =" "4"; then ok "Info-Dialog per Escape geschlossen, danach Tastatur ohne Absturz"
    else nok "Info-Dialog schließen per Escape / Folgetasten"; fi
else
    blocked "Menüposition Hilfe nicht gemeldet"
fi

# ---------------------------------------------------- Zweite Instanz
"$BIN" >>"$OUT/rechner2.log" 2>&1 &
PID2=$!
sleep 2
if running && ! kill -0 "$PID2" 2>/dev/null; then ok "Zweiter Aufruf holt das vorhandene Fenster nach vorn (eine Instanz)"
else nok "Zweite Instanz"; kill "$PID2" 2>/dev/null; fi

# --------------------------------------------------------- Beenden
q "chord ctrl q" "wait 1500"
if running; then nok "Strg+Q beendet nicht"; kill "$PID"; else
    wait "$PID"; EC=$?
    [ "$EC" = 0 ] && ok "Strg+Q beendet das Programm (Exit 0)" || nok "Exit $EC beim Beenden"
fi
grep -i -E 'critical|warning|error' "$OUT/rechner.err" | head -20 >"$OUT/gtk-meldungen.txt"
[ -s "$OUT/gtk-meldungen.txt" ] && echo "HINW GTK-Meldungen: $(wc -l <"$OUT/gtk-meldungen.txt")" | tee -a "$E"
echo "Ergebnis $R: $(grep -c '^PASS' "$E") PASS, $(grep -c '^FAIL' "$E") FAIL, $(grep -c '^BLOCKED' "$E") BLOCKED" | tee -a "$E"
touch "$HMP04_PROFILE/logs/FERTIG"
