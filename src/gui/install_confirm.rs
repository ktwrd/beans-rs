use fltk::{*, prelude::*};
use log::{debug, error};
use crate::appvar::AppVarData;
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::InstallConfirmInterface;
use crate::{helper, RunnerContext};
use crate::version::RemoteVersion;

#[derive(Clone, Copy, Debug)]
pub enum InstallConfirmResult {
    Continue,
    Cancel
}
/// Will return `Continue` when the `Install` button was clicked and we have enough space.
pub async fn run(ctx: &RunnerContext, version_id: usize, version_details: RemoteVersion)
    -> InstallConfirmResult {
    let mut result = InstallConfirmResult::Cancel;
    let av = AppVarData::get();

    // Initialize the sizes that are displayed.
    let dl_size = match version_details.get_download_size().await {
        Some(x) => {
            Some(x as usize)
        },
        None => None
    };
    let dl_size_txt = match dl_size {
        Some(x) => helper::format_size(x),
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
        Some(x) => helper::format_size(x),
        None => String::from("<unknown>")
    };
    let total_size = dl_size.unwrap_or(0usize) + install_size.unwrap_or(0usize);

    // Initialize app & window.
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
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

    crate::gui::window_ensure(&mut ui.win, 640, 250);
    while app.wait() {
        if let Some(action) = receive_action.recv() {
            match action {
                GUIAppStatus::Quit => {
                    app.quit();
                    return result;
                },
                GUIAppStatus::BtnContinue => {
                    app.quit();
                    return match has_space {
                        true => InstallConfirmResult::Continue,
                        false => InstallConfirmResult::Cancel
                    };
                },
                GUIAppStatus::BtnCancel => {
                    app.quit();
                    return InstallConfirmResult::Cancel;
                },
                _ => {}
            }
        }
    }
    app.quit();
    return result;
}