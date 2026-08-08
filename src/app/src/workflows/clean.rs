use std::path::Path;

use beans_core::BeansError;
use log::{debug,
          info,
          warn};

use crate::RunnerContext;

#[derive(Debug, Clone)]
pub struct CleanWorkflow
{
    pub context: RunnerContext
}

impl CleanWorkflow
{
    pub fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let target_directory = beans_core::path::get_tmp_dir();
        let staging_dir_location = ctx.get_staging_location();

        info!("[CleanWorkflow] Cleaning up {}", target_directory);
        if !beans_core::path::file_exists(target_directory.clone())
        {
            warn!("[CleanWorkflow] Temporary directory not found, nothing to clean.")
        }

        // delete directory and it's contents (and error handling)
        CleanWorkflow::cleanup_temp_path(&target_directory)?;

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

        // clean up butler files if it was interrupted in previous install
        if !beans_core::path::file_exists(staging_dir_location.clone())
        {
            debug!("[CleanWorkflow] Staging directory used by butler not found, nothing to clean.")
        }
        else
        {
            // delete temp butler directory and it's contents (and error handling)
            CleanWorkflow::cleanup_temp_path(&staging_dir_location)?;
        }

        info!("[CleanWorkflow] Done!");
        Ok(())
    }

    fn cleanup_temp_path<P: AsRef<Path>>(path: P) -> Result<(), BeansError>
    {
        // delete temp butler directory and it's contents (and error handling)
        let pref = path.as_ref();
        info!("[CleanWorkflow] Cleaning up {}", pref.display());
        if let Err(e) = std::fs::remove_dir_all(pref)
        {
            let error = BeansError::CleanTempFailure {
                location: format!("{}", pref.display()),
                error: e,
                backtrace: std::backtrace::Backtrace::capture()
            };
            debug!("[CleanWorkflow::wizard] remove_dir_all {:#?}", error);
            return Err(error);
        }
        return Ok(());
    }
}
