use fltk::{*, prelude::*};
use crate::appvar::AppVarData;
use crate::gui;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::UpdateAlreadyLatestDialog;

pub fn run(current_version: usize, latest_version: usize) {
    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = UpdateAlreadyLatestDialog::make_window();

    ui.win.set_label(&format!("beans - Update {}", &av.mod_info.name_stylized));
    ui.label_2.set_label(&format!("You already have the latest version of {} installed.", &av.mod_info.name_stylized));
    ui.label_3.set_label(&format!("v{} installed, v{} is the latest version.", current_version, latest_version));

    ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

    gui::window_ensure(&mut ui.win, 600, 125);
    gui::wait_for_quit(&app, &receive_action);
}