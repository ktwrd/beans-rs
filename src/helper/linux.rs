use std::fs::read_to_string;
use crate::BeansError;
use crate::helper::generate_rand_str;

/// all possible known directory where steam *might* be
/// only is used on linux, since windows will use the registry.
pub const STEAM_POSSIBLE_DIR:  &'static [&'static str] = &[
    "~/.steam/registry.vdf",
    "~/.var/app/com.valvesoftware.Steam/.steam/registry.vdf"
];

/// find sourcemod path on linux.
/// fetches the fake registry that steam uses from find_steam_reg_path
/// and gets the value of Registry/HKCU/Software/Valve/Steam/SourceModInstallPath
pub fn find_sourcemod_path() -> Result<String, BeansError>
{
    let reg_path = find_steam_reg_path()?;

    let reg_content = match read_to_string(reg_path.as_str())
    {
        Ok(v) => v,
        Err(e) => {
            sentry::capture_error(&e);
            return Err(BeansError::FileOpenFailure(reg_path, e));
        }
    };

    for line in reg_content.lines() {
        if line.contains("SourceModInstallPath")
        {
            let split = &line.split("\"SourceModInstallPath\"");
            let mut last = split.clone()
                .last()
                .expect("Failed to find SourceModInstallPath")
                .trim()
                .replace("\\\\", "/")
                .replace("\\", "/")
                .replace("\"", "");
            if last.ends_with("/") == false {
                last.push_str("/");
            }
            return Ok(last);
        }
    }

    return Err(BeansError::SourceModLocationNotFound);
}
/// returns the first item in STEAM_POSSIBLE_DIR that exists. otherwise None
fn find_steam_reg_path() -> Result<String, BeansError>
{
    for x in STEAM_POSSIBLE_DIR.into_iter() {
        match simple_home_dir::home_dir() {
            Some(v) => {
                match v.to_str() {
                    Some(k) => {
                        let mut h = k.to_string();
                        if h.ends_with("/") {
                            h.pop();
                        }
                        let reg_loc = x.replace("~", h.as_str());
                        if crate::helper::file_exists(reg_loc.clone())
                        {
                            return Ok(reg_loc);
                        }
                    },
                    None => {
                        return Err(BeansError::SteamNotFound);
                    }
                }
            },
            None => {
                return Err(BeansError::SteamNotFound);
            }
        }
    }
    return Err(BeansError::SteamNotFound);
}
pub fn get_tmp_file(filename: String) -> String
{
    let mut loc = std::env::temp_dir().to_str().unwrap_or("").to_string();
    if loc.ends_with("/") == false && loc.len() > 1{
        loc.push_str("/");
    }
    loc.push_str(generate_rand_str(8).as_str());
    loc.push_str(format!("_{}", filename).as_str());
    loc
}