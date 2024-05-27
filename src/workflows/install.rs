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
        if let Some(_cv) = ctx.current_version {
            println!("[InstallWorkflow::wizard] re-installing! game files will not be touched until extraction");
        }

        let presz_loc = RunnerContext::download_package(latest_remote).await?;
        Self::install_from(presz_loc.clone(), ctx.sourcemod_path.clone(), Some(latest_remote_id)).await?;
        if helper::file_exists(presz_loc.clone()) {
            std::fs::remove_file(presz_loc)?;
        }
        Ok(())
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
            let x = AdastralVersionFile {
                version: lri.to_string()
            }.write();
            if let Err(e) = x {
                println!("[InstallWorkflow::install_from] Failed to set version to {} in .adastral", lri);
                if helper::do_debug() {
                    eprintln!("{:#?}", e);
                }
            }
        } else {
            eprintln!("Not writing .adastral since the version wasn't provided");
        }
        println!("Done! Make sure that the following is done;");
        println!("  1. Install Source SDK Base 2013 Multiplayer on Steam");
        println!("  2. Restart Steam");
        println!("  3. Play Open Fortress!");
        #[cfg(not(target_os = "windows"))]
        println!("-- Important --\nMake sure that Proton is enabled for Source SDK Base 2013 Multiplayer!");
        Ok(())
    }
}