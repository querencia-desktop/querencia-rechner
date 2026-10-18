//! Rechenkern: Tokens, Parser und Auswertung – ohne GTK, vollständig mit
//! Unit-Tests abgedeckt.
//!
//! **Grammatik** (Punkt vor Strich, Klammern, Vorzeichen, Prozent):
//!
//! ```text
//! ausdruck := term   (('+' | '−') term)*
//! term     := faktor (('×' | '÷') faktor)*
//! faktor   := '−' faktor | primaer
//! primaer  := ( zahl | '(' ausdruck ')' ) '%'*
//! ```
//!
//! **Zahlen** rechnet der Kern als `f64`; Zwischenwerte bleiben ungerundet
//! (so ergibt (1 ÷ 3) × 3 wieder 1), das Endergebnis und die Ergebnisse der
//! Funktionen √, x², 1/x werden mit [`runden`] auf 12 signifikante Stellen
//! gerundet, damit Dezimalbrüche wie 0,1 + 0,2 exakt 0,3 ergeben und
//! Folgerechnungen keine Binärreste mitschleppen.
//! [`formatieren`] zeigt höchstens 12 signifikante Stellen mit Dezimalkomma;
//! außerhalb von 10⁻⁹ … 10¹² wird die Exponentenschreibweise (`1,5e+15`)
//! verwendet. Das Vorzeichen ist in allen Texten des Kerns das ASCII-Minus
//! (zum Einfügen in andere Programme); das Fenster zeigt es als „−“ (U+2212).
//!
//! **Prozent** folgt der Semantik von gcalctool bzw. seinem Nachfolger
//! gnome-calculator (das Prozentzeichen ist ein nachgestellter Operator):
//!
//! | Eingabe      | Bedeutung       | Beispiel            |
//! |--------------|-----------------|---------------------|
//! | `n %`        | n ÷ 100         | `50 %` = 0,5        |
//! | `a + b %`    | a + a·b ÷ 100   | `200 + 10 %` = 220  |
//! | `a − b %`    | a − a·b ÷ 100   | `200 − 10 %` = 180  |
//! | `a × b %`    | a × b ÷ 100     | `200 × 10 %` = 20   |
//! | `a ÷ b %`    | a ÷ (b ÷ 100)   | `200 ÷ 10 %` = 2000 |
//!
//! Steht das Prozent links eines Operators oder in Klammern, zählt es nur als
//! n ÷ 100: `10 % + 5` = 5,1 und `(10 %) × 3` = 0,3. Punkt vor Strich bleibt
//! erhalten: `2 × 3 + 10 %` = 6 + 0,6 = 6,6.
//!
//! **Fehler** sind Werte, keine Abbrüche: Division durch Null, Überlauf
//! (unendlich oder nicht darstellbar), Wurzel aus negativer Zahl, unvollständiger
//! oder ungültiger Ausdruck.

use std::fmt;

/// Zweistellige Operatoren.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Plus,
    Minus,
    Mal,
    Geteilt,
}

impl Op {
    /// Zeichen in der Eingabezeile.
    pub fn zeichen(self) -> &'static str {
        match self {
            Op::Plus => "+",
            Op::Minus => "−",
            Op::Mal => "×",
            Op::Geteilt => "÷",
        }
    }
}

/// Bausteine eines Ausdrucks. Zahlen bleiben als Text, wie sie getippt
/// wurden (z. B. `0,50` oder `-3`), damit die Anzeige beim Tippen stimmt.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Zahl(String),
    Op(Op),
    Auf,
    Zu,
    Prozent,
}

/// Rechenfehler, in der Anzeige als lesbarer Text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fehler {
    DivisionDurchNull,
    Ueberlauf,
    NegativeWurzel,
    Unvollstaendig,
    Ungueltig,
}

impl Fehler {
    pub fn text(self) -> &'static str {
        match self {
            Fehler::DivisionDurchNull => "Division durch Null",
            Fehler::Ueberlauf => "Überlauf",
            Fehler::NegativeWurzel => "Wurzel aus negativer Zahl",
            Fehler::Unvollstaendig => "Unvollständiger Ausdruck",
            Fehler::Ungueltig => "Ungültiger Ausdruck",
        }
    }
}

impl fmt::Display for Fehler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text())
    }
}

/// Signifikante Stellen für Rundung und Anzeige.
const STELLEN: i32 = 12;

/// Zwischenwert: unendlich/NaN ist ein Überlauf.
fn endlich(x: f64) -> Result<f64, Fehler> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(Fehler::Ueberlauf)
    }
}

/// Rundet auf 12 signifikante Stellen; unendlich/NaN wird zum Fehler.
pub fn runden(x: f64) -> Result<f64, Fehler> {
    if !x.is_finite() {
        return Err(Fehler::Ueberlauf);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    let e = x.abs().log10().floor() as i32;
    if !(-280..=280).contains(&e) {
        // Rundung würde selbst überlaufen; die Anzeige rundet ohnehin.
        return Ok(x);
    }
    let faktor = 10f64.powi(STELLEN - 1 - e);
    let r = (x * faktor).round() / faktor;
    if r.is_finite() {
        Ok(r)
    } else {
        Err(Fehler::Ueberlauf)
    }
}

/// Zahl mit Dezimalkomma für die Anzeige (ASCII-Minus, kein Tausenderpunkt).
pub fn formatieren(x: f64) -> String {
    if !x.is_finite() {
        return Fehler::Ueberlauf.text().to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    // Mantisse mit 12 signifikanten Stellen, Exponent dezimal.
    let s = format!("{:.*e}", (STELLEN - 1) as usize, x);
    let (mantisse, exponent) = s.split_once('e').expect("Exponentform");
    let exponent: i32 = exponent.parse().expect("Exponent");
    let negativ = mantisse.starts_with('-');
    let ziffern: String = mantisse.chars().filter(char::is_ascii_digit).collect();
    let ziffern = ziffern.trim_end_matches('0');
    let ziffern = if ziffern.is_empty() { "0" } else { ziffern };
    let mut aus = String::new();
    if negativ {
        aus.push('-');
    }
    if (-9..STELLEN).contains(&exponent) {
        if exponent >= 0 {
            let vorkomma = exponent as usize + 1;
            if ziffern.len() <= vorkomma {
                aus.push_str(ziffern);
                aus.push_str(&"0".repeat(vorkomma - ziffern.len()));
            } else {
                aus.push_str(&ziffern[..vorkomma]);
                aus.push(',');
                aus.push_str(&ziffern[vorkomma..]);
            }
        } else {
            aus.push_str("0,");
            aus.push_str(&"0".repeat((-exponent - 1) as usize));
            aus.push_str(ziffern);
        }
    } else {
        aus.push_str(&ziffern[..1]);
        if ziffern.len() > 1 {
            aus.push(',');
            aus.push_str(&ziffern[1..]);
        }
        aus.push('e');
        aus.push(if exponent < 0 { '-' } else { '+' });
        aus.push_str(&exponent.abs().to_string());
    }
    aus
}

/// Liest eine Zahl aus Text: Dezimalkomma oder -punkt, Vorzeichen `-`, `−`
/// oder `+`, optional Exponent (`1,5e+15`). Leerraum außen wird ignoriert.
pub fn zahl_lesen(text: &str) -> Option<f64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            ',' => s.push('.'),
            '−' => s.push('-'),
            '\u{a0}' | '\u{202f}' | '\'' => {} // geschützte Leerzeichen/Apostroph als Tausendertrenner
            c => s.push(c),
        }
    }
    // Nur Ziffern, höchstens ein Punkt, ein Vorzeichen, ein Exponent.
    let ok = s
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'));
    if !ok || s.matches('.').count() > 1 || !s.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let x: f64 = s.parse().ok()?;
    x.is_finite().then_some(x)
}

/// Quadratwurzel.
pub fn wurzel(x: f64) -> Result<f64, Fehler> {
    if x < 0.0 {
        return Err(Fehler::NegativeWurzel);
    }
    runden(x.sqrt())
}

/// Quadrat.
pub fn quadrat(x: f64) -> Result<f64, Fehler> {
    runden(x * x)
}

/// Kehrwert.
pub fn kehrwert(x: f64) -> Result<f64, Fehler> {
    if x == 0.0 {
        return Err(Fehler::DivisionDurchNull);
    }
    runden(1.0 / x)
}

/// Zwischenwert der Auswertung: ein Prozentwert merkt sich, dass er mit `%`
/// gebildet wurde, damit `a + b %` die gcalctool-Bedeutung bekommt.
#[derive(Clone, Copy, Debug)]
enum Wert {
    Normal(f64),
    Prozent(f64),
}

impl Wert {
    fn zahl(self) -> f64 {
        match self {
            Wert::Normal(x) => x,
            Wert::Prozent(p) => p / 100.0,
        }
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl Parser<'_> {
    fn naechstes(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn ausdruck(&mut self) -> Result<Wert, Fehler> {
        let mut links = self.term()?;
        while let Some(Token::Op(op @ (Op::Plus | Op::Minus))) = self.naechstes() {
            let op = *op;
            self.pos += 1;
            let rechts = self.term()?;
            let a = links.zahl();
            let r = match (op, rechts) {
                (Op::Plus, Wert::Prozent(p)) => a + a * p / 100.0,
                (Op::Minus, Wert::Prozent(p)) => a - a * p / 100.0,
                (Op::Plus, Wert::Normal(b)) => a + b,
                (Op::Minus, Wert::Normal(b)) => a - b,
                _ => unreachable!(),
            };
            links = Wert::Normal(endlich(r)?);
        }
        Ok(links)
    }

    fn term(&mut self) -> Result<Wert, Fehler> {
        let mut links = self.faktor()?;
        while let Some(Token::Op(op @ (Op::Mal | Op::Geteilt))) = self.naechstes() {
            let op = *op;
            self.pos += 1;
            let rechts = self.faktor()?;
            let a = links.zahl();
            let b = rechts.zahl();
            let r = match op {
                Op::Mal => a * b,
                Op::Geteilt => {
                    if b == 0.0 {
                        return Err(Fehler::DivisionDurchNull);
                    }
                    a / b
                }
                _ => unreachable!(),
            };
            links = Wert::Normal(endlich(r)?);
        }
        Ok(links)
    }

    fn faktor(&mut self) -> Result<Wert, Fehler> {
        if let Some(Token::Op(Op::Minus)) = self.naechstes() {
            self.pos += 1;
            let w = self.faktor()?;
            return Ok(match w {
                Wert::Normal(x) => Wert::Normal(-x),
                Wert::Prozent(p) => Wert::Prozent(-p),
            });
        }
        self.primaer()
    }

    fn primaer(&mut self) -> Result<Wert, Fehler> {
        let mut wert = match self.naechstes() {
            Some(Token::Zahl(text)) => {
                let x = zahl_lesen(text).ok_or(Fehler::Ungueltig)?;
                self.pos += 1;
                Wert::Normal(x)
            }
            Some(Token::Auf) => {
                self.pos += 1;
                let innen = self.ausdruck()?;
                match self.naechstes() {
                    Some(Token::Zu) => self.pos += 1,
                    None => return Err(Fehler::Unvollstaendig),
                    Some(_) => return Err(Fehler::Ungueltig),
                }
                Wert::Normal(endlich(innen.zahl())?)
            }
            None => return Err(Fehler::Unvollstaendig),
            Some(_) => return Err(Fehler::Ungueltig),
        };
        while let Some(Token::Prozent) = self.naechstes() {
            self.pos += 1;
            wert = Wert::Prozent(wert.zahl());
        }
        Ok(wert)
    }
}

/// Wertet einen vollständigen Ausdruck aus.
pub fn auswerten(tokens: &[Token]) -> Result<f64, Fehler> {
    if tokens.is_empty() {
        return Err(Fehler::Unvollstaendig);
    }
    let mut p = Parser { tokens, pos: 0 };
    let wert = p.ausdruck()?;
    match p.naechstes() {
        None => runden(wert.zahl()),
        Some(Token::Zu) => Err(Fehler::Ungueltig),
        Some(_) => Err(Fehler::Ungueltig),
    }
}

/// Text der Eingabezeile: `12 + 7 × (3 − 1)`, Vorzeichen als „−“.
pub fn ausdruck_text(tokens: &[Token]) -> String {
    let mut aus = String::new();
    for (i, t) in tokens.iter().enumerate() {
        let stueck: String = match t {
            Token::Zahl(z) => z.replace('-', "−"),
            Token::Op(op) => op.zeichen().to_string(),
            Token::Auf => "(".to_string(),
            Token::Zu => ")".to_string(),
            Token::Prozent => "%".to_string(),
        };
        let ohne_abstand = i == 0
            || matches!(t, Token::Zu | Token::Prozent)
            || matches!(tokens[i - 1], Token::Auf);
        if !ohne_abstand {
            aus.push(' ');
        }
        aus.push_str(&stueck);
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(s: &str) -> Token {
        Token::Zahl(s.to_string())
    }
    fn op(o: Op) -> Token {
        Token::Op(o)
    }

    /// Hilfsparser für die Tests: Ziffern/Komma/Punkt zu Zahlen, Operatoren
    /// `+ - * /`, Klammern, `%`; Leerzeichen trennen.
    fn t(s: &str) -> Vec<Token> {
        let mut aus = Vec::new();
        let mut zahl = String::new();
        for c in s.chars() {
            if c.is_ascii_digit() || c == ',' || c == '.' {
                zahl.push(if c == '.' { ',' } else { c });
                continue;
            }
            if !zahl.is_empty() {
                aus.push(Token::Zahl(std::mem::take(&mut zahl)));
            }
            match c {
                '+' => aus.push(op(Op::Plus)),
                '-' => aus.push(op(Op::Minus)),
                '*' => aus.push(op(Op::Mal)),
                '/' => aus.push(op(Op::Geteilt)),
                '(' => aus.push(Token::Auf),
                ')' => aus.push(Token::Zu),
                '%' => aus.push(Token::Prozent),
                ' ' => {}
                _ => panic!("Testzeichen {c}"),
            }
        }
        if !zahl.is_empty() {
            aus.push(Token::Zahl(zahl));
        }
        aus
    }

    fn rechne(s: &str) -> Result<f64, Fehler> {
        auswerten(&t(s))
    }

    #[test]
    fn grundrechenarten() {
        assert_eq!(rechne("1+2"), Ok(3.0));
        assert_eq!(rechne("7-10"), Ok(-3.0));
        assert_eq!(rechne("6*7"), Ok(42.0));
        assert_eq!(rechne("1/4"), Ok(0.25));
        assert_eq!(rechne("12,5+0,5"), Ok(13.0));
    }

    #[test]
    fn punkt_vor_strich() {
        assert_eq!(rechne("2+3*4"), Ok(14.0));
        assert_eq!(rechne("2*3+4"), Ok(10.0));
        assert_eq!(rechne("10-4/2"), Ok(8.0));
        assert_eq!(rechne("1-2-3"), Ok(-4.0), "links-assoziativ");
        assert_eq!(rechne("100/10/2"), Ok(5.0), "links-assoziativ");
    }

    #[test]
    fn klammern() {
        assert_eq!(rechne("(2+3)*4"), Ok(20.0));
        assert_eq!(rechne("2*(3+4)"), Ok(14.0));
        assert_eq!(rechne("((1+2)*(3+4))"), Ok(21.0));
        assert_eq!(rechne("12+7*(3-1)"), Ok(26.0));
        assert_eq!(rechne("(5)"), Ok(5.0));
    }

    #[test]
    fn vorzeichen() {
        assert_eq!(rechne("-5+2"), Ok(-3.0));
        assert_eq!(rechne("3*-2"), Ok(-6.0));
        assert_eq!(rechne("-(2+3)"), Ok(-5.0));
        assert_eq!(rechne("--4"), Ok(4.0));
        assert_eq!(auswerten(&[z("-3"), op(Op::Mal), z("-3")]), Ok(9.0));
    }

    #[test]
    fn prozent_wie_gcalctool() {
        assert_eq!(rechne("50%"), Ok(0.5));
        assert_eq!(rechne("200+10%"), Ok(220.0));
        assert_eq!(rechne("200-10%"), Ok(180.0));
        assert_eq!(rechne("200*10%"), Ok(20.0));
        assert_eq!(rechne("200/10%"), Ok(2000.0));
        assert_eq!(rechne("10%+5"), Ok(5.1), "Prozent links zählt als n/100");
        assert_eq!(
            rechne("(10%)*3"),
            Ok(0.3),
            "Klammer macht aus Prozent eine Zahl"
        );
        assert_eq!(rechne("2*3+10%"), Ok(6.6), "Punkt vor Strich bleibt");
        assert_eq!(rechne("100+(10%)"), Ok(100.1));
        assert_eq!(rechne("-50%"), Ok(-0.5));
    }

    #[test]
    fn dezimalbrueche_ohne_binaerreste() {
        assert_eq!(rechne("0,1+0,2"), Ok(0.3));
        assert_eq!(formatieren(rechne("0,1+0,2").unwrap()), "0,3");
        assert_eq!(formatieren(rechne("1/3").unwrap()), "0,333333333333");
        assert_eq!(formatieren(rechne("2/3").unwrap()), "0,666666666667");
        assert_eq!(rechne("(1/3)*3"), Ok(1.0));
    }

    #[test]
    fn fehler_sind_werte() {
        assert_eq!(rechne("1/0"), Err(Fehler::DivisionDurchNull));
        assert_eq!(rechne("0/0"), Err(Fehler::DivisionDurchNull));
        assert_eq!(rechne("5/(2-2)"), Err(Fehler::DivisionDurchNull));
        assert_eq!(rechne("1+"), Err(Fehler::Unvollstaendig));
        assert_eq!(rechne("(1+2"), Err(Fehler::Unvollstaendig));
        assert_eq!(rechne(""), Err(Fehler::Unvollstaendig));
        assert_eq!(rechne("1+2)"), Err(Fehler::Ungueltig));
        assert_eq!(rechne("2 3"), Err(Fehler::Ungueltig));
        assert_eq!(rechne("*3"), Err(Fehler::Ungueltig));
        assert_eq!(wurzel(-4.0), Err(Fehler::NegativeWurzel));
        assert_eq!(kehrwert(0.0), Err(Fehler::DivisionDurchNull));
        assert_eq!(Fehler::DivisionDurchNull.text(), "Division durch Null");
    }

    #[test]
    fn ueberlauf() {
        let gross = Token::Zahl("1e200".to_string());
        assert_eq!(
            auswerten(&[gross.clone(), op(Op::Mal), gross.clone()]),
            Err(Fehler::Ueberlauf)
        );
        assert_eq!(quadrat(1e200), Err(Fehler::Ueberlauf));
        assert_eq!(runden(f64::INFINITY), Err(Fehler::Ueberlauf));
        assert_eq!(runden(f64::NAN), Err(Fehler::Ueberlauf));
        // Nahe der Grenze darf die Rundung selbst nicht überlaufen.
        assert!(runden(1e300).is_ok());
        assert!(runden(1e-300).is_ok());
    }

    #[test]
    fn unaere_funktionen() {
        assert_eq!(wurzel(9.0), Ok(3.0));
        assert_eq!(
            wurzel(2.0).map(formatieren),
            Ok("1,41421356237".to_string())
        );
        assert_eq!(quadrat(12.0), Ok(144.0));
        assert_eq!(quadrat(-0.5), Ok(0.25));
        assert_eq!(kehrwert(4.0), Ok(0.25));
        assert_eq!(
            kehrwert(3.0).map(formatieren),
            Ok("0,333333333333".to_string())
        );
    }

    #[test]
    fn formatieren_dezimalkomma() {
        assert_eq!(formatieren(0.0), "0");
        assert_eq!(formatieren(-0.0), "0");
        assert_eq!(formatieren(42.0), "42");
        assert_eq!(formatieren(-3.5), "-3,5");
        assert_eq!(formatieren(0.25), "0,25");
        assert_eq!(formatieren(0.001), "0,001");
        assert_eq!(formatieren(123456789012.0), "123456789012");
        assert_eq!(formatieren(100.0), "100");
        assert_eq!(formatieren(1e11), "100000000000");
        assert_eq!(formatieren(1e-9), "0,000000001");
        assert_eq!(formatieren(1.5), "1,5");
        assert_eq!(formatieren(1234.5678), "1234,5678");
    }

    #[test]
    fn formatieren_exponent() {
        assert_eq!(formatieren(1e12), "1e+12");
        assert_eq!(formatieren(1.5e15), "1,5e+15");
        assert_eq!(formatieren(-2.5e20), "-2,5e+20");
        assert_eq!(formatieren(1e-10), "1e-10");
        assert_eq!(formatieren(1.23456789012345e-12), "1,23456789012e-12");
        assert_eq!(formatieren(f64::INFINITY), "Überlauf");
    }

    #[test]
    fn runden_zwoelf_stellen() {
        assert_eq!(runden(0.1 + 0.2), Ok(0.3));
        assert_eq!(runden(1.0 / 3.0), Ok(0.333333333333));
        assert_eq!(runden(123456789012345.0), Ok(123456789012000.0));
        assert_eq!(runden(-2.5), Ok(-2.5));
        assert_eq!(runden(0.0), Ok(0.0));
    }

    #[test]
    fn zahl_lesen_varianten() {
        assert_eq!(zahl_lesen("12,5"), Some(12.5));
        assert_eq!(zahl_lesen("12.5"), Some(12.5));
        assert_eq!(zahl_lesen(" -3 "), Some(-3.0));
        assert_eq!(zahl_lesen("−3"), Some(-3.0));
        assert_eq!(zahl_lesen("+7"), Some(7.0));
        assert_eq!(zahl_lesen("1,5e+15"), Some(1.5e15));
        assert_eq!(zahl_lesen("1e-3"), Some(0.001));
        assert_eq!(zahl_lesen("0,"), Some(0.0));
        assert_eq!(zahl_lesen(",5"), Some(0.5));
        assert_eq!(zahl_lesen(""), None);
        assert_eq!(zahl_lesen("abc"), None);
        assert_eq!(zahl_lesen("1.234,5"), None, "zwei Trenner sind mehrdeutig");
        assert_eq!(zahl_lesen("12 + 3"), None);
        assert_eq!(zahl_lesen("-"), None);
        assert_eq!(zahl_lesen("1e400"), None, "unendlich wird abgelehnt");
    }

    #[test]
    fn ausdruck_text_abstaende() {
        assert_eq!(ausdruck_text(&t("12+7*(3-1)")), "12 + 7 × (3 − 1)");
        assert_eq!(ausdruck_text(&t("200+10%")), "200 + 10%");
        assert_eq!(ausdruck_text(&[z("-3"), op(Op::Mal), z("2")]), "−3 × 2");
        assert_eq!(ausdruck_text(&[]), "");
        assert_eq!(ausdruck_text(&t("(")), "(");
    }
}
