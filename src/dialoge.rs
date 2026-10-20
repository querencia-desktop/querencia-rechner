//! Info-Dialog und Fenstersymbol. Dialoge sind gewöhnliche modale Fenster
//! ohne eigene Titelleiste: labwc dekoriert sie wie jedes andere Fenster.

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::APP_NAME;
use crate::i18n::{gettext, gettext_f};

/// Name des Fenstersymbols, das GTK labwc schickt (xdg-toplevel-icon).
///
/// Bewusst ein eigener, **nur als SVG** eingebetteter Name statt der App-ID:
/// Läge das App-ID-Symbol im Thema zugleich in `48x48` und in einem
/// Scalable-Verzeichnis, gäbe GTK 4.22 einen Symbolpuffer doppelt frei und
/// das Programm stürzte beim Schließen eines fokussierten Dialogs ab
/// (GTK-Symbolabsturz, geprüft im Oktober 2026). Mit nur einem SVG
/// entsteht genau ein Puffer, der Fehler tritt nicht auf.
const WINDOW_ICON: &str = "org.querencia.Rechner-fenster";

/// Bettet das Fenstersymbol als Ressource ein und macht es dem Symbolthema
/// bekannt; läuft nach dem Start von GTK.
pub fn register_window_icon() {
    let bytes = glib::Bytes::from_static(include_bytes!("../data/resource/rechner.gresource"));
    match gio::Resource::from_data(&bytes) {
        Ok(resource) => {
            gio::resources_register(&resource);
            if let Some(display) = gdk::Display::default() {
                gtk::IconTheme::for_display(&display)
                    .add_resource_path("/org/querencia/Rechner/icons");
            }
        }
        Err(err) => eprintln!("querencia-rechner: Ressourcen nicht lesbar: {err}"),
    }
}

pub fn set_window_icon(window: &impl IsA<gtk::Window>) {
    window.as_ref().set_icon_name(Some(WINDOW_ICON));
}

/// Schließt `window` erst im nächsten Leerlauf, nicht aus dem gerade
/// laufenden Tasten- oder Klick-Handler heraus.
pub fn close_later(window: &gtk::Window) {
    let window = window.clone();
    glib::idle_add_local_once(move || window.close());
}

fn modal_window(parent: &impl IsA<gtk::Window>, title: &str) -> gtk::Window {
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .resizable(false)
        .destroy_with_parent(true)
        .build();
    window.add_css_class("qr-dialog");
    // Dialoge gehören zur Anwendung (App-ID für labwc und Fensterliste).
    if let Some(app) = parent.as_ref().application() {
        window.set_application(Some(&app));
    }
    set_window_icon(&window);
    window
}

fn close_on_escape(window: &gtk::Window) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let w = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            if let Some(w) = w.upgrade() {
                close_later(&w);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
}

const LOGO_64: &[u8] = include_bytes!("../data/logo/querencia-logo-64.png");

/// „Info“: offizielles Querencia-Logo C2 (das Q), Name, Version, Lizenz.
/// Der Schließen-Knopf hat den Tastaturfokus; Escape und Eingabetaste
/// schließen.
pub fn about(parent: &impl IsA<gtk::Window>) {
    // TRANSLATORS: Fenstertitel des Info-Dialogs; {name} ist „Querencia Rechner“.
    let window = modal_window(parent, &gettext_f("About {name}", &[("name", APP_NAME)]));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_halign(gtk::Align::Center);
    if let Ok(texture) = gdk::Texture::from_bytes(&glib::Bytes::from_static(LOGO_64)) {
        let image = gtk::Image::from_paintable(Some(&texture));
        image.set_pixel_size(64);
        body.append(&image);
    }
    let name = gtk::Label::new(None);
    name.set_markup(&format!(
        "<b><big>{APP_NAME} {}</big></b>",
        env!("CARGO_PKG_VERSION")
    ));
    let text = gtk::Label::new(Some(&gettext(
        "Classic calculator for Querencia Desktop\n\
         modelled on gcalctool (GNOME 2.30).\n\n\
         Basic arithmetic, parentheses, percent, square root,\n\
         square, reciprocal and memory.",
    )));
    text.set_justify(gtk::Justification::Center);
    let tech = gtk::Label::new(Some(&format!(
        "GTK {}.{}.{}",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version()
    )));
    tech.add_css_class("dim-label");
    let license = gtk::Label::new(Some(&gettext("License: GNU GPL, version 3 or later")));
    license.add_css_class("dim-label");
    body.append(&name);
    body.append(&text);
    body.append(&tech);
    body.append(&license);
    let close = gtk::Button::with_mnemonic(&gettext("_Close"));
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_halign(gtk::Align::End);
    row.set_margin_top(12);
    row.append(&close);
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_margin_top(12);
    outer.set_margin_bottom(12);
    outer.set_margin_start(18);
    outer.set_margin_end(18);
    outer.append(&body);
    outer.append(&row);
    window.set_child(Some(&outer));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    window.present();
    close.grab_focus();
}
