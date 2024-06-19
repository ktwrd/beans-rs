use async_recursion::async_recursion;
use fltk::{*, prelude::*};
use log::{debug, error, trace};
use crate::{RunnerContext, gui};
use crate::appvar::AppVarData;
use crate::gui::dialog_confirm::{ConfirmInstallDialogDetails, ConfirmUpdateDialogDetails, DialogConfirmType, DialogResult};
use crate::gui::{dialog_generic, GUIAppStatus};
use crate::gui::wizard_ui::WizardInterface;
use crate::version::RemoteVersion;
use crate::workflows::{InstallWorkflow, UpdateWorkflow, VerifyWorkflow};

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
    ui.btn_verify.emit(send_action, GUIAppStatus::WizardBtnVerify);

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
                    ui.win.platform_hide();
                    app.quit();
                },
                GUIAppStatus::WizardBtnUpdate => {
                    final_action = Some(GUIAppStatus::WizardBtnUpdate);
                    ui.win.platform_hide();
                    app.quit();
                },
                GUIAppStatus::WizardBtnVerify => {
                    final_action = Some(GUIAppStatus::WizardBtnVerify);
                    ui.win.platform_hide();
                    app.quit();
                }
                _ => {}
            }
        }
    }
    let mut x = ctx.clone();
    match final_action {
        Some(GUIAppStatus::WizardBtnInstall) => {
            btn_install_latest(&mut x).await;
        },
        Some(GUIAppStatus::WizardBtnUpdate) => {
            btn_update(&mut x).await;
        },
        Some(GUIAppStatus::WizardBtnVerify) => {
            btn_verify(&mut x).await;
        },
        _ => {}
    }
}
/// Install (or reinstall) the latest version.
async fn btn_install_latest(ctx: &mut RunnerContext) {
    let (i, r) = ctx.latest_remote_version();
    trace!("[gui::wizard::btn_install] Showing confirm dialog for v{}", i);
    btn_install(ctx, i, r).await;
}
async fn btn_install(ctx: &mut RunnerContext, id: usize, details: RemoteVersion) {
    let mut ct = DialogConfirmType::Install(ConfirmInstallDialogDetails {
        id,
        details: details.clone(),
        total_size_required: None
    });
    match gui::dialog_confirm::run(ctx, &mut ct).await {
        DialogResult::Continue => {
            let mut iwf = InstallWorkflow {
                context: ctx.clone()
            };
            trace!("[gui::wizard::btn_install] User wants to install! Calling RunnerContext");
            match iwf.install_version(id).await {
                Ok(_) => {
                    gui::install_complete::run();
                    // This is done so we don't prompt the user once the GUI has closed.
                    unsafe {crate::HEADLESS = true;}
                },
                Err(e) => {
                    error!("[gui::wizard::btn_install] Failed to run InstallWorkflow::install_version({id}) {:#?}", e);
                    panic!("[gui::wizard::btn_install] Failed to run InstallWorkflow::install_version({id}) {:#?}", e);
                }
            };
        },
        _ => {
            debug!("[gui::wizard::btn_install] User clicked on the Cancel button, showing Wizard again");
            run(ctx).await;
        }
    }
}
async fn btn_update(ctx: &mut RunnerContext) {
    let av = AppVarData::get();
    let (i, _) = ctx.latest_remote_version();
    if let Some(ci) = ctx.current_version {
        if i <= ci {
            gui::dialog_generic::run(
                &format!("beans - Update {}", &av.mod_info.name_stylized),
                &format!("Latest version of {} is already installed (v{})", &av.mod_info.name_stylized, ci)
            );
            // This is done so we don't prompt the user once the GUI has closed.
            unsafe {crate::HEADLESS = true;}
            return;
        }
    } else {
        gui::dialog_generic::run(
            &format!("beans - Update {}", &av.mod_info.name_stylized),
            &format!("Unable to update since {} is not installed.", &av.mod_info.name_stylized)
        );
        // This is done so we don't prompt the user once the GUI has closed.
        unsafe {crate::HEADLESS = true;}
        return;
    }

    let target_patch = match ctx.has_patch_available() {
        Some(p) => p,
        None => {
            gui::dialog_generic::run(
                &format!("beans - Update {}", &av.mod_info.name_stylized),
                &format!("No patch available for {} v{}. Please re-install.", &av.mod_info.name_stylized, i)
            );
            // This is done so we don't prompt the user once the GUI has closed.
            unsafe {crate::HEADLESS = true;}
            return;
        }
    };

    let mut ct = DialogConfirmType::Update(ConfirmUpdateDialogDetails {
        id_target: i,
        id_source: ctx.current_version.unwrap_or(0),
        dl_size: 0,
        details: target_patch.clone()
    });
    match gui::dialog_confirm::run(ctx, &mut ct).await {
        DialogResult::Continue => {
            trace!("[gui::wizard::btn_update] User wants to update! Calling UpdateWorkflow");
            match UpdateWorkflow::wizard(ctx).await {
                Ok(()) => {
                    gui::dialog_generic::run(
                        &format!("beans - Update {}", &av.mod_info.name_stylized),
                        &format!("{} has successfully been updated to v{}", &av.mod_info.name_stylized, i)
                    );
                },
                Err(e) => {
                    let (x, y) = gui::get_center_screen();
                    error!("[gui::wizard::btn_update] Failed to run UpdateWorkflow::wizard {:#?}", e);
                    dialog::message(x, y, &format!("Failed to Update. {:}", e));
                }
            };
        },
        _ => {
            debug!("[gui::wizard::btn_update] User clicked on the Cancel button, showing Wizard again");
            run(ctx).await;
        }
    }
}
async fn btn_verify(ctx: &mut RunnerContext) {
    let av = AppVarData::get();

    let current_version = match ctx.current_version {
        Some(x) => x,
        None => {
            dialog_generic::run(
                "beans - Failed to Verify",
                &format!("Unable to verify game files since {} is not installed.", av.mod_info.name_stylized));
            run(ctx).await;
            return;
        }
    };

    match ctx.current_remote_version() {
        Ok(x) => {
            if x.heal_url.is_none() {
                dialog_generic::run(
                    "beans - Failed to Verify",
                    &format!("Missing heal URL for {} v{}", av.mod_info.name_stylized, current_version));
                run(ctx).await;
                return;
            }
            if x.signature_url.is_none() {
                dialog_generic::run(
                    "beans - Failed to Verify",
                    &format!("Missing signature URL for {} v{}", av.mod_info.name_stylized, current_version));
                run(ctx).await;
                return;
            }
        },
        Err(e) => {
            error!("[gui::wizard::btn_verify] Failed to get current version details. {:#?}", e);
            dialog_generic::run(
                "beans - Failed to Verify",
                &format!("Failed to get current version details for {} v{}.\nReason: {:}",
                         av.mod_info.name_stylized,
                         current_version,
                         e));
            return;
        }
    };

    if let Err(e) = VerifyWorkflow::wizard(ctx).await {
        error!("[gui::wizard::btn_verify] Failed to run VerifyWorkflow {:#?}", e);
        dialog_generic::run(
            "beans - Failed to Verify",
            &format!("Failed to verify {}.\nReason: {:}", av.mod_info.name_stylized, e));
    } else {
        dialog_generic::run(
            "beans - Verification",
            &format!("Verification for {} has completed.\nAny corruption has been repaired", av.mod_info.name_stylized));
    }
}