use crate::{BeansError, butler, helper, RunnerContext};
use crate::version::RemoteVersion;

pub struct UpdateWorkflow
{
    pub ctx: RunnerContext
}
impl UpdateWorkflow
{
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let current_version_id = match ctx.current_version {
            Some(v) => v,
            None => {
                println!("[UpdateWorkflow::wizard] Unable to update game since it is not installed!");
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

        let gameinfo_backup = ctx.read_gameinfo_file()?;

        if helper::sml_has_free_space(patch.tempreq)? == false {
            println!("[UpdateWorkflow::wizard] Not enough free space! Requires {}", helper::format_size(patch.tempreq));
        }

        if remote_version.signature_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Couldn't get signature URL for version {}", current_version_id);
        }
        if remote_version.heal_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Couldn't get heal URL for version {}", current_version_id);
        }
        if remote_version.signature_url.is_none() || remote_version.heal_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Unable to update, missing remote files!");
            return Ok(());
        }

        let mod_dir_location = ctx.get_mod_location();
        let staging_dir_location = ctx.get_staging_location();

        ctx.gameinfo_perms()?;
        butler::verify(
            format!("{}{}", crate::SOURCE_URL, remote_version.signature_url.unwrap()),
            mod_dir_location.clone(),
            format!("{}{}", crate::SOURCE_URL, remote_version.heal_url.unwrap()))?;
        ctx.gameinfo_perms()?;
        butler::patch_dl(
            format!("{}{}", crate::SOURCE_URL, patch.url),
            staging_dir_location,
            patch.file,
            mod_dir_location).await?;

        if let Some(gi) = gameinfo_backup {
            let loc = ctx.gameinfo_location();
            std::fs::write(&loc, gi)?;
            ctx.gameinfo_perms()?;
        }

        println!("Game has been updated!");
        Ok(())
    }
}