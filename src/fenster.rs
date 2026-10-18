//! Das Rechnerfenster: Menüleiste „Rechner | Hilfe“, zweizeilige Anzeige,
//! Tastenfeld 5 × 6. Keine eigene Titelleiste (`set_titlebar` wird nie
//! gerufen): labwc zeichnet die klassische Server-Titelleiste.
//!
//! Tastatur (ohne Strg/Alt, auch Ziffernblock):
//! Ziffern · `,` `.` Komma · `+ − × ÷` als `+ - * /` · `( )` · `%` ·
//! Eingabetaste oder `=` Gleich · Rücktaste · Entf = C · Escape = AC ·
//! `s` √ · `q` x² · `r` 1/x · `v` ± · Strg+C Kopieren · Strg+V Einfügen ·
//! Strg+Q Beenden.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::dialoge;
use crate::modell::{Anzeige, Modell, Taste};
use crate::rechenkern::Op;

const CSS: &str = include_str!("style.css");

/// Rückruf einer Fensteraktion.
type ActionFn = Box<dyn Fn(&Rc<Fenster>)>;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn install_accels(app: &gtk::Application) {
    for (action, keys) in [
        ("win.copy", &["<Control>c"][..]),
        ("win.paste", &["<Control>v"]),
        ("win.quit", &["<Control>q"]),
    ] {
        app.set_accels_for_action(action, keys);
    }
}

fn menu_model() -> gio::Menu {
    let rechner = gio::Menu::new();
    let s1 = gio::Menu::new();
    s1.append(Some("_Kopieren"), Some("win.copy"));
    s1.append(Some("E_infügen"), Some("win.paste"));
    rechner.append_section(None, &s1);
    let s2 = gio::Menu::new();
    s2.append(Some("_Beenden"), Some("win.quit"));
    rechner.append_section(None, &s2);
    let hilfe = gio::Menu::new();
    hilfe.append(Some("_Info"), Some("win.about"));
    let bar = gio::Menu::new();
    bar.append_submenu(Some("_Rechner"), &rechner);
    bar.append_submenu(Some("_Hilfe"), &hilfe);
    bar
}

/// Tastenfeld: Beschriftung, Taste, CSS-Klasse. Fünf Spalten, sechs Zeilen.
const TASTEN: [[(&str, Taste, &str); 5]; 6] = [
    [
        ("MC", Taste::MC, "qr-speicher-taste"),
        ("MR", Taste::MR, "qr-speicher-taste"),
        ("M−", Taste::MMinus, "qr-speicher-taste"),
        ("M+", Taste::MPlus, "qr-speicher-taste"),
        ("←", Taste::Ruecktaste, "qr-loeschen"),
    ],
    [
        ("C", Taste::C, "qr-loeschen"),
        ("AC", Taste::AC, "qr-loeschen"),
        ("(", Taste::Auf, "qr-op"),
        (")", Taste::Zu, "qr-op"),
        ("÷", Taste::Op(Op::Geteilt), "qr-op"),
    ],
    [
        ("7", Taste::Ziffer(7), "qr-ziffer"),
        ("8", Taste::Ziffer(8), "qr-ziffer"),
        ("9", Taste::Ziffer(9), "qr-ziffer"),
        ("×", Taste::Op(Op::Mal), "qr-op"),
        ("√", Taste::Wurzel, "qr-funktion"),
    ],
    [
        ("4", Taste::Ziffer(4), "qr-ziffer"),
        ("5", Taste::Ziffer(5), "qr-ziffer"),
        ("6", Taste::Ziffer(6), "qr-ziffer"),
        ("−", Taste::Op(Op::Minus), "qr-op"),
        ("x²", Taste::Quadrat, "qr-funktion"),
    ],
    [
        ("1", Taste::Ziffer(1), "qr-ziffer"),
        ("2", Taste::Ziffer(2), "qr-ziffer"),
        ("3", Taste::Ziffer(3), "qr-ziffer"),
        ("+", Taste::Op(Op::Plus), "qr-op"),
        ("1/x", Taste::Kehrwert, "qr-funktion"),
    ],
    [
        ("0", Taste::Ziffer(0), "qr-ziffer"),
        (",", Taste::Komma, "qr-ziffer"),
        ("±", Taste::Vorzeichen, "qr-funktion"),
        ("%", Taste::Prozent, "qr-op"),
        ("=", Taste::Gleich, "qr-gleich"),
    ],
];

/// Kurzbeschreibung als Tooltip (Hilfe für Maus und Tastatur).
fn tooltip(taste: Taste) -> Option<&'static str> {
    Some(match taste {
        Taste::MC => "Speicher löschen",
        Taste::MR => "Speicher abrufen",
        Taste::MMinus => "Vom Speicher abziehen",
        Taste::MPlus => "Zum Speicher addieren",
        Taste::Ruecktaste => "Letztes Zeichen löschen (Rücktaste)",
        Taste::C => "Aktuelle Zahl löschen (Entf)",
        Taste::AC => "Alles löschen (Escape)",
        Taste::Wurzel => "Quadratwurzel (s)",
        Taste::Quadrat => "Quadrat (q)",
        Taste::Kehrwert => "Kehrwert 1/x (r)",
        Taste::Vorzeichen => "Vorzeichen wechseln (v)",
        Taste::Prozent => "Prozent",
        Taste::Gleich => "Berechnen (Eingabetaste)",
        _ => return None,
    })
}

pub struct Fenster {
    window: gtk::ApplicationWindow,
    modell: RefCell<Modell>,
    eingabezeile: gtk::Label,
    ergebnis: gtk::Label,
    speicher: gtk::Label,
    debug: bool,
}

thread_local! {
    static FENSTER: RefCell<Option<Rc<Fenster>>> = const { RefCell::new(None) };
}

impl Fenster {
    /// Das (einzige) Fenster der Anwendung, bei Bedarf neu angelegt.
    pub fn get(app: &gtk::Application) -> Rc<Self> {
        if let Some(f) = FENSTER.with(|f| f.borrow().clone()) {
            return f;
        }
        let f = Self::build(app);
        FENSTER.with(|slot| *slot.borrow_mut() = Some(f.clone()));
        f
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn build(app: &gtk::Application) -> Rc<Self> {
        app.set_menubar(Some(&menu_model()));
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Taschenrechner")
            .resizable(false)
            .show_menubar(true)
            .build();
        window.add_css_class("qr-window");
        dialoge::set_window_icon(&window);

        // Anzeige
        let eingabezeile = gtk::Label::new(None);
        eingabezeile.add_css_class("qr-eingabezeile");
        eingabezeile.set_xalign(1.0);
        eingabezeile.set_ellipsize(gtk::pango::EllipsizeMode::Start);
        eingabezeile.set_single_line_mode(true);
        eingabezeile.set_hexpand(true);
        let ergebnis = gtk::Label::new(Some("0"));
        ergebnis.add_css_class("qr-ergebnis");
        ergebnis.set_xalign(1.0);
        ergebnis.set_ellipsize(gtk::pango::EllipsizeMode::Start);
        ergebnis.set_single_line_mode(true);
        ergebnis.set_hexpand(true);
        ergebnis.set_selectable(false);
        let speicher = gtk::Label::new(None);
        speicher.add_css_class("qr-speicher");
        speicher.set_xalign(0.0);
        speicher.set_valign(gtk::Align::End);
        let zeile2 = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        zeile2.append(&speicher);
        zeile2.append(&ergebnis);
        let anzeige = gtk::Box::new(gtk::Orientation::Vertical, 2);
        anzeige.add_css_class("qr-anzeige");
        anzeige.set_width_request(260);
        anzeige.append(&eingabezeile);
        anzeige.append(&zeile2);

        // Tastenfeld
        let grid = gtk::Grid::new();
        grid.add_css_class("qr-tasten");
        grid.set_row_spacing(4);
        grid.set_column_spacing(4);
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.append(&anzeige);
        vbox.append(&grid);
        window.set_child(Some(&vbox));

        let debug = std::env::var_os("QUERENCIA_RECHNER_DEBUG").is_some();
        let fenster = Rc::new(Self {
            window,
            modell: RefCell::new(Modell::new()),
            eingabezeile,
            ergebnis,
            speicher,
            debug,
        });

        let mut knoepfe = Vec::new();
        for (zeile, reihe) in TASTEN.iter().enumerate() {
            for (spalte, (text, taste, klasse)) in reihe.iter().enumerate() {
                let b = gtk::Button::with_label(text);
                b.add_css_class(klasse);
                b.set_focusable(false); // Tastatur geht immer an das Fenster
                b.set_tooltip_text(tooltip(*taste));
                let weak = Rc::downgrade(&fenster);
                let taste = *taste;
                b.connect_clicked(move |_| {
                    if let Some(f) = weak.upgrade() {
                        f.taste(taste);
                    }
                });
                grid.attach(&b, spalte as i32, zeile as i32, 1, 1);
                knoepfe.push((*text, b));
            }
        }

        fenster.install_actions();
        fenster.install_keys();
        fenster.refresh();

        if debug {
            fenster.debug_positionen(knoepfe);
        }
        fenster
    }

    /// Nur für die isolierte GUI-Prüfung (`QUERENCIA_RECHNER_DEBUG`):
    /// Positionen von Tasten und Menüköpfen relativ zum Fenster, dazu jede
    /// Zeigerbewegung – daraus bestimmt das Prüfskript die Fensterlage auf
    /// dem Bildschirm (Wayland verrät sie dem Programm nicht).
    fn debug_positionen(self: &Rc<Self>, knoepfe: Vec<(&'static str, gtk::Button)>) {
        let motion = gtk::EventControllerMotion::new();
        motion.connect_motion(|_, x, y| {
            println!("zeiger\t{x:.0}\t{y:.0}");
            use std::io::Write;
            let _ = std::io::stdout().flush();
        });
        self.window.add_controller(motion);
        let w = self.window.clone();
        // Erst melden, wenn das Fenster ein Layout hat (im headless-Betrieb
        // dauert der erste Frame länger als in der Sitzung).
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            let fertig = knoepfe
                .first()
                .and_then(|(_, b)| b.compute_bounds(&w))
                .is_some_and(|r| r.width() > 20.0 && r.y() > 0.0);
            if !fertig {
                return glib::ControlFlow::Continue;
            }
            let rechteck = |name: &str, widget: &gtk::Widget| {
                if let Some(r) = widget.compute_bounds(&w) {
                    println!(
                        "{name}\t{:.0}\t{:.0}\t{:.0}\t{:.0}",
                        r.x(),
                        r.y(),
                        r.width(),
                        r.height()
                    );
                }
            };
            for (text, b) in &knoepfe {
                rechteck(&format!("knopf\t{text}"), b.upcast_ref());
            }
            // Menüköpfe: Kinder der PopoverMenuBar in der Reihenfolge des Modells.
            if let Some(bar) = finde_widget(w.upcast_ref(), gtk::PopoverMenuBar::static_type()) {
                let mut item = bar.first_child();
                let mut i = 0;
                while let Some(it) = item {
                    rechteck(&format!("menue\t{i}"), &it);
                    i += 1;
                    item = it.next_sibling();
                }
            }
            println!("fenster\t{}\t{}", w.width(), w.height());
            use std::io::Write;
            let _ = std::io::stdout().flush();
            glib::ControlFlow::Break
        });
    }

    fn install_actions(self: &Rc<Self>) {
        let add = |name: &str, f: ActionFn| {
            let action = gio::SimpleAction::new(name, None);
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, _| {
                if let Some(s) = weak.upgrade() {
                    f(&s);
                }
            });
            self.window.add_action(&action);
        };
        add("copy", Box::new(|s| s.kopieren()));
        add("paste", Box::new(|s| s.einfuegen()));
        add("quit", Box::new(|s| s.window.close()));
        add("about", Box::new(|s| dialoge::about(&s.window)));
    }

    /// Tastatur: alle Tasten ohne Strg/Alt gehen an das Modell – außer
    /// wenn der Fokus in einem Menü liegt (dort navigiert GTK).
    fn install_keys(self: &Rc<Self>) {
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |ctrl, key, _, state| {
            let Some(f) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            if state.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK) {
                return glib::Propagation::Proceed;
            }
            // Tasten für ein offenes Menü (eigene Popup-Oberfläche) oder mit
            // Fokus im Menü gehören GTK: dort navigieren Pfeile und Eingabe.
            let im_popup =
                ctrl.current_event().and_then(|e| e.surface()) != NativeExt::surface(&f.window);
            let weiterreichen =
                im_popup || f.fokus_im_menue() || menue_offen(f.window.upcast_ref());
            if f.debug {
                println!(
                    "taste\t{}\t{}\t{}",
                    key.name().unwrap_or_default(),
                    if weiterreichen { "gtk" } else { "rechner" },
                    GtkWindowExt::focus(&f.window)
                        .map(|w| w.type_().name().to_string())
                        .unwrap_or_else(|| "kein Fokus".into())
                );
            }
            if weiterreichen {
                return glib::Propagation::Proceed;
            }
            match taste_fuer(key) {
                Some(t) => {
                    f.taste(t);
                    glib::Propagation::Stop
                }
                None => glib::Propagation::Proceed,
            }
        });
        self.window.add_controller(keys);
    }

    fn fokus_im_menue(&self) -> bool {
        GtkWindowExt::focus(&self.window)
            .is_some_and(|w| w.ancestor(gtk::Popover::static_type()).is_some())
    }

    fn taste(&self, taste: Taste) {
        self.modell.borrow_mut().druecken(taste);
        self.refresh();
    }

    fn refresh(&self) {
        let Anzeige {
            eingabezeile,
            ergebnis,
            fehler,
            speicher_belegt,
        } = self.modell.borrow().anzeige();
        self.eingabezeile.set_text(if eingabezeile.is_empty() {
            " "
        } else {
            &eingabezeile
        });
        self.ergebnis.set_text(&ergebnis);
        if fehler {
            self.ergebnis.add_css_class("qr-fehler");
        } else {
            self.ergebnis.remove_css_class("qr-fehler");
        }
        self.speicher
            .set_text(if speicher_belegt { "M" } else { "" });
        if self.debug {
            println!(
                "anzeige\t{}\t{}\t{}",
                eingabezeile.trim(),
                ergebnis,
                if speicher_belegt { "M" } else { "-" }
            );
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    fn kopieren(&self) {
        if let Some(text) = self.modell.borrow().kopierwert() {
            self.window.clipboard().set_text(&text);
            if self.debug {
                println!("kopiert\t{text}");
            }
        }
    }

    fn einfuegen(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.window
            .clipboard()
            .read_text_async(None::<&gio::Cancellable>, move |ergebnis| {
                let Some(f) = weak.upgrade() else {
                    return;
                };
                if let Ok(Some(text)) = ergebnis {
                    let ok = f.modell.borrow_mut().einfuegen(&text);
                    if f.debug {
                        println!(
                            "eingefuegt\t{}\t{}",
                            if ok { "ja" } else { "nein" },
                            text.trim()
                        );
                    }
                    f.refresh();
                }
            });
    }
}

/// Ist unter `wurzel` ein Popover (Menü) sichtbar?
fn menue_offen(wurzel: &gtk::Widget) -> bool {
    let mut kind = wurzel.first_child();
    while let Some(k) = kind {
        if (k.is::<gtk::Popover>() && k.is_visible()) || menue_offen(&k) {
            return true;
        }
        kind = k.next_sibling();
    }
    false
}

/// Erstes Widget des Typs `typ` unter `wurzel` (Tiefensuche).
fn finde_widget(wurzel: &gtk::Widget, typ: glib::Type) -> Option<gtk::Widget> {
    let mut kind = wurzel.first_child();
    while let Some(k) = kind {
        if k.type_().is_a(typ) {
            return Some(k);
        }
        if let Some(t) = finde_widget(&k, typ) {
            return Some(t);
        }
        kind = k.next_sibling();
    }
    None
}

/// Tastaturtaste → Rechnertaste (Haupt- und Ziffernblock).
fn taste_fuer(key: gdk::Key) -> Option<Taste> {
    use gdk::Key;
    Some(match key {
        Key::_0 | Key::KP_0 => Taste::Ziffer(0),
        Key::_1 | Key::KP_1 => Taste::Ziffer(1),
        Key::_2 | Key::KP_2 => Taste::Ziffer(2),
        Key::_3 | Key::KP_3 => Taste::Ziffer(3),
        Key::_4 | Key::KP_4 => Taste::Ziffer(4),
        Key::_5 | Key::KP_5 => Taste::Ziffer(5),
        Key::_6 | Key::KP_6 => Taste::Ziffer(6),
        Key::_7 | Key::KP_7 => Taste::Ziffer(7),
        Key::_8 | Key::KP_8 => Taste::Ziffer(8),
        Key::_9 | Key::KP_9 => Taste::Ziffer(9),
        Key::comma | Key::period | Key::KP_Decimal | Key::KP_Separator => Taste::Komma,
        Key::plus | Key::KP_Add => Taste::Op(Op::Plus),
        Key::minus | Key::KP_Subtract => Taste::Op(Op::Minus),
        Key::asterisk | Key::KP_Multiply | Key::multiply => Taste::Op(Op::Mal),
        Key::slash | Key::KP_Divide | Key::division => Taste::Op(Op::Geteilt),
        Key::parenleft => Taste::Auf,
        Key::parenright => Taste::Zu,
        Key::percent => Taste::Prozent,
        Key::Return | Key::KP_Enter | Key::equal | Key::KP_Equal => Taste::Gleich,
        Key::BackSpace => Taste::Ruecktaste,
        Key::Delete | Key::KP_Delete => Taste::C,
        Key::Escape => Taste::AC,
        Key::s | Key::S => Taste::Wurzel,
        Key::q | Key::Q => Taste::Quadrat,
        Key::r | Key::R => Taste::Kehrwert,
        Key::v | Key::V => Taste::Vorzeichen,
        _ => return None,
    })
}
