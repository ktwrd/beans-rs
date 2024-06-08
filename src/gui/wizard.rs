use async_recursion::async_recursion;
use fltk::{*, prelude::*};
use log::{debug, error, trace};
use crate::{RunnerContext, BeansError, gui};
use crate::appvar::AppVarData;
use crate::gui::GUIAppStatus;
use crate::gui::install_confirm::InstallConfirmResult;
use crate::gui::wizard_ui::WizardInterface;
use crate::workflows::InstallWorkflow;

#[async_recursion]
pub async fn run(ctx: &mut RunnerContext) {
    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = WizardInterface::make_window();

    ui.win.set_label(&format!("beans for {}", &av.mod_info.name_stylized));
    ui.label_install.set_label(&format!("Install or reinstall {}", &av.mod_info.name_stylized));
    ui.label_update.set_label(&format!("Update to the latest available version of {}", &av.mod_info.name_stylized));
    ui.label_verify.set_label(&format!("Verify the game files of {}", &av.mod_info.name_stylized));

    ui.btn_install.emit(send_action, GUIAppStatus::Wizard_BtnInstall);

    gui::window_ensure(&mut ui.win, 640, 250);
    let mut final_action: Option<GUIAppStatus> = None;
    while app.wait() {
        if let Some(action) = receive_action.recv() {
            trace!("[receive_action] {:#?}", action);
            match action {
                GUIAppStatus::Quit => {
                    trace!("[gui::wizard::run] Received quit event!");
                    app.quit();
                },
                GUIAppStatus::Wizard_BtnInstall => {
                    final_action = Some(GUIAppStatus::Wizard_BtnInstall);
                    ui.win.hide();
                    trace!("[gui::wizard::run] Attempting to quit");
                    app.quit();
                }
                _ => {}
            }
        }
    }
    match final_action {
        Some(GUIAppStatus::Wizard_BtnInstall) => {
            let (i, r) = ctx.latest_remote_version();
            trace!("[gui::wizard::run] Showing confirm dialog for v{}", i);
            match gui::install_confirm::run(ctx, i, r).await {
                InstallConfirmResult::Continue => {
                    let mut iwf = InstallWorkflow {
                        context: ctx.clone()
                    };
                    trace!("[gui::wizard::run] User wants to install! Calling RunnerContext");
                    match iwf.install_version(i).await {
                        Ok(_) => {
                            todo!("Show dialog with the post-install instructions");
                        },
                        Err(e) => {
                            error!("[gui::wizard::run] Failed to run InstallWorkflow::install_version({i}) {:#?}", e);
                            panic!("[gui::wizard::run] Failed to run InstallWorkflow::install_version({i}) {:#?}", e);
                        }
                    };
                },
                InstallConfirmResult::Cancel => {
                    debug!("[gui::wizard::run] User clicked on the Cancel button, showing Wizard again");
                    run(ctx).await;
                }
            }
        },
        _ => {}
    }
}