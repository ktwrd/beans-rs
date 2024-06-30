use fltk::{*, prelude::*};
use crate::gui;
use crate::gui::GUIAppStatus;
use crate::gui::shared_ui::GenericDialog;

pub fn run(title: &str, label: &str) {
    if crate::has_gui_support() == false {
        println!("============ {} ============", title);
        println!("{}", label);
        return;
    }
    
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = GenericDialog::make_window();

    ui.win.set_label(title);
    ui.label.set_label(label);
    ui.btn_ok.set_size(70, 24);
    ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

    let (label_w, label_h) = ui.label.measure_label();
    ui.win.set_size(
        25 + label_w + 25,
        10 + label_h + 5 + ui.btn_ok.height() + 5
    );

    ui.btn_ok.set_pos(25, ui.win.height() - 24 - 5);
    gui::window_centre_screen(&mut ui.win);
    ui.win.handle(move |w, ev| match ev {
        fltk::enums::Event::Resize => {
            let height = w.height();
            ui.btn_ok.set_pos(25, height - 24 - 5);
            ui.btn_ok.set_size(70, 24);
            let (lw, lh) = ui.label.measure_label();
            if w.width() > lw+50 {
                w.set_size(lw+50, 10+lh+5+ ui.btn_ok.height() + 5);
            }
            true
        },
        _ => false
    });
    ui.win.make_resizable(true);
    ui.win.show();
    gui::wait_for_quit(&app, &receive_action);
}
