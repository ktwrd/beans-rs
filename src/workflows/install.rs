use log::{debug, error, warn};
use crate::{DownloadFailureReason, helper, RunnerContext};
use crate::BeansError;
use crate::version::{AdastralVersionFile, RemoteVersion};

#[derive(Debug, Clone)]
pub struct InstallWorkflow {
    pub context: RunnerContext
}
impl InstallWorkflow {
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let (latest_remote_id, latest_remote) = ctx.latest_remote_version();
        if let Some(_cv) = ctx.current_version {
            println!("[InstallWorkflow::wizard] re-installing! game files will not be touched until extraction");
        }

        Self::install_with_remote_version(ctx, latest_remote_id, latest_remote).await
    }

    /// Install the specified version by its ID to the output directory.
    pub async fn install_version(&mut self, version_id: usize) -> Result<(), BeansError>
    {
        let target_version = match self.context.remote_version_list.versions.get(&version_id) {
            Some(v) => v,
            None => {
                error!("Could not find remote version {version_id}");
                return Err(BeansError::RemoteVersionNotFound {
                    version: Some(version_id)
                });
            }
        };
        let mut ctx = self.context.clone();
        InstallWorkflow::install_with_remote_version(&mut ctx, version_id, target_version.clone()).await
    }

    pub async fn install_with_remote_version(ctx: &mut RunnerContext, version_id: usize, version: RemoteVersion)
        -> Result<(), BeansError>
    {
        println!("{:=>60}\nInstalling version {} to {}\n{0:=>60}", "=", version_id, &ctx.sourcemod_path);
        let presz_loc = RunnerContext::download_package(version, crate::ctx::DownloadSource::Install).await?;
        Self::install_from(presz_loc.clone(), ctx.sourcemod_path.clone(), Some(version_id)).await?;
        if helper::file_exists(presz_loc.clone()) {
            std::fs::remove_file(presz_loc)?;
        }
        Ok(())
    }

    /// Install the `.tar.zstd` file at `package_loc` to `out_dir`
    /// package_loc: Location to a file that is a `.tar.zstd` file.
    /// out_dir: should be `RunnerContext.sourcemod_path`
    /// version_id: Version that is from `package_loc`. When not specified, `.adastral` will not be written to.
    /// Note: This function doesn't check the extension when extracting.
    pub async fn install_from(package_loc: String, out_dir: String, version_id: Option<usize>)
        -> Result<(), BeansError>
    {
        if helper::file_exists(package_loc.clone()) == false {
            error!("[InstallWorkflow::Wizard] Failed to find package! (location: {package_loc})");
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::FileNotFound {
                    location: package_loc.clone()
                }
            });
        }

        println!("[InstallWorkflow::Wizard] Extracting to {out_dir}");
        RunnerContext::extract_package(package_loc, out_dir.clone())?;
        if let Some(lri) = version_id {
            let x = AdastralVersionFile {
                version: lri.to_string()
            }.write(Some(out_dir.clone()));
            if let Err(e) = x {
                println!("[InstallWorkflow::install_from] Failed to set version to {} in .adastral", lri);
                debug!("{:#?}", e);
            }
        } else {
            warn!("Not writing .adastral since the version wasn't provided");
        }
        let av = crate::appvar::parse();
        println!("{}", av.sub(INSTALL_FINISH_MSG.to_string()));
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub const INSTALL_FINISH_MSG: &str = include_str!("../text/install_complete_linux.txt");
#[cfg(target_os = "windows")]
pub const INSTALL_FINISH_MSG: &str = include_str!("../text/install_complete_windows.txt");