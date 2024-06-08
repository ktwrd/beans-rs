use fltk::{*, prelude::*};
use fltk::app;
use crate::appvar::AppVarData;
use crate::gui;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::NotInstalledDialog;

pub fn run(action: &str) {

    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = NotInstalledDialog::make_window();

    ui.win.set_label(&format!("beans - {} {}", action, &av.mod_info.name_stylized));
    ui.label_1.set_label(&format!("Unable to {} since {} is not installed.", action, &av.mod_info.name_stylized));

    ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

    gui::window_ensure(&mut ui.win, 600, 125);
    gui::wait_for_quit(&app, &receive_action);
}