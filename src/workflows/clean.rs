use log::{debug,
          info,
          warn};

use crate::{helper,
            BeansError,
            RunnerContext};

#[derive(Debug, Clone)]
pub struct CleanWorkflow
{
    pub context: RunnerContext
}

impl CleanWorkflow
{
    pub fn wizard(_ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let target_directory = helper::get_tmp_dir();

        info!("[CleanWorkflow] Cleaning up {}", target_directory);
        if !helper::file_exists(target_directory.clone())
        {
            warn!("[CleanWorkflow] Temporary directory not found, nothing to clean.")
        }

        // delete directory and it's contents (and error handling)
        if let Err(e) = std::fs::remove_dir_all(&target_directory)
        {
            debug!("[CleanWorkflow::wizard] remove_dir_all {:#?}", e);
            return Err(BeansError::CleanTempFailure {
                location: target_directory,
                error: e,
                backtrace: std::backtrace::Backtrace::capture()
            });
        }

        // re-creating the temporary directory (and error handling)
        if let Err(e) = std::fs::create_dir(&target_directory)
        {
            debug!("[CleanWorkflow::wizard] create_dir {:#?}", e);
            return Err(BeansError::DirectoryCreateFailure {
                location: target_directory,
                error: e,
                backtrace: std::backtrace::Backtrace::capture()
            });
        }

        info!("[CleanWorkflow] Done!");
        Ok(())
    }
}
