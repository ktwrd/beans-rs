use async_recursion::async_recursion;
use fltk::{*, prelude::*};
use log::{debug, error, trace};
use crate::{RunnerContext, BeansError, gui};
use crate::appvar::AppVarData;
use crate::gui::GUIAppStatus;
use crate::gui::install_confirm::InstallConfirmResult;
use crate::gui::wizard_ui::WizardInterface;
use crate::workflows::{InstallWorkflow, UpdateWorkflow};

#[async_recursion]
pub async fn run(ctx: &mut RunnerContext) {
    let av = AppVarData::get();
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (send_action, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = WizardInterface::make_window();

    ui.win.set_label(&format!("beans for {}", &av.mod_info.name_stylized));
    ui.label_install.set_label(&format!("Install or reinstall {}", &av.mod_info.name_stylized));
    ui.label_update.set_label(&format!("Update to the latest available version of {}", &av.mod_info.name_stylized));
    ui.label_verify.set_label(&format!("Verify the game files of {}", &av.mod_info.name_stylized));

    ui.btn_install.emit(send_action, GUIAppStatus::WizardBtnInstall);
    ui.btn_update.emit(send_action, GUIAppStatus::WizardBtnUpdate);

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
                GUIAppStatus::WizardBtnInstall => {
                    final_action = Some(GUIAppStatus::WizardBtnInstall);
                    ui.win.hide();
                    app.quit();
                },
                GUIAppStatus::WizardBtnUpdate => {
                    final_action = Some(GUIAppStatus::WizardBtnUpdate);
                    ui.win.hide();
                    app.quit();
                },
                _ => {}
            }
        }
    }
    match final_action {
        Some(GUIAppStatus::WizardBtnInstall) => {
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
                            gui::install_complete::run();
                            // This is done so we don't prompt the user once the GUI has closed.
                            unsafe {crate::HEADLESS = true;}
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
        Some(GUIAppStatus::WizardBtnUpdate) => {
            let (i, r) = ctx.latest_remote_version();
            if let Some(ci) = ctx.current_version {
                if i >= ci {
                    gui::update_alreadylatest::run(ci, i);
                    // This is done so we don't prompt the user once the GUI has closed.
                    unsafe {crate::HEADLESS = true;}
                    return;
                }
            } else {
                gui::dialog_notinstalled::run("Update");
                // This is done so we don't prompt the user once the GUI has closed.
                unsafe {crate::HEADLESS = true;}
                return;
            }
            if ctx.has_patch_available().is_none() {
                gui::update_nopatchavailable::run(i);
                // This is done so we don't prompt the user once the GUI has closed.
                unsafe {crate::HEADLESS = true;}
                return;
            }

            let mut uwf = UpdateWorkflow {
                ctx: ctx.clone()
            };
            trace!("[gui::wizard::run] User wants to update! Calling UpdateWorkflow");
        }
        _ => {}
    }
}