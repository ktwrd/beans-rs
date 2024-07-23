use crate::{butler, helper, BeansError, RunnerContext};
use log::{debug, info};

pub struct UpdateWorkflow {
    pub ctx: RunnerContext,
}
impl UpdateWorkflow {
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError> {
        let av = crate::appvar::parse();

        let current_version_id = match ctx.current_version {
            Some(v) => v,
            None => {
                println!(
                    "[UpdateWorkflow::wizard] Unable to update game since it is not installed!"
                );
                return Ok(());
            }
        };

        let remote_version = ctx.current_remote_version()?;

        ctx.prepare_symlink()?;
        let patch = match ctx.has_patch_available() {
            Some(v) => v,
            None => {
                println!("[UpdateWorkflow::wizard] No patch is available for the version that is currently installed.");
                return Ok(());
            }
        };

        ctx.gameinfo_perms()?;

        if !helper::has_free_space(ctx.sourcemod_path.clone(), patch.clone().tempreq)? {
            println!(
                "[UpdateWorkflow::wizard] Not enough free space! Requires {}",
                helper::format_size(patch.tempreq)
            );
        }
        debug!("remote_version: {:#?}", remote_version);
        if remote_version.signature_url.is_none() {
            eprintln!(
                "[UpdateWorkflow::wizard] Couldn't get signature URL for version {}",
                current_version_id
            );
        }
        if remote_version.heal_url.is_none() {
            eprintln!(
                "[UpdateWorkflow::wizard] Couldn't get heal URL for version {}",
                current_version_id
            );
        }
        if remote_version.signature_url.is_none() || remote_version.heal_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Unable to update, missing remote files!");
            return Ok(());
        }

        let mod_dir_location = ctx.get_mod_location();
        let staging_dir_location = ctx.get_staging_location();

        helper::backup_gameinfo(ctx)?;

        ctx.gameinfo_perms()?;
        info!("[UpdateWorkflow] Verifying game");
        if let Err(e) = butler::verify(
            format!(
                "{}{}",
                &av.remote_info.base_url,
                remote_version.signature_url.unwrap()
            ),
            mod_dir_location.clone(),
            format!(
                "{}{}",
                &av.remote_info.base_url,
                remote_version.heal_url.unwrap()
            ),
        ) {
            sentry::capture_error(&e);
            return Err(e);
        }
        ctx.gameinfo_perms()?;
        info!("[UpdateWorkflow] Patching game");
        if let Err(e) = butler::patch_dl(
            format!("{}{}", &av.remote_info.base_url, patch.file),
            staging_dir_location,
            patch.file,
            mod_dir_location,
        )
        .await
        {
            sentry::capture_error(&e);
            return Err(e);
        }

        ctx.gameinfo_perms()?;

        println!("Game has been updated!");
        Ok(())
    }
}
