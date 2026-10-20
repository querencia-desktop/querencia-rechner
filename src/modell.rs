//! Bedienmodell des Rechners: Tasten verändern einen Ausdruck aus Tokens,
//! der Rechenkern wertet ihn aus. Ohne GTK, mit Unit-Tests.
//!
//! **Anzeige** (zwei Zeilen wie gcalctool im einfachen Modus mit Ausdruck):
//! die *Eingabezeile* zeigt den ganzen Ausdruck (`12 + 7 × (3 − 1)`, nach
//! Gleich mit ` =`), die *Ergebniszeile* die gerade getippte Zahl, nach Gleich
//! das Ergebnis, im Fehlerfall den Fehlertext. Dazu die Marke „M“, solange
//! der Speicher belegt ist.
//!
//! **Regeln** (klassisches Verhalten, Vorbild gcalctool 5.30):
//! - Nach Gleich beginnt eine Ziffer, ein Komma oder `(` einen neuen Ausdruck;
//!   ein Operator rechnet mit dem Ergebnis weiter; √, x², ±, 1/x und %
//!   wirken auf das Ergebnis.
//! - Die Funktionen √, x², 1/x wirken sofort auf die aktuelle Zahl (oder auf
//!   eine gerade geschlossene Klammer, die dazu ausgewertet wird). Ohne Zahl
//!   wirken sie auf 0.
//! - ± kehrt das Vorzeichen der getippten Zahl um; ohne Zahl setzt (oder
//!   entfernt) es ein Vorzeichen-Minus vor der nächsten Zahl.
//! - Ein Operator nach einem Operator ersetzt ihn; Minus nach einem Operator
//!   ist ein Vorzeichen (`5 × −3`), zweimal Minus hebt sich auf. Vor der
//!   ersten Zahl und nach `(` ist nur Minus erlaubt.
//! - `(` nach einer Zahl oder `)` fügt stillschweigend „×“ ein; `)` nur, wenn
//!   eine Klammer offen ist. Bei Gleich werden offene Klammern geschlossen
//!   und ein nachgestellter Operator entfernt.
//! - Rücktaste löscht die letzte Ziffer bzw. das letzte Token; nach Gleich
//!   macht sie den Ausdruck wieder bearbeitbar. C löscht die aktuelle Zahl
//!   (nach Gleich oder im Fehlerfall alles), AC löscht alles. Der Speicher
//!   bleibt bei C und AC erhalten.
//! - M+ und M− rechnen mit dem Anzeigewert (Ergebnis oder aktuelle Zahl),
//!   MR fügt den Speicherwert als Zahl ein, MC leert den Speicher.
//! - Kopieren liefert den Anzeigewert (Ergebnis oder aktuelle Zahl) mit
//!   Dezimalkomma und ASCII-Minus, gerundet wie die Anzeige; Einfügen
//!   übernimmt eine Zahl mit Komma oder Punkt als neue aktuelle Zahl.
//! - Eine Ziffer direkt nach einem Funktionsergebnis, MR oder Einfügen
//!   beginnt keine neue Zahl, sondern ergänzt „ד (`2 √ 3` = √2 × 3). Das
//!   Vorbild hat diese Tasten im einfachen Modus nicht; im erweiterten Modus
//!   von gcalctool 5.30 sind √, x², x⁻¹ Zeichen im Ausdruck, und nur nach
//!   Gleich beginnt eine Ziffer neu (`display.c`, `FN_TEXT`).
//!
//! **Genauigkeit** (seit 0.1.1): Ergebnisse, Funktionsergebnisse, Speicher
//! und eingefügte Zahlen bleiben als [`Token::Wert`] mit voller
//! `f64`-Genauigkeit im Ausdruck; gerundet wird nur für Anzeige und
//! Zwischenablage (`2 √ x²` = 2, danach `1/x` = 0,5).

use crate::rechenkern::{self as kern, Fehler, Op, Token};

/// Alle Tasten des Rechners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taste {
    Ziffer(u8),
    Komma,
    Op(Op),
    Auf,
    Zu,
    Prozent,
    Gleich,
    Wurzel,
    Quadrat,
    Vorzeichen,
    Kehrwert,
    Ruecktaste,
    C,
    AC,
    MPlus,
    MMinus,
    MR,
    MC,
}

/// Was das Fenster zeigt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anzeige {
    pub eingabezeile: String,
    pub ergebnis: String,
    pub fehler: bool,
    pub speicher_belegt: bool,
}

/// Höchstzahl der Zeichen einer getippten Zahl (ohne Vorzeichen).
const MAX_ZIFFERN: usize = 15;

#[derive(Debug, Default)]
pub struct Modell {
    tokens: Vec<Token>,
    /// Das letzte Token ist eine Zahl, an die Ziffern angehängt werden.
    tippt: bool,
    ergebnis: Option<Result<f64, Fehler>>,
    speicher: Option<f64>,
}

impl Modell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn anzeige(&self) -> Anzeige {
        let mut eingabezeile = kern::ausdruck_text(&self.tokens);
        let (ergebnis, fehler) = match self.ergebnis {
            Some(Ok(x)) => {
                eingabezeile.push_str(" =");
                (kern::formatieren(x), false)
            }
            Some(Err(f)) => (f.text(), true),
            None => (self.aktuelle_zahl_text(), false),
        };
        Anzeige {
            eingabezeile,
            ergebnis: ergebnis.replace('-', "−"),
            fehler,
            speicher_belegt: self.speicher.is_some(),
        }
    }

    /// Wert für die Zwischenablage (ASCII-Minus, Dezimalkomma); im Fehlerfall
    /// oder ohne Zahl nichts.
    pub fn kopierwert(&self) -> Option<String> {
        match self.ergebnis {
            Some(Ok(x)) => Some(kern::formatieren(x)),
            Some(Err(_)) => None,
            None => match self.tokens.last() {
                Some(Token::Zahl(z)) => Some(z.clone()),
                Some(Token::Wert(x)) => Some(kern::formatieren(*x)),
                _ => None,
            },
        }
    }

    /// Fügt eine Zahl aus Text ein; `false`, wenn der Text keine Zahl ist.
    pub fn einfuegen(&mut self, text: &str) -> bool {
        let Some(x) = kern::zahl_lesen(text) else {
            return false;
        };
        self.zahl_einsetzen(x);
        true
    }

    pub fn druecken(&mut self, taste: Taste) {
        match taste {
            Taste::Ziffer(d) => self.ziffer(d),
            Taste::Komma => self.komma(),
            Taste::Op(op) => self.operator(op),
            Taste::Auf => self.auf(),
            Taste::Zu => self.zu(),
            Taste::Prozent => self.prozent(),
            Taste::Gleich => self.gleich(),
            Taste::Wurzel => self.funktion(kern::wurzel),
            Taste::Quadrat => self.funktion(kern::quadrat),
            Taste::Kehrwert => self.funktion(kern::kehrwert),
            Taste::Vorzeichen => self.vorzeichen(),
            Taste::Ruecktaste => self.ruecktaste(),
            Taste::C => self.c(),
            Taste::AC => self.ac(),
            Taste::MPlus => self.speicher_rechnen(1.0),
            Taste::MMinus => self.speicher_rechnen(-1.0),
            Taste::MR => {
                if let Some(m) = self.speicher {
                    self.zahl_einsetzen(m);
                }
            }
            Taste::MC => self.speicher = None,
        }
    }

    // ------------------------------------------------------------ Zustand

    fn aktuelle_zahl_text(&self) -> String {
        self.tokens
            .iter()
            .rev()
            .find_map(|t| match t {
                Token::Zahl(z) => Some(z.clone()),
                Token::Wert(x) => Some(kern::formatieren(*x)),
                _ => None,
            })
            .unwrap_or_else(|| "0".to_string())
    }

    /// Wert der aktuellen Zahl (getippt oder berechnet), 0 ohne Zahl.
    fn aktuelle_zahl(&self) -> f64 {
        self.tokens
            .iter()
            .rev()
            .find_map(|t| match t {
                Token::Zahl(z) => Some(kern::zahl_lesen(z).unwrap_or(0.0)),
                Token::Wert(x) => Some(*x),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    /// Anzeigewert als Zahl: Ergebnis oder aktuelle Zahl (0 ohne Zahl);
    /// im Fehlerfall nichts.
    fn anzeigewert(&self) -> Option<f64> {
        match self.ergebnis {
            Some(Ok(x)) => Some(x),
            Some(Err(_)) => None,
            None => Some(self.aktuelle_zahl()),
        }
    }

    fn offene_klammern(&self) -> usize {
        let auf = self.tokens.iter().filter(|t| **t == Token::Auf).count();
        let zu = self.tokens.iter().filter(|t| **t == Token::Zu).count();
        auf.saturating_sub(zu)
    }

    /// Nach Gleich oder Fehler: Ausdruck neu beginnen. Mit `weiter` wird ein
    /// Ergebnis als erste Zahl übernommen (Operator nach Gleich).
    fn frisch(&mut self, weiter: bool) {
        match self.ergebnis.take() {
            Some(Ok(x)) if weiter => {
                self.tokens = vec![Token::Wert(x)];
            }
            _ => self.tokens.clear(),
        }
        self.tippt = false;
    }

    /// Steht am Ende etwas, hinter dem eine Zahl nur mit „×“ folgen kann?
    fn braucht_mal(&self) -> bool {
        matches!(self.tokens.last(), Some(Token::Zu | Token::Prozent))
    }

    /// Letzte Zahl (zum Tippen oder für Funktionen) – nur wenn das letzte
    /// Token eine Zahl ist.
    fn letzte_zahl_mut(&mut self) -> Option<&mut String> {
        match self.tokens.last_mut() {
            Some(Token::Zahl(z)) => Some(z),
            _ => None,
        }
    }

    /// Endet der Ausdruck mit einer Zahl (getippt oder berechnet)?
    fn endet_mit_zahl(&self) -> bool {
        matches!(self.tokens.last(), Some(Token::Zahl(_) | Token::Wert(_)))
    }

    /// Setzt `x` als neue aktuelle Zahl mit voller Genauigkeit (MR, Einfügen).
    fn zahl_einsetzen(&mut self, x: f64) {
        if self.ergebnis.is_some() {
            self.frisch(false);
        }
        if self.tippt || self.endet_mit_zahl() {
            self.tokens.pop();
        } else if self.braucht_mal() {
            self.tokens.push(Token::Op(Op::Mal));
        }
        self.tokens.push(Token::Wert(x));
        self.tippt = false;
    }

    // -------------------------------------------------------------- Tasten

    fn ziffer(&mut self, d: u8) {
        if self.ergebnis.is_some() {
            self.frisch(false);
        }
        if self.tippt {
            let z = self.letzte_zahl_mut().expect("tippt ⇒ Zahl");
            let stellen = z.trim_start_matches('-').len();
            if *z == "0" {
                *z = d.to_string();
            } else if *z == "-0" {
                *z = format!("-{d}");
            } else if stellen < MAX_ZIFFERN {
                z.push((b'0' + d) as char);
            }
            return;
        }
        if self.endet_mit_zahl() {
            // Zahl aus MR/Einfügen/Funktion: neue Zahl ersetzt sie nicht,
            // sondern wird ohne Operator als ungültig gewertet – deshalb „×“.
            self.tokens.push(Token::Op(Op::Mal));
        } else if self.braucht_mal() {
            self.tokens.push(Token::Op(Op::Mal));
        }
        self.tokens.push(Token::Zahl(d.to_string()));
        self.tippt = true;
    }

    fn komma(&mut self) {
        if self.ergebnis.is_some() {
            self.frisch(false);
        }
        if self.tippt {
            let z = self.letzte_zahl_mut().expect("tippt ⇒ Zahl");
            if !z.contains(',') && z.trim_start_matches('-').len() < MAX_ZIFFERN {
                z.push(',');
            }
            return;
        }
        if self.endet_mit_zahl() || self.braucht_mal() {
            self.tokens.push(Token::Op(Op::Mal));
        }
        self.tokens.push(Token::Zahl("0,".to_string()));
        self.tippt = true;
    }

    fn operator(&mut self, op: Op) {
        if self.ergebnis.is_some() {
            self.frisch(true);
            if self.tokens.is_empty() {
                return; // nach einem Fehler gibt es nichts zum Weiterrechnen
            }
        }
        self.tippt = false;
        match self.tokens.last() {
            None | Some(Token::Auf) => {
                // Nur ein Vorzeichen-Minus darf hier stehen.
                if op == Op::Minus {
                    self.tokens.push(Token::Op(Op::Minus));
                }
            }
            Some(Token::Op(_)) => {
                let n = self.tokens.len();
                if self.ist_vorzeichen(n - 1) {
                    // Hinter einem Vorzeichen-Minus: noch ein Minus hebt es
                    // auf; ein anderer Operator entfernt es und ersetzt den
                    // Operator davor (am Anfang: wird ignoriert).
                    self.tokens.pop();
                    if op != Op::Minus && n >= 2 {
                        self.tokens[n - 2] = Token::Op(op);
                    }
                } else if op == Op::Minus {
                    self.tokens.push(Token::Op(Op::Minus)); // „× −“ = Vorzeichen
                } else {
                    self.tokens[n - 1] = Token::Op(op);
                }
            }
            Some(Token::Zahl(_) | Token::Wert(_) | Token::Zu | Token::Prozent) => {
                self.tokens.push(Token::Op(op))
            }
        }
    }

    fn auf(&mut self) {
        if self.ergebnis.is_some() {
            self.frisch(false);
        }
        if self.endet_mit_zahl() || self.braucht_mal() {
            self.tokens.push(Token::Op(Op::Mal));
        }
        self.tokens.push(Token::Auf);
        self.tippt = false;
    }

    fn zu(&mut self) {
        if self.ergebnis.is_some() {
            return;
        }
        if self.offene_klammern() == 0 {
            return;
        }
        if matches!(
            self.tokens.last(),
            Some(Token::Zahl(_) | Token::Wert(_) | Token::Zu | Token::Prozent)
        ) {
            self.tokens.push(Token::Zu);
            self.tippt = false;
        }
    }

    fn prozent(&mut self) {
        if self.ergebnis.is_some() {
            self.frisch(true);
        }
        if matches!(
            self.tokens.last(),
            Some(Token::Zahl(_) | Token::Wert(_) | Token::Zu)
        ) {
            self.tokens.push(Token::Prozent);
            self.tippt = false;
        }
    }

    fn gleich(&mut self) {
        if self.ergebnis.is_some() || self.tokens.is_empty() {
            return;
        }
        // Nachgestellte Operatoren entfernen, offene Klammern schließen.
        while matches!(self.tokens.last(), Some(Token::Op(_) | Token::Auf)) {
            self.tokens.pop();
        }
        if self.tokens.is_empty() {
            return;
        }
        for _ in 0..self.offene_klammern() {
            self.tokens.push(Token::Zu);
        }
        self.tippt = false;
        self.ergebnis = Some(kern::auswerten(&self.tokens));
    }

    /// √, x², 1/x: sofort auf Ergebnis, aktuelle Zahl oder geschlossene
    /// Klammer anwenden; das Ergebnis bleibt ungerundet im Ausdruck.
    fn funktion(&mut self, f: fn(f64) -> Result<f64, Fehler>) {
        if self.ergebnis.is_some() {
            self.frisch(true);
            if self.tokens.is_empty() {
                return;
            }
        }
        self.tippt = false;
        let wert = match self.tokens.last() {
            Some(Token::Zahl(z)) => {
                let x = kern::zahl_lesen(z).unwrap_or(0.0);
                self.tokens.pop();
                x
            }
            Some(Token::Wert(x)) => {
                let x = *x;
                self.tokens.pop();
                x
            }
            Some(Token::Zu) => {
                let start = self.klammer_anfang();
                let gruppe: Vec<Token> = self.tokens.drain(start..).collect();
                match kern::auswerten(&gruppe) {
                    Ok(x) => x,
                    Err(e) => {
                        self.tokens = gruppe;
                        self.ergebnis = Some(Err(e));
                        return;
                    }
                }
            }
            _ => {
                if self.braucht_mal() {
                    self.tokens.push(Token::Op(Op::Mal));
                }
                0.0
            }
        };
        match f(wert) {
            Ok(x) => self.tokens.push(Token::Wert(x)),
            Err(e) => {
                self.tokens.push(Token::Wert(wert));
                self.ergebnis = Some(Err(e));
            }
        }
    }

    /// Index der „(“, die zur letzten „)“ gehört.
    fn klammer_anfang(&self) -> usize {
        let mut tiefe = 0usize;
        for (i, t) in self.tokens.iter().enumerate().rev() {
            match t {
                Token::Zu => tiefe += 1,
                Token::Auf => {
                    tiefe -= 1;
                    if tiefe == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        0
    }

    fn vorzeichen(&mut self) {
        if self.ergebnis.is_some() {
            self.frisch(true);
            if self.tokens.is_empty() {
                return;
            }
        }
        match self.tokens.last() {
            Some(Token::Zahl(_)) => {
                let z = self.letzte_zahl_mut().expect("Zahl");
                if let Some(rest) = z.strip_prefix('-') {
                    *z = rest.to_string();
                } else {
                    z.insert(0, '-');
                }
            }
            Some(Token::Wert(x)) => {
                let x = -*x;
                self.tokens.pop();
                self.tokens.push(Token::Wert(x));
            }
            Some(Token::Zu) => {
                // Geschlossene Klammer: Vorzeichen-Minus davor setzen/entfernen.
                let start = self.klammer_anfang();
                if start > 0
                    && self.tokens[start - 1] == Token::Op(Op::Minus)
                    && self.ist_vorzeichen(start - 1)
                {
                    self.tokens.remove(start - 1);
                } else {
                    self.tokens.insert(start, Token::Op(Op::Minus));
                }
            }
            Some(Token::Op(Op::Minus)) if self.ist_vorzeichen(self.tokens.len() - 1) => {
                self.tokens.pop();
            }
            Some(Token::Prozent) => {}
            _ => {
                if self.braucht_mal() {
                    self.tokens.push(Token::Op(Op::Mal));
                }
                self.tokens.push(Token::Op(Op::Minus));
                self.tippt = false;
            }
        }
    }

    /// Ist das Minus an Position `i` ein Vorzeichen (am Anfang, nach „(“
    /// oder nach einem Operator)?
    fn ist_vorzeichen(&self, i: usize) -> bool {
        self.tokens[i] == Token::Op(Op::Minus)
            && (i == 0 || matches!(self.tokens[i - 1], Token::Op(_) | Token::Auf))
    }

    fn ruecktaste(&mut self) {
        match self.ergebnis {
            Some(Ok(_)) => {
                // Ausdruck wieder bearbeitbar machen.
                self.ergebnis = None;
                self.tippt = false;
                return;
            }
            Some(Err(_)) => {
                self.ac();
                return;
            }
            None => {}
        }
        if self.tippt {
            let z = self.letzte_zahl_mut().expect("tippt ⇒ Zahl");
            z.pop();
            if z.is_empty() || z == "-" {
                self.tokens.pop();
                self.tippt = false;
            }
            return;
        }
        self.tokens.pop();
        self.tippt = matches!(self.tokens.last(), Some(Token::Zahl(_)));
    }

    fn c(&mut self) {
        if self.ergebnis.is_some() {
            self.ac();
            return;
        }
        if self.endet_mit_zahl() {
            self.tokens.pop();
        }
        self.tippt = false;
    }

    fn ac(&mut self) {
        self.tokens.clear();
        self.tippt = false;
        self.ergebnis = None;
    }

    fn speicher_rechnen(&mut self, faktor: f64) {
        let Some(wert) = self.anzeigewert() else {
            return;
        };
        let alt = self.speicher.unwrap_or(0.0);
        // Volle Genauigkeit; Auslöschung (0,1 + 0,2 − 0,3) ergibt 0.
        let neu = kern::summe(alt, faktor * wert).unwrap_or(alt + faktor * wert);
        self.speicher = Some(neu);
        // Wie bei gcalctool: der Anzeigewert ist danach abgeschlossen.
        self.tippt = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tastenfolge als Kurzschrift: Ziffern, `,` `+ - * /` `( ) %` `=`,
    /// `w` Wurzel, `q` Quadrat, `k` Kehrwert, `v` Vorzeichen, `<` Rücktaste,
    /// `c` C, `a` AC, `M` M+, `m` M−, `r` MR, `x` MC.
    fn tippen(m: &mut Modell, folge: &str) {
        for c in folge.chars() {
            let t = match c {
                '0'..='9' => Taste::Ziffer(c as u8 - b'0'),
                ',' => Taste::Komma,
                '+' => Taste::Op(Op::Plus),
                '-' => Taste::Op(Op::Minus),
                '*' => Taste::Op(Op::Mal),
                '/' => Taste::Op(Op::Geteilt),
                '(' => Taste::Auf,
                ')' => Taste::Zu,
                '%' => Taste::Prozent,
                '=' => Taste::Gleich,
                'w' => Taste::Wurzel,
                'q' => Taste::Quadrat,
                'k' => Taste::Kehrwert,
                'v' => Taste::Vorzeichen,
                '<' => Taste::Ruecktaste,
                'c' => Taste::C,
                'a' => Taste::AC,
                'M' => Taste::MPlus,
                'm' => Taste::MMinus,
                'r' => Taste::MR,
                'x' => Taste::MC,
                ' ' => continue,
                _ => panic!("Testtaste {c}"),
            };
            m.druecken(t);
        }
    }

    fn nach(folge: &str) -> Anzeige {
        let mut m = Modell::new();
        tippen(&mut m, folge);
        m.anzeige()
    }

    #[test]
    fn grundrechnung() {
        let a = nach("12+7=");
        assert_eq!(a.eingabezeile, "12 + 7 =");
        assert_eq!(a.ergebnis, "19");
        assert!(!a.fehler);
        assert_eq!(nach("6*7=").ergebnis, "42");
        assert_eq!(nach("1/4=").ergebnis, "0,25");
        assert_eq!(nach("7-10=").ergebnis, "−3");
    }

    #[test]
    fn tippen_zeigt_zahl() {
        let a = nach("12+7");
        assert_eq!(a.eingabezeile, "12 + 7");
        assert_eq!(a.ergebnis, "7");
        assert_eq!(nach("").ergebnis, "0");
        assert_eq!(nach("").eingabezeile, "");
        assert_eq!(nach("12+").ergebnis, "12");
        assert_eq!(nach("0,5").ergebnis, "0,5");
        assert_eq!(nach(",5").ergebnis, "0,5");
        assert_eq!(nach("007").ergebnis, "7");
        assert_eq!(nach("1,,5").ergebnis, "1,5");
        assert_eq!(nach("1234567890123456789").ergebnis, "123456789012345");
    }

    #[test]
    fn klammern_und_vorrang() {
        assert_eq!(nach("12+7*(3-1)=").ergebnis, "26");
        assert_eq!(nach("(2+3)*4=").ergebnis, "20");
        assert_eq!(nach("2(3+4)=").ergebnis, "14", "× wird vor ( ergänzt");
        assert_eq!(nach("(2+3)4=").ergebnis, "20", "× wird nach ) ergänzt");
        assert_eq!(
            nach("(2+3=").ergebnis,
            "5",
            "offene Klammer wird geschlossen"
        );
        assert_eq!(nach("2+3)=").ergebnis, "5", "überzählige ) wird ignoriert");
        assert_eq!(nach("2+3)").eingabezeile, "2 + 3");
    }

    #[test]
    fn prozent() {
        assert_eq!(nach("200+10%=").ergebnis, "220");
        assert_eq!(nach("50%=").ergebnis, "0,5");
        assert_eq!(nach("200*10%=").ergebnis, "20");
        assert_eq!(nach("200+10%").eingabezeile, "200 + 10%");
        assert_eq!(nach("%").eingabezeile, "", "Prozent ohne Zahl tut nichts");
        assert_eq!(nach("50%=%=").ergebnis, "0,005", "Prozent auf Ergebnis");
    }

    #[test]
    fn division_durch_null_als_text() {
        let a = nach("5/0=");
        assert_eq!(a.ergebnis, "Division by zero");
        assert!(a.fehler);
        assert_eq!(nach("0k").ergebnis, "Division by zero");
        // Nach dem Fehler beginnt eine Ziffer neu, ein Operator tut nichts.
        assert_eq!(nach("5/0=7").eingabezeile, "7");
        assert_eq!(nach("5/0=+").eingabezeile, "");
        assert_eq!(nach("5/0=c").ergebnis, "0");
        assert_eq!(nach("5/0=<").ergebnis, "0");
    }

    #[test]
    fn ueberlauf_als_text() {
        let a = nach("1000000000000qqqqq");
        assert_eq!(a.ergebnis, "Overflow");
        assert!(a.fehler);
    }

    #[test]
    fn funktionen_sofort() {
        assert_eq!(nach("9w").ergebnis, "3");
        assert_eq!(nach("9w").eingabezeile, "3");
        assert_eq!(nach("12q").ergebnis, "144");
        assert_eq!(nach("4k").ergebnis, "0,25");
        assert_eq!(
            nach("2+9w=").ergebnis,
            "5",
            "Funktion wirkt nur auf die Zahl"
        );
        assert_eq!(nach("(2+7)w").eingabezeile, "3", "Klammer wird ausgewertet");
        assert_eq!(nach("2*(2+7)w=").ergebnis, "6");
        assert_eq!(nach("w").ergebnis, "0", "Wurzel aus nichts = 0");
        let a = nach("4vw");
        assert_eq!(a.ergebnis, "Square root of a negative number");
        assert!(a.fehler);
        assert_eq!(nach("16=w").ergebnis, "4", "Funktion auf Ergebnis");
        assert_eq!(nach("16=w").eingabezeile, "4");
        assert_eq!(nach("16=wq").ergebnis, "16");
    }

    #[test]
    fn vorzeichen() {
        assert_eq!(nach("5v").ergebnis, "−5");
        assert_eq!(nach("5v").eingabezeile, "−5");
        assert_eq!(nach("5vv").ergebnis, "5");
        assert_eq!(nach("5v3").ergebnis, "−53", "weiter tippen nach ±");
        assert_eq!(nach("3*5v=").ergebnis, "−15");
        assert_eq!(nach("v5=").ergebnis, "−5", "± vor der Zahl");
        assert_eq!(nach("vv5=").ergebnis, "5");
        assert_eq!(nach("(2+3)v=").ergebnis, "−5", "± vor Klammer");
        assert_eq!(nach("(2+3)vv=").ergebnis, "5");
        assert_eq!(nach("8=v").ergebnis, "−8", "± auf Ergebnis");
        assert_eq!(nach("0v5").ergebnis, "−5");
    }

    #[test]
    fn operator_ersetzen_und_vorzeichen_minus() {
        assert_eq!(nach("5+*3=").ergebnis, "15", "Operator ersetzt");
        assert_eq!(nach("5*-3=").ergebnis, "−15", "Minus nach × ist Vorzeichen");
        assert_eq!(nach("5*--3=").ergebnis, "15", "zweimal Minus hebt sich auf");
        assert_eq!(
            nach("5*-+3=").ergebnis,
            "8",
            "Plus hebt das Vorzeichen auf und ersetzt den Operator"
        );
        assert_eq!(nach("5*-+3").eingabezeile, "5 + 3");
        assert_eq!(nach("-5+2=").ergebnis, "−3");
        assert_eq!(
            nach("+5=").ergebnis,
            "5",
            "Plus vor der ersten Zahl wird ignoriert"
        );
        assert_eq!(nach("*5=").eingabezeile, "5 =");
        assert_eq!(
            nach("5+=").ergebnis,
            "5",
            "nachgestellter Operator fällt weg"
        );
        assert_eq!(nach("5+").eingabezeile, "5 +");
    }

    #[test]
    fn nach_gleich_weiterrechnen() {
        assert_eq!(nach("2+3=*4=").ergebnis, "20");
        assert_eq!(nach("2+3=*4=").eingabezeile, "5 × 4 =");
        assert_eq!(nach("2+3=7").eingabezeile, "7", "Ziffer beginnt neu");
        assert_eq!(nach("2+3=7").ergebnis, "7");
        assert_eq!(nach("2+3=,5").ergebnis, "0,5");
        assert_eq!(nach("2+3=(1+1)=").ergebnis, "2");
        assert_eq!(nach("2+3==").ergebnis, "5", "zweites Gleich ändert nichts");
    }

    #[test]
    fn ruecktaste_c_ac() {
        assert_eq!(nach("123<").ergebnis, "12");
        assert_eq!(nach("1<").ergebnis, "0");
        assert_eq!(nach("1<").eingabezeile, "");
        assert_eq!(nach("12+<").eingabezeile, "12");
        assert_eq!(
            nach("12+<3").ergebnis,
            "123",
            "nach Rücktaste weiter tippen"
        );
        assert_eq!(nach("12+7=<").eingabezeile, "12 + 7");
        assert_eq!(nach("12+7=<").ergebnis, "7");
        assert_eq!(nach("12+7=<<").eingabezeile, "12 +");
        assert_eq!(nach("5v<").ergebnis, "0");
        assert_eq!(nach("12+7c").eingabezeile, "12 +");
        assert_eq!(nach("12+7c").ergebnis, "12");
        assert_eq!(nach("12+7c9=").ergebnis, "21");
        assert_eq!(nach("12+7=c").eingabezeile, "");
        assert_eq!(nach("12+7a").eingabezeile, "");
        assert_eq!(nach("12+7a").ergebnis, "0");
        assert_eq!(nach("1<<<").ergebnis, "0", "Rücktaste auf leer ist harmlos");
        assert_eq!(nach("<c a").ergebnis, "0");
    }

    #[test]
    fn speicher() {
        let mut m = Modell::new();
        assert!(!m.anzeige().speicher_belegt);
        tippen(&mut m, "6*7=M");
        assert!(m.anzeige().speicher_belegt);
        tippen(&mut m, "a2*r=");
        assert_eq!(m.anzeige().ergebnis, "84");
        assert_eq!(m.anzeige().eingabezeile, "2 × 42 =");
        tippen(&mut m, "a10m");
        tippen(&mut m, "ar");
        assert_eq!(m.anzeige().ergebnis, "32", "42 − 10");
        tippen(&mut m, "x");
        assert!(!m.anzeige().speicher_belegt);
        tippen(&mut m, "ar");
        assert_eq!(m.anzeige().eingabezeile, "", "MR ohne Speicher tut nichts");
        // M+ mit getippter Zahl, MR ersetzt die getippte Zahl
        let mut m = Modell::new();
        tippen(&mut m, "5M a 3+9r=");
        assert_eq!(m.anzeige().ergebnis, "8");
        tippen(&mut m, "a(1+1)r=");
        assert_eq!(m.anzeige().ergebnis, "10", "MR nach ) ergänzt ×");
        // Fehler kann nicht gespeichert werden; C/AC lassen den Speicher stehen
        let mut m = Modell::new();
        tippen(&mut m, "1/0=M");
        assert!(!m.anzeige().speicher_belegt);
        tippen(&mut m, "a7Mca");
        assert!(m.anzeige().speicher_belegt);
        // M+ auf 0 belegt den Speicher (Marke sichtbar)
        assert!(nach("M").speicher_belegt);
    }

    #[test]
    fn kopieren_und_einfuegen() {
        let mut m = Modell::new();
        tippen(&mut m, "6*7=");
        assert_eq!(m.kopierwert(), Some("42".to_string()));
        tippen(&mut m, "a7-10=");
        assert_eq!(
            m.kopierwert(),
            Some("-10".to_string()).map(|_| "-3".to_string())
        );
        tippen(&mut m, "a12,5");
        assert_eq!(m.kopierwert(), Some("12,5".to_string()));
        tippen(&mut m, "+");
        assert_eq!(m.kopierwert(), None, "nach Operator nichts zu kopieren");
        tippen(&mut m, "a1/0=");
        assert_eq!(m.kopierwert(), None);

        let mut m = Modell::new();
        assert!(m.einfuegen("12,5"));
        assert_eq!(m.anzeige().ergebnis, "12,5");
        tippen(&mut m, "*2=");
        assert_eq!(m.anzeige().ergebnis, "25");
        assert!(m.einfuegen(" 3.25 "), "Punkt wird angenommen");
        assert_eq!(
            m.anzeige().eingabezeile,
            "3,25",
            "Einfügen nach Gleich beginnt neu"
        );
        assert!(!m.einfuegen("Hallo"));
        assert_eq!(m.anzeige().eingabezeile, "3,25");
        tippen(&mut m, "+");
        assert!(m.einfuegen("-1"));
        tippen(&mut m, "=");
        assert_eq!(m.anzeige().ergebnis, "2,25");
        tippen(&mut m, "a(1+1)");
        assert!(m.einfuegen("5"));
        tippen(&mut m, "=");
        assert_eq!(m.anzeige().ergebnis, "10", "Einfügen nach ) ergänzt ×");
        tippen(&mut m, "a123");
        assert!(m.einfuegen("4"));
        assert_eq!(
            m.anzeige().eingabezeile,
            "4",
            "Einfügen ersetzt die getippte Zahl"
        );
        tippen(&mut m, "5");
        assert_eq!(
            m.anzeige().eingabezeile,
            "4 × 5",
            "nach eingefügter Zahl: × ergänzt"
        );
    }

    #[test]
    fn genauigkeit_ketten() {
        // QREL-05 R.3 (Werkstatt, kd7.png/kd1.png): 2 √ x² = 2, 1/x = 0,5.
        assert_eq!(nach("2w").ergebnis, "1,41421356237");
        assert_eq!(nach("2wq").ergebnis, "2");
        assert_eq!(nach("2wq").eingabezeile, "2");
        assert_eq!(nach("2wqk").ergebnis, "0,5");
        assert_eq!(nach("2wqk*30=").ergebnis, "15");
        assert_eq!(nach("2wqk*30=").eingabezeile, "0,5 × 30 =");
        assert_eq!(nach("2=wq").ergebnis, "2", "Funktionen auf Ergebnis");
        assert_eq!(nach("2wq=").ergebnis, "2");
        assert_eq!(nach("3kk").ergebnis, "3", "1/x 1/x");
        assert_eq!(nach("1/3*3=").ergebnis, "1");
        assert_eq!(nach("1/3=*3=").ergebnis, "1", "Ergebnis bleibt genau");
        assert_eq!(nach("1/3=*3=").eingabezeile, "0,333333333333 × 3 =");
        assert_eq!(nach("0,1+0,2=").ergebnis, "0,3");
        assert_eq!(nach("0,1+0,2=-0,3=").ergebnis, "0", "keine Binärreste");
        assert_eq!(nach("0,1+0,2-0,3=").ergebnis, "0");
        assert_eq!(nach("2w+(2w)=q").ergebnis, "8");
        assert_eq!(nach("2wv").ergebnis, "−1,41421356237");
        assert_eq!(nach("2wvq").ergebnis, "2", "± auf genauen Wert");
        assert_eq!(nach("2w%").eingabezeile, "1,41421356237%");
        // Kopieren wie die Anzeige (gerundet), Rechnen mit vollem Wert.
        let mut m = Modell::new();
        tippen(&mut m, "2w");
        assert_eq!(m.kopierwert(), Some("1,41421356237".to_string()));
        tippen(&mut m, "q");
        assert_eq!(m.kopierwert(), Some("2".to_string()));
        tippen(&mut m, "k");
        assert_eq!(m.kopierwert(), Some("0,5".to_string()));
        tippen(&mut m, "=");
        assert_eq!(m.kopierwert(), Some("0,5".to_string()));
        // Einfügen behält alle Stellen (hier mehr als die Anzeige).
        let mut m = Modell::new();
        assert!(m.einfuegen("1,4142135623730951"));
        assert_eq!(m.anzeige().ergebnis, "1,41421356237");
        tippen(&mut m, "q");
        assert_eq!(m.anzeige().ergebnis, "2");
    }

    #[test]
    fn speicher_genau() {
        let mut m = Modell::new();
        tippen(&mut m, "2wM a rq");
        assert_eq!(m.anzeige().ergebnis, "2", "MR liefert den vollen Wert");
        let mut m = Modell::new();
        tippen(&mut m, "0,1M a0,2M a0,3m a r");
        assert_eq!(m.anzeige().ergebnis, "0", "Auslöschung im Speicher");
        let mut m = Modell::new();
        tippen(&mut m, "1/3=M a 3*r=");
        assert_eq!(m.anzeige().ergebnis, "1");
    }

    #[test]
    fn ziffer_nach_funktion() {
        // Unverändert seit 0.1.0: Ziffer nach Funktionsergebnis ergänzt „ד
        // (Vorbild: gcalctool 5.30 beginnt nur nach Gleich neu).
        assert_eq!(nach("4w3").eingabezeile, "2 × 3");
        assert_eq!(nach("4w3=").ergebnis, "6");
        assert_eq!(nach("4w=3").eingabezeile, "3", "nach Gleich neu");
        assert_eq!(nach("4w+<").eingabezeile, "2");
        assert_eq!(
            nach("4w+<5").eingabezeile,
            "2 × 5",
            "Funktionsergebnis ist keine getippte Zahl"
        );
        assert_eq!(nach("4wc").eingabezeile, "", "C löscht den Wert");
    }

    #[test]
    fn dezimalbruch_anzeige() {
        assert_eq!(nach("0,1+0,2=").ergebnis, "0,3");
        assert_eq!(nach("1/3=").ergebnis, "0,333333333333");
        assert_eq!(nach("2w").ergebnis, "1,41421356237");
    }
}
