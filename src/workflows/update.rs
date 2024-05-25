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

        let mut signature_url: Option<String> = None;
        let mut heal_url: Option<String> = None;
        let mut target_remote_version: Option<(usize, RemoteVersion)> = None;
        for (v, i) in ctx.remote_version_list.clone().versions.into_iter() {
            if v == current_version_id && target_remote_version.is_none() {
                if let Some(x) = i.clone().signature_url {
                    signature_url = Some(format!("{}{}", crate::SOURCE_URL, x));
                }
                if let Some(x) = i.clone().heal_url {
                    heal_url = Some(format!("{}{}", crate::SOURCE_URL, x));
                }
                target_remote_version = Some((v, i.clone()));
            }
        }
        if target_remote_version.is_none() {
            println!("[UpdateWorkflow::wizard] Current version does not exist on server, patch might not be available.");
        }

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

        if signature_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Couldn't get signature URL for version {}", current_version_id);
        }
        if heal_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Couldn't get heal URL for version {}", current_version_id);
        }
        if signature_url.is_none() || heal_url.is_none() {
            eprintln!("[UpdateWorkflow::wizard] Unable to update, missing remote files!");
            return Ok(());
        }

        let mod_dir_location = ctx.get_mod_location();
        let staging_dir_location = ctx.get_staging_location();

        ctx.gameinfo_perms()?;
        butler::verify(signature_url.unwrap(), mod_dir_location.clone(), heal_url.unwrap())?;
        ctx.gameinfo_perms()?;
        butler::patch_dl(format!("{}{}", crate::SOURCE_URL, patch.url), staging_dir_location, patch.file, mod_dir_location).await?;

        if let Some(gi) = gameinfo_backup {
            let loc = ctx.gameinfo_location();
            std::fs::write(&loc, gi)?;
            ctx.gameinfo_perms()?;
        }

        println!("Game has been updated!");
        Ok(())
    }
}