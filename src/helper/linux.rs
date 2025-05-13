use std::fs::read_to_string;

use log::{debug,
          error};

use crate::{helper::format_directory_path,
            BeansError};

/// all possible known directory where steam *might* be
/// only is used on linux, since windows will use the registry.
pub const STEAM_POSSIBLE_DIR: &[&str] = &[
    "~/.steam/registry.vdf",
    "~/.var/app/com.valvesoftware.Steam/.steam/registry.vdf"
];

/// find sourcemod path on linux.
/// fetches the fake registry that steam uses from find_steam_reg_path
/// and gets the value of
/// Registry/HKCU/Software/Valve/Steam/SourceModInstallPath
pub fn find_sourcemod_path() -> Result<String, BeansError>
{
    let reg_path = find_steam_reg_path()?;

    let reg_content = match read_to_string(reg_path.as_str())
    {
        Ok(v) => v,
        Err(e) =>
        {
            sentry::capture_error(&e);
            return Err(BeansError::FileOpenFailure {
                location: reg_path,
                error: e
            });
        }
    };

    for line in reg_content.lines()
    {
        if line.contains("SourceModInstallPath")
        {
            let split = &line.split("\"SourceModInstallPath\"");
            let last = split
                .clone()
                .last()
                .expect("Failed to find SourceModInstallPath")
                .trim()
                .replace("\"", "");
            return Ok(format_directory_path(last));
        }
    }

    Err(BeansError::SourceModLocationNotFound)
}
/// returns the first item in STEAM_POSSIBLE_DIR that exists. otherwise None
fn find_steam_reg_path() -> Result<String, BeansError>
{
    for x in STEAM_POSSIBLE_DIR.iter()
    {
        match simple_home_dir::home_dir()
        {
            Some(v) => match v.to_str()
            {
                Some(k) =>
                {
                    let h = format_directory_path(k.to_string());
                    let reg_loc = x.replace("~", h.as_str());
                    if crate::helper::file_exists(reg_loc.clone())
                    {
                        return Ok(reg_loc.clone());
                    }
                }
                None =>
                {
                    debug!("[helper::find_steam_reg_path] simple_home_dir::home_dir().to_str() returned None!");
                    return Err(BeansError::SteamNotFound);
                }
            },
            None =>
            {
                debug!("[helper::find_steam_reg_path] simple_home_dir::home_dir() returned None!");
                return Err(BeansError::SteamNotFound);
            }
        }
    }
    error!("Couldn't find any of the locations in STEAM_POSSIBLE_DIR");
    Err(BeansError::SteamNotFound)
}

pub fn unmark_readonly(location: String) -> Result<(), BeansError>
{
    // does nothing since this function only
    // matters for windows
    // -kate, 13th mar 2025
    Ok(())
}
