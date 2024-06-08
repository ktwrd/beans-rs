use fltk::{*, prelude::*};
use fltk::app;
use crate::appvar::AppVarData;
use crate::gui;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::UpdateNoPatchDialog;

pub fn run(version: usize) {

    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = UpdateNoPatchDialog::make_window();

    ui.win.set_label(&format!("beans - Update {}", &av.mod_info.name_stylized));
    ui.label_1.set_label(&format!("No patch available for v{}. Please re-install", version));

    ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

    gui::window_ensure(&mut ui.win, 600, 125);
    gui::wait_for_quit(&app, &receive_action);
}