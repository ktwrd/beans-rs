use fltk::{*, prelude::*};
use log::{debug, error};
use crate::appvar::AppVarData;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::InstallConfirmInterface;
use crate::{gui, helper, RunnerContext};
use crate::version::RemoteVersion;

#[derive(Clone, Copy, Debug)]
pub enum InstallConfirmResult {
    Continue,
    Cancel
}
/// Will return `Continue` when the `Install` button was clicked and we have enough space.
pub async fn run(ctx: &RunnerContext, version_id: usize, version_details: RemoteVersion)
    -> InstallConfirmResult {
    let av = AppVarData::get();

    // Initialize the sizes that are displayed.
    let dl_size = match version_details.get_download_size().await {
        Some(x) => {
            Some(x as usize)
        },
        None => None
    };
    let dl_size_txt = match dl_size {
        Some(x) => helper::format_size(x as u64),
        None => String::from("<unknown>")
    };
    let install_size = match version_details.post_sz {
        Some(post) => {
            match version_details.pre_sz {
                Some(pre) => Some(post - pre),
                None => Some(post)
            }
        },
        None => None
    };
    let install_size_txt = match install_size {
        Some(x) => helper::format_size(x as u64),
        None => String::from("<unknown>")
    };
    let total_size = (dl_size.unwrap_or(0usize) + install_size.unwrap_or(0usize)) as u64;

    // Initialize app & window.
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (s, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = InstallConfirmInterface::make_window();

    ui.win.set_label(&format!("beans - {}", &av.mod_info.name_stylized));
    let has_space = match helper::has_free_space(ctx.sourcemod_path.clone(), total_size) {
        Ok(x) => x,
        Err(e) => {
            error!("[gui::install_confirm::run] Failed to calculate free space on {} (required {}b)\n{:#?}", ctx.sourcemod_path.clone(), total_size, e);
            false
        }
    };

    // Set labels depending on how much space is available.
    ui.label_1.set_label(&format!("{} (v{})", &av.mod_info.name_stylized, version_id));
    if has_space {
        ui.label_2.set_label(&format!("Download Size: {}", dl_size_txt));
        ui.label_3.set_label(&format!("Install Size: {}", install_size_txt));
        ui.label_4.set_label(&format!("Total Disk Space Required: {}", helper::format_size(total_size)));
    } else {
        ui.label_2.set_label(&format!("Not enough free space! {} is required.", helper::format_size(total_size)));
        ui.label_3.set_label("");
        ui.label_4.set_label("");
        ui.btn_install.deactivate();
    }
    debug!("[gui::install_confirm::run] has_space: {has_space}");

    ui.btn_install.emit(s, GUIAppStatus::BtnContinue);
    ui.btn_cancel.emit(s, GUIAppStatus::BtnCancel);

    gui::window_ensure(&mut ui.win, 570, 150);
    let mut return_value = InstallConfirmResult::Cancel;
    while app.wait() {
        if let Some(action) = receive_action.recv() {
            match action {
                GUIAppStatus::Quit => {
                    ui.win.hide();
                    app.quit();
                },
                GUIAppStatus::BtnContinue => {
                    ui.win.hide();
                    app.quit();
                    return_value = match has_space {
                        true => InstallConfirmResult::Continue,
                        false => InstallConfirmResult::Cancel
                    };
                },
                GUIAppStatus::BtnCancel => {
                    ui.win.hide();
                    app.quit();
                    return_value = InstallConfirmResult::Cancel;
                },
                _ => {}
            }
        }
    }
    ui.win.hide();
    app.quit();
    return return_value;
}