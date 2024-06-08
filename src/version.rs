use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::Write;
use log::{debug, error, trace};
use crate::appvar::AppVarData;
use crate::{DownloadFailureReason, helper};
use crate::helper::{find_sourcemod_path, InstallType};
use crate::BeansError;

/// get the current version installed via the .adastral file in the sourcemod mod folder.
/// will parse the value of `version` as usize.
pub fn get_current_version(sourcemods_location: Option<String>) -> Option<usize>
{
    let install_state = helper::install_state(sourcemods_location.clone());
    if install_state != InstallType::Adastral {
        return None;
    }
    match get_mod_location(sourcemods_location) {
        Some(smp_x) => {
            // TODO generate BeansError instead of using .expect
            let location = format!("{}.adastral", smp_x);
            let content = read_to_string(&location).expect(format!("Failed to open {}", location).as_str());
            let data: AdastralVersionFile = serde_json::from_str(&content).expect(format!("Failed to deserialize data at {}", location).as_str());
            let parsed = data.version.parse::<usize>().expect(format!("Failed to convert version to usize! ({})", data.version).as_str());
            Some(parsed)
        },
        None => None
    }
}
fn get_version_location(sourcemods_location: Option<String>) -> Option<String>
{
    match get_mod_location(sourcemods_location) {
        Some(v) => Some(format!("{}.adastral", v)),
        None => None
    }
}
/// get the full location of the sourcemod mod directory.
fn get_mod_location(sourcemods_location: Option<String>) -> Option<String>
{
    let smp_x = match sourcemods_location {
        Some(v) => v,
        None => match find_sourcemod_path() {
            Ok(v) => v,
            Err(e) => {
                sentry::capture_error(&e);
                debug!("[version::get_mod_location] {} {:#?}", BeansError::SourceModLocationNotFound, e);
                return None;
            }
        }
    };
    return Some(helper::join_path(smp_x, crate::data_dir()))
}
/// migrate from old file (.revision) to new file (.adastral) in sourcemod mod directory.
pub fn update_version_file(sourcemods_location: Option<String>) -> Result<(), BeansError>
{
    let install_state = helper::install_state(sourcemods_location.clone());
    if install_state == InstallType::Adastral {
        debug!("[version::update_version_file] install_state is {:#?}, ignoring.", install_state);
        return Ok(());
    }
    // ignore :)
    else if install_state == InstallType::OtherSourceManual {
        debug!("[version::update_version_file] install_state is {:#?}, ignoring.", install_state);
        return Ok(());
    }
    // ignore :)
    else if install_state == InstallType::NotInstalled {
        debug!("[version::update_version_file] install_state is {:#?}, ignoring.", install_state);
        return Ok(());
    }

    let smp_x = match sourcemods_location {
        Some(v) => v,
        None => match find_sourcemod_path() {
            Ok(v) => v,
            Err(e) => {
                error!("[version::update_version_file] Could not find sourcemods folder! {:}", e);
                debug!("{:#?}", e);
                sentry::capture_error(&e);
                return Err(e);
            }
        }
    };

    let data_dir = helper::join_path(smp_x, crate::data_dir());

    let old_version_file_location = format!("{}.revision", &data_dir);
    let old_version_file_content = match read_to_string(&old_version_file_location) {
        Ok(v) => v,
        Err(e) => {
            debug!("[update_version_file] failed to read {}. {:#?}", old_version_file_location, e);
            sentry::capture_error(&e);
            return Err(BeansError::VersionFileReadFailure {
                error: e,
                location: old_version_file_location
            });
        }
    };
    let old_version_idx = match old_version_file_content.parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            debug!("[update_version_file] Failed to parse content {} caused error {:}", old_version_file_content, e);
            sentry::capture_error(&e);
            return Err(BeansError::VersionFileParseFailure {
                error: e,
                old_location: old_version_file_location,
                old_content: old_version_file_content
            });
        }
    };

    let new_file_content = AdastralVersionFile
    {
        version: old_version_idx.to_string()
    };

    let new_version_file_location = format!("{}.adastral", &data_dir);
    let new_version_file_content = match serde_json::to_string(&new_file_content) {
        Ok(v) => v,
        Err(e) => {
            sentry::capture_error(&e);
            return Err(BeansError::VersionFileSerialize {
                error: e,
                instance: new_file_content
            });
        }
    };

    if let Err(e) = std::fs::write(new_version_file_location.clone(), new_version_file_content) {
        sentry::capture_error(&e);
        return Err(BeansError::VersionFileMigrationFailure {
            error: e,
            location: new_version_file_location
        });
    }
    if let Err(e) = std::fs::remove_file(old_version_file_location.clone()) {
        sentry::capture_error(&e);
        return Err(BeansError::VersionFileMigrationDeleteFailure {
            error: e,
            location: old_version_file_location
        })
    }

    Ok(())
}

/// fetch the version list from `{crate::SOURCE_URL}versions.json`
pub async fn get_version_list() -> Result<RemoteVersionResponse, BeansError>
{
    let av = crate::appvar::parse();
    let response = match reqwest::get(&av.remote_info.versions_url).await {
        Ok(v) => v,
        Err(e) => {
            error!("[version::get_version_list] Failed to get available versions! {:}", e);
            sentry::capture_error(&e);
            return Err(BeansError::Reqwest {
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let response_text = response.text().await?;
    let data: RemoteVersionResponse = serde_json::from_str(&response_text)?;

    return Ok(data);
}

/// Version file that is used as `.adastral` in the sourcemod mod folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdastralVersionFile {
    pub version: String
}
impl AdastralVersionFile {
    pub fn write(&self, sourcemods_location: Option<String>) -> Result<(), BeansError> {
        match get_version_location(sourcemods_location) {
            Some(vl) => {
                let f = match helper::file_exists(vl.clone()) {
                    true => std::fs::File::create(vl.clone()),
                    false => std::fs::File::create_new(vl.clone())
                };
                match f {
                    Ok(mut file) => {
                        match serde_json::to_string(self) {
                            Ok(ser) => {
                                match file.write_all(ser.as_bytes()) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(BeansError::FileWriteFailure {
                                        location: vl,
                                        error: e
                                    })
                                }
                            },
                            Err(e) => Err(e.into())
                        }
                    },
                    Err(e) => Err(BeansError::FileOpenFailure {
                        location: vl,
                        error: e
                    })
                }
            },
            None => Err(BeansError::SourceModLocationNotFound)
        }
    }
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
    #[serde(rename = "heal")]
    pub heal_url: Option<String>
}
impl RemoteVersion {
    pub async fn get_download_size(&self) -> Option<u64> {
        if let Some(f) = &self.file {
            let av = AppVarData::get();
            let url = format!("{}{}", av.remote_info.base_url, f);

            trace!("[download::with_progress_custom] fetching details from {url}");
            let res = match reqwest::Client::new()
                .get(&url)
                .send()
                .await {
                Ok(v) => v,
                Err(e) => {
                    trace!("[RemoteVersion::get_download_size] failed to get size from {url}\n {:#?}", e);
                    return None;
                }
            };

            return res.content_length();
        } else {
            return None;
        }
    }
}
/// `versions.json` response content from remote server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteVersionResponse
{
    pub versions: HashMap<usize, RemoteVersion>,
    pub patches: HashMap<usize, RemotePatch>
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemotePatch
{
    pub url: String,
    pub file: String,
    /// Amount of file space required for temporary file. Assumed to be measured in bytes.
    pub tempreq: usize
}