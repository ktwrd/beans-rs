use fltk::{*, prelude::*};
use fltk::text::TextBuffer;
use crate::appvar::AppVarData;
use crate::gui;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::InstallCompleteDialog;
use crate::workflows::INSTALL_FINISH_MSG;

pub fn run() {
    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = InstallCompleteDialog::make_window();

    ui.win.set_label(&format!("{} Install Complete", &av.mod_info.name_stylized));
    ui.label_1.set_label(&format!("{} has finished installing!", &av.mod_info.name_stylized));
    let content = av.sub(INSTALL_FINISH_MSG.to_string());
    let mut buf = TextBuffer::default();
    buf.set_text(content.as_str());
    ui.display_instructions.set_buffer(Some(buf));

    ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

    gui::window_ensure(&mut ui.win, 660, 330);
    gui::wait_for_quit(&app, &receive_action);
}