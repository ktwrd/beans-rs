use log::{debug,
          error};

use crate::{appvar::AppVarData,
            butler,
            helper,
            version::RemoteVersion,
            BeansError,
            RunnerContext};

pub struct VerifyWorkflow
{
    pub ctx: RunnerContext
}
impl VerifyWorkflow
{
    pub async fn wizard(ctx: &mut RunnerContext) -> Result<(), BeansError>
    {
        let av = AppVarData::get();

        let current_version_id = match ctx.current_version
        {
            Some(v) => v,
            None =>
            {
                println!(
                    "[VerifyWorkflow::wizard] Unable to update game since it is not installed!"
                );
                return Ok(());
            }
        };

        let remote: RemoteVersion = ctx.current_remote_version()?;
        if remote.signature_url.is_none()
        {
            error!(
                "[VerifyWorkflow::wizard] Couldn't get signature URL for version {}",
                current_version_id
            );
        }
        if remote.heal_url.is_none()
        {
            error!(
                "[VerifyWorkflow::wizard] Couldn't get heal URL for version {}",
                current_version_id
            );
        }
        if remote.signature_url.is_none() || remote.heal_url.is_none()
        {
            error!("[VerifyWorkflow::wizard] Unable to update, missing remote files!");
            return Ok(());
        }

        helper::backup_gameinfo(ctx)?;
        let mod_dir_location = ctx.get_mod_location();
        butler::verify(
            format!(
                "{}{}",
                &av.remote_info.base_url,
                remote.signature_url.unwrap()
            ),
            mod_dir_location.clone(),
            format!("{}{}", &av.remote_info.base_url, remote.heal_url.unwrap())
        )?;
        Self::post_verify_msg();
        ctx.gameinfo_perms()?;
        Ok(())
    }
    fn post_verify_msg()
    {
        let av = AppVarData::get();
        println!("{}", av.sub(VERIFY_FINISH_MSG.to_string()));
        debug!("[VerifyWorkflow::post_verify_msg] Displayed INSTALL_FINISH_MSG");

        #[cfg(target_os = "windows")]
        winconsole::window::show(true);
        #[cfg(target_os = "windows")]
        winconsole::window::flash(winconsole::window::FlashInfo {
            count: 0,
            flash_caption: true,
            flash_tray: true,
            indefinite: false,
            rate: 0,
            until_foreground: true
        });
    }
}

#[cfg(not(target_os = "windows"))]
pub const VERIFY_FINISH_MSG: &str = include_str!("../text/verify_complete_linux.txt");
#[cfg(target_os = "windows")]
pub const VERIFY_FINISH_MSG: &str = include_str!("../text/verify_complete_windows.txt");
