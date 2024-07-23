use crate::{helper, BeansError, RunnerContext};
use log::{info, warn};

#[derive(Debug, Clone)]
pub struct CleanWorkflow {
    pub context: RunnerContext,
}
impl CleanWorkflow {
    pub fn wizard(_ctx: &mut RunnerContext) -> Result<(), BeansError> {
        let target_directory = helper::get_tmp_dir();

        info!("[CleanWorkflow] Cleaning up {}", target_directory);
        if helper::file_exists(target_directory.clone()) == false {
            warn!("[CleanWorkflow] Temporary directory not found, nothing to clean.")
        }

        // delete directory and it's contents (and error handling)
        if let Err(e) = std::fs::remove_dir_all(&target_directory) {
            return Err(BeansError::CleanTempFailure {
                location: target_directory,
                error: e,
            });
        }

        // re-creating the temporary directory (and error handling)
        if let Err(e) = std::fs::create_dir(&target_directory) {
            return Err(BeansError::DirectoryCreateFailure {
                location: target_directory,
                error: e,
            });
        }

        info!("[CleanWorkflow] Done!");
        return Ok(());
    }
}
