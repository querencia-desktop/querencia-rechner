//! Querencia Rechner – klassischer Taschenrechner für Querencia Desktop.
//!
//! Vorbild ist gcalctool im einfachen Modus (GNOME 2.30): Menüleiste,
//! zweizeilige Anzeige (Ausdruck und Ergebnis), feste Tastenanordnung.
//! GTK 4 ohne libadwaita und ohne eigene Titelleiste, damit labwc die
//! klassische Server-Titelleiste (Thema Querencia-Klassisch) zeichnet.
//!
//! Texte über GNU gettext (`i18n`): msgid englisch, Deutsch aus `po/de.po`.
//!
//! Aufbau: `rechenkern` (Parser und Arithmetik), `modell` (Tasten und
//! Zustand), beide ohne GTK und mit Unit-Tests; `fenster` (Oberfläche),
//! `dialoge` (Info-Dialog, Fenstersymbol).
//!
//! Eine Instanz je Sitzung (GApplication `org.querencia.Rechner`): ein zweiter
//! Aufruf holt das vorhandene Fenster nach vorn.
//!
//! `QUERENCIA_RECHNER_DEBUG=1` schreibt für die isolierte GUI-Prüfung jede
//! Anzeigeänderung und die Tastenpositionen nach stdout.

mod dialoge;
mod fenster;
mod i18n;
mod modell;
mod rechenkern;

use gtk::prelude::*;
use gtk::{gio, glib};

pub const APP_ID: &str = "org.querencia.Rechner";
pub const APP_NAME: &str = "Querencia Rechner";

fn main() -> glib::ExitCode {
    i18n::init();
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::default())
        .build();
    app.connect_startup(|app| {
        dialoge::register_window_icon();
        fenster::load_css();
        fenster::install_accels(app);
        // QD-081: GTK 4.22 wendet ein beim Start schon dunkles Farbschema
        // nicht auf das Thema an (load_named setzt es zurück); ein Neustart
        // bliebe hell. Die Meldung überträgt den Wert einmal neu (hell
        // unverändert).
        if let Some(s) = gtk::Settings::default() {
            s.notify("gtk-interface-color-scheme");
        }
    });
    app.connect_activate(|app| fenster::Fenster::get(app).present());
    app.run()
}
