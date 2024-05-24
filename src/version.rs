use std::collections::HashMap;
use std::fs::read_to_string;
use crate::helper;
use crate::helper::{find_sourcemod_path, InstallType};

/// get the current version installed via the .adastral file in the sourcemod mod folder.
/// will parse the value of `version` as usize.
pub fn get_current_version() -> Option<usize>
{
    let install_state = helper::install_state();
    if install_state != InstallType::Adastral {
        return None;
    }
    match get_mod_location() {
        Some(smp_x) => {
            let location = format!("{}{}.adastral", smp_x, crate::DATA_DIR);
            let content = read_to_string(&location).expect(format!("Failed to open {}", location).as_str());
            let data: AdastralVersionFile = serde_json::from_str(&content).expect(format!("Failed to deserialize data at {}", location).as_str());
            let parsed = data.version.parse::<usize>().expect(format!("Failed to convert version to usize! ({})", data.version).as_str());
            Some(parsed)
        },
        None => None
    }
}
/// get the full location of the sourcemod mod directory.
fn get_mod_location() -> Option<String>
{
    let smp = find_sourcemod_path();
    if smp.is_none() {
        // sourcemod path couldn't be found, doesn't matter :3
        return None;
    }
    let mut smp_x = smp.unwrap();
    if smp_x.ends_with("/") || smp_x.ends_with("\\") {
        smp_x.pop();
    }
    Some(smp_x)
}
/// migrate from old file (.revision) to new file (.adastral) in sourcemod mod directory.
pub fn update_version_file()
{
    let install_state = helper::install_state();
    if install_state == InstallType::Adastral {
        return;
    }
    // ignore :)
    else if install_state == InstallType::OtherSourceManual {
        return;
    }
    // ignore :)
    else if install_state == InstallType::NotInstalled {
        return;
    }

    let smp = find_sourcemod_path();
    if smp.is_none() {
        // sourcemod path couldn't be found, doesn't matter :3
        return;
    }
    let mut smp_x = smp.unwrap();
    if smp_x.ends_with("/") || smp_x.ends_with("\\") {
        smp_x.pop();
    }

    let old_version_file_location = format!("{}{}.revision", smp_x, crate::DATA_DIR);
    let old_version_file_content = read_to_string(&old_version_file_location).expect(format!("Failed to open {}", old_version_file_location).as_str());
    let old_version_idx = match old_version_file_content.parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to parse old version number from {}\nIt was; {}\n\n{:#?}", old_version_file_location, old_version_file_content, e);
        }
    };

    let new_file_content = AdastralVersionFile
    {
        version: old_version_idx.to_string()
    };

    let new_version_file_location = format!("{}{}.adastral", smp_x, crate::DATA_DIR);
    let new_version_file_content = match serde_json::to_string(&new_file_content) {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to serialize! {:#?}", e);
        }
    };
    std::fs::write(new_version_file_location, new_version_file_content).expect("Failed to migrate old file to new file!");
    std::fs::remove_file(old_version_file_location).expect("Failed to delete old version file");
}

/// fetch the version list from `{crate::SOURCE_URL}versions.json`
pub async fn get_version_list() -> RemoteVersionResponse
{
    let response = match reqwest::get(format!("{}versions.json", crate::SOURCE_URL)).await {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to get versions from server!\n{:#?}", e);
        }
    };
    let response_text = response.text().await.expect("Failed to get version details content");
    let data = serde_json::from_str(&response_text).expect("Failed to deserialize version details from server");

    return data;
}

/// Version file that is used as `.adastral` in the sourcemod mod folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdastralVersionFile {
    pub version: String
}
/// Value of the `versions` property in `RemoteVersionResponse`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteVersion
{
    pub url: Option<String>,
    pub file: Option<String>,
    #[serde(rename = "presz")]
    pub pre_sz: Option<usize>,
    #[serde(rename = "postsz")]
    pub post_sz: Option<usize>,
    #[serde(rename = "signature")]
    pub signature_url: Option<String>,
    // should be set when signature_url is set
    pub signature_content: Option<String>,
    pub heal: Option<String>
}
/// `versions.json` response content from remote server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteVersionResponse
{
    pub versions: HashMap<usize, RemoteVersion>
}