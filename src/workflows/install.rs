use crate::{DownloadFailureReason, helper, RunnerContext};
use crate::BeansError;
use crate::version::AdastralVersionFile;

#[derive(Debug, Clone)]
pub struct InstallWorkflow {
    pub context: RunnerContext
}
impl InstallWorkflow {
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let (latest_remote_id, latest_remote) = ctx.latest_remote_version();
        if let Some(cv) = ctx.current_version {
            if latest_remote_id < cv {
                println!("Installed version is newer than the latest remote version? (local: {}, remote: {})", cv, latest_remote_id);
                return Err(BeansError::LatestVersionAlreadyInstalled {
                    current: cv,
                    latest: latest_remote_id
                });
            }
            if latest_remote_id == cv {
                println!("You've got the latest version installed already! (local: {}, remote: {})", cv, latest_remote_id);
                return Err(BeansError::LatestVersionAlreadyInstalled {
                    current: cv,
                    latest: latest_remote_id
                });
            }
        }

        let presz_loc = RunnerContext::download_package(latest_remote).await?;
        Self::install_from(presz_loc, ctx.sourcemod_path.clone(), Some(latest_remote_id)).await
    }

    /// Install the `.tar.zstd` file at `package_loc` to `out_dir`
    pub async fn install_from(package_loc: String, out_dir: String, version_id: Option<usize>) -> Result<(), BeansError>
    {
        if helper::file_exists(package_loc.clone()) == false {
            eprintln!("[InstallWorkflow::Wizard] Failed to find package! (location: {package_loc})");
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::FileNotFound {
                    location: package_loc.clone()
                }
            });
        }

        println!("[InstallWorkflow::Wizard] Extracting game");
        RunnerContext::extract_package(package_loc, out_dir.clone())?;
        if let Some(lri) = version_id {
            AdastralVersionFile {
                version: lri.to_string()
            }.write()?;
        } else {
            eprintln!("Not writing .adastral since the version wasn't provided");
        }
        println!("Done! Make sure to restart steam before playing");
        Ok(())
    }
}