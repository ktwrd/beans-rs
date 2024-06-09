use fltk::{*, prelude::*};
use log::{error, trace};
use crate::gui::GUIAppStatus;
use crate::{gui, helper, RunnerContext};
use crate::appvar::AppVarData;
use crate::gui::wizard_ui::GenericConfirmDialog;
use crate::version::{RemotePatch, RemoteVersion};

#[derive(Clone, Debug)]
pub enum DialogConfirmType
{
    Install(ConfirmInstallDialogDetails),
    Update(ConfirmUpdateDialogDetails)
}
#[derive(Clone, Debug)]
pub struct ConfirmInstallDialogDetails {
    pub id: usize,
    pub details: RemoteVersion,
    pub total_size_required: Option<usize>
}
impl ConfirmInstallDialogDetails {
    pub fn has_space(&self, ctx: &mut RunnerContext) -> bool {
        if let Some(x) = self.total_size_required {
            match helper::has_free_space(ctx.sourcemod_path.clone(), x as u64) {
                Ok(x) => x,
                Err(e) => {
                    error!("[ConfirmInstallDialogDetails::has_space] Failed to calculate free space on {} (required {}b)\n{:#?}", ctx.sourcemod_path.clone(), x, e);
                    false
                }
            }
        } else {
            false
        }
    }
    pub async fn set_details(&mut self, ctx: &mut RunnerContext, ui: &mut GenericConfirmDialog) {
        let av = AppVarData::get();

        // Initialize the sizes that are displayed.
        let dl_size = match self.details.get_download_size().await {
            Some(x) => {
                Some(x as usize)
            },
            None => None
        };
        let dl_size_txt = match dl_size {
            Some(x) => helper::format_size(x as u64),
            None => String::from("<unknown>")
        };
        let install_size = match &self.details.post_sz {
            Some(post) => {
                let ps = post.clone();
                match &self.details.pre_sz {
                    Some(pre) => Some(ps - pre.clone()),
                    None => Some(ps)
                }
            },
            None => None
        };
        let install_size_txt = match install_size {
            Some(x) => helper::format_size(x as u64),
            None => String::from("<unknown>")
        };
        let total_size = dl_size.unwrap_or(0usize) + install_size.unwrap_or(0usize);
        self.total_size_required = Some(total_size);
        let has_space = self.has_space(ctx);

        ui.win.set_label(&format!("beans - {}", av.mod_info.name_stylized));
        ui.label_title.set_label(&format!("{} (v{})", &av.mod_info.name_stylized, &self.id));
        ui.btn_continue.set_label("Install");
        ui.btn_cancel.set_label("Cancel");
        if has_space {
            ui.label_desc1.set_label(&format!("Download Size: {}", dl_size_txt));
            ui.label_desc2.set_label(&format!("Install Size: {}", install_size_txt));
            ui.label_desc3.set_label(&format!("Total Disk Space Required: {}", helper::format_size(total_size as u64)));
        } else {
            ui.label_desc1.set_label(&format!("Not enough free space! {} is required.", helper::format_size(total_size as u64)));
            ui.label_desc2.set_label("");
            ui.label_desc3.set_label("");
            ui.btn_continue.deactivate();
        }
    }
}
#[derive(Clone, Debug)]
pub struct ConfirmUpdateDialogDetails {
    pub id_target: usize,
    pub id_source: usize,
    pub dl_size: u64,
    pub details: RemotePatch
}
impl ConfirmUpdateDialogDetails {
    pub fn has_space(&self, ctx: &mut RunnerContext) -> bool {
        match helper::has_free_space(ctx.sourcemod_path.clone(), self.dl_size) {
            Ok(x) => x,
            Err(e) => {
                error!("[ConfirmUpdateDialogDetails::has_space] Failed to calculate free space on {} (required {}b)\n{:#?}", ctx.sourcemod_path.clone(), self.dl_size, e);
                false
            }
        }
    }
    pub async fn set_details(&mut self, ctx: &mut RunnerContext, ui: &mut GenericConfirmDialog) {
        let av = AppVarData::get();
        let dl_size = match self.details.get_download_size().await {
            Some(x) => {
                Some(x)
            },
            None => None
        };
        if let Some(x) = dl_size {
            self.dl_size = x;
        }
        let dl_size_txt = match dl_size {
            Some(x) => helper::format_size(x),
            None => String::from("<unknown>")
        };
        let has_space = self.has_space(ctx);

        ui.win.set_label(&format!("beans - {}", &av.mod_info.name_stylized));
        ui.label_title.set_label(&format!("Update {}", &av.mod_info.name_stylized));
        ui.btn_continue.set_label("Update");
        ui.btn_cancel.set_label("Cancel");
        ui.label_desc1.set_label(&format!("Updating from v{} to v{}", self.id_source, self.id_target));
        if has_space {
            ui.label_desc2.set_label(&format!("Download Size: {}", dl_size_txt));
            ui.label_desc3.set_label("");
        } else {
            ui.label_desc2.set_label(&format!("Required Disk Space: {}", dl_size_txt));
            ui.label_desc3.set_label("Unable to install. Not enough space");
            ui.btn_continue.deactivate();
        }
    }
}
#[derive(Clone, Debug)]
pub enum DialogResult {
    None = 0,
    OK = 1,
    Cancel = 2,
    Abort = 3,
    Retry = 4,
    Ignore = 5,
    Yes = 6,
    No = 7,
    TryAgain = 10,
    Continue = 11,
}
impl From<DialogResult> for GUIAppStatus {
    fn from(item: DialogResult) -> Self {
        match item {
            DialogResult::None => Self::UnknownStatus,
            DialogResult::OK => Self::BtnOk,
            DialogResult::Cancel => Self::BtnCancel,
            DialogResult::Abort => Self::BtnAbort,
            DialogResult::Retry => Self::BtnRetry,
            DialogResult::Ignore => Self::BtnIgnore,
            DialogResult::Yes => Self::BtnYes,
            DialogResult::No => Self::BtnNo,
            DialogResult::TryAgain => Self::BtnTryAgain,
            DialogResult::Continue => Self::BtnContinue
        }
    }
}
pub async fn run(ctx: &RunnerContext, confirm_type: &mut DialogConfirmType) -> DialogResult {
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    gui::apply_app_scheme();
    let (s, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = GenericConfirmDialog::make_window();

    let mut ctx_c = ctx.clone();
    match confirm_type {
        DialogConfirmType::Install(details) => {
            details.set_details(&mut ctx_c, &mut ui).await;
        },
        DialogConfirmType::Update(details) => {
            details.set_details(&mut ctx_c, &mut ui).await;
        }
    };

    ui.btn_continue.emit(s, GUIAppStatus::BtnContinue);
    ui.btn_cancel.emit(s, GUIAppStatus::BtnCancel);

    gui::window_ensure(&mut ui.win, 550, 150);
    let mut return_value = DialogResult::Cancel;
    while app.wait() {
        if let Some(action) = receive_action.recv() {
            match action {
                GUIAppStatus::Quit => {
                    app.quit();
                },
                GUIAppStatus::BtnContinue => {
                    ui.win.platform_hide();
                    return_value = match confirm_type {
                        DialogConfirmType::Install(details) => {
                            match details.has_space(&mut ctx_c) {
                                true => DialogResult::Continue,
                                false => DialogResult::Cancel
                            }
                        },
                        DialogConfirmType::Update(details) => {
                            match details.has_space(&mut ctx_c) {
                                true => DialogResult::Continue,
                                false => DialogResult::Cancel
                            }
                        },
                        _ => DialogResult::Continue
                    };
                    trace!("[dialog_confirm::run->BtnContinue] Set return value to {:#?}", return_value);
                    app.quit();
                },
                GUIAppStatus::BtnCancel => {
                    ui.win.platform_hide();
                    return_value = DialogResult::Cancel;
                    app.quit();
                },
                _ => {}
            }
        }
    };
    return return_value;
}