use log::{error, info, trace};
use crate::{BeansError, helper, RunnerContext};
use crate::appvar::AppVarData;

#[derive(Debug, Clone)]
pub struct UninstallWorkflow {
    pub context: RunnerContext
}
impl UninstallWorkflow {
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let av = AppVarData::get();
        if ctx.current_version.is_none() {
            info!("No version is currently installed.");
            return Ok(());
        }

        let mod_location = ctx.get_mod_location();
        if let Some(pid) = helper::is_game_running(mod_location.clone()) {
            info!("{} (pid: {:}) is running! Can't uninstall since the game files are being used.", av.mod_info.name_stylized, pid);
            return Err(BeansError::GameStillRunning {
                name: av.mod_info.name_stylized.clone(),
                pid: format!("{:}", pid)
            });
        }

        if let Err(e) = std::fs::remove_dir_all(&mod_location) {
            trace!("{:#?}", e);
            error!("[UninstallWorkflow] Failed to delete mod directory {} ({:})", mod_location, e);
            return Err(BeansError::DirectoryDeleteFailure {
                location: mod_location,
                error: e
            });
        }

        info!("[UninstallWorkflow] Successfully uninstalled {}. Please restart Steam.", av.mod_info.name_stylized);
        return Ok(());
    }
}