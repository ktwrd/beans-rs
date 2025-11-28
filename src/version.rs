use std::{backtrace::Backtrace,
          collections::HashMap,
          fs::{File,
               read_to_string},
          io::{Read,
               Write}};

use log::{debug,
          error,
          trace};
use serde_json::{self,
                 Value};
use valve_pak::{VPK,
                VPKFile};

use crate::{BeansError,
            appvar::AppVarData,
            helper::{self,
                     InstallType,
                     find_sourcemod_path}};

/// get the current version installed via the .adastral file in the sourcemod
/// mod folder. will parse the value of `version` as usize.
pub fn get_current_version(sourcemods_location: Option<String>) -> Option<usize>
{
    let install_state = helper::install_state(sourcemods_location.clone());

    if install_state == InstallType::NotInstalled
    {
        return None;
    }

    if install_state != InstallType::Adastral
    {
        // generate an .adastral file at this location
        let version_file: AdastralVersionFile = match generate_version_file(sourcemods_location)
        {
            Ok(f) => f,
            Err(_) => panic!("Failed to generate .adastral file!")
        };

        return Some(
            version_file
                .version
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("Failed to get generated version file's usize"))
        );
    }
    match get_mod_location(sourcemods_location.clone())
    {
        Some(smp_x) =>
        {
            // TODO generate BeansError instead of using panic
            let location = format!("{}.adastral", smp_x);
            let content =
                read_to_string(&location).unwrap_or_else(|_| panic!("Failed to open {}", location));
            let data: AdastralVersionFile = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to deserialize data at {}", location));
            let parsed = data.version.parse::<usize>().unwrap_or_else(|_| {
                panic!("Failed to convert version to usize! ({})", data.version)
            });

            Some(parsed)
        }
        None => None
    }
}

const FILE_MAP_JSON_DATA: &str = "filemap.json";
/// Read a version file from a json file in the mod path.
/// Returns the version file's contents WITHOUT trailing whitespaces
fn read_mod_version_file(sourcemods_location: &str) -> Result<String, BeansError>
{
    let mod_path = match get_mod_location(Some(sourcemods_location.to_owned()))
    {
        Some(x) => x,
        None =>
        {
            panic!("version::read_mod_version_file: Failed to get the sourcemods_location!");
        }
    };

    // get the filename and pak directory from the json file
    let data_json_content = open_json_file_content(sourcemods_location)?;
    let mod_pak_filename = data_json_content["data"]["pack_file"]
        .to_string()
        .replace("\"", "");
    let mod_version_filename = data_json_content["data"]["version_file"]
        .to_string()
        .replace("\"", "");
    let mod_version_full_path = mod_path.clone() + &mod_version_filename;
    let mod_pak_full_path = mod_path.clone() + &mod_pak_filename;

    // Check regular sourcemod directory
    if helper::path_exists(mod_version_full_path.clone())
    {
        let mut mod_version_file = File::open(mod_version_full_path.clone())?;
        let version_content = &mut String::new();
        let __ = mod_version_file.read_to_string(version_content);
        return Ok(version_content.trim().to_owned().clone());
    }
    // else check inside vpk
    else if helper::path_exists(mod_pak_full_path.clone())
    {
        let mod_pack_file: VPK = match VPK::open(mod_pak_full_path.clone())
        {
            Ok(f) => f,
            Err(_) => panic!("version::read_mod_version_file: VPK not found")
        };
        let mut mod_pack_file_in_vpk: VPKFile = match mod_pack_file
            .get_file(mod_version_filename.as_str())
        {
            Ok(f) => f,
            Err(_) => panic!(
                "version::read_mod_version_file: {} not found in {}",
                mod_version_filename, mod_pak_filename
            )
        };
        let pak_version_content = &mut String::new();
        let __ = mod_pack_file_in_vpk.read_to_string(pak_version_content);
        let ___ = pak_version_content.trim();
        return Ok(pak_version_content.trim().to_owned().clone());
    }

    // error out if we can't find the vpk file (last file we checked for)
    Err(BeansError::FileNotFound {
        location: mod_path.clone() + &mod_pak_filename,
        backtrace: Backtrace::capture()
    })
}

/// Opens the json file DATA_JSON_NAME from the mod folder. returns a
/// serde_json::Value
fn open_json_file_content(sourcemods_location: &str) -> Result<Value, BeansError>
{
    // get the mod path
    let mod_path = match get_mod_location(Some(sourcemods_location.to_owned()))
    {
        Some(p) => p,
        None => panic!("version::open_json_file_content: Failed to get mod location!")
    };

    // try to open the json data file
    let mut json_file = match File::open(mod_path.clone() + FILE_MAP_JSON_DATA)
    {
        Ok(f) =>
        {
            log::info!(
                "version::open_json_file_content: found {}",
                mod_path + FILE_MAP_JSON_DATA
            );
            f
        }
        Err(_) =>
        {
            panic!(
                "version::open_json_file_content: Failed to get {}!",
                FILE_MAP_JSON_DATA
            );
        }
    };

    // parse it
    let json_content = &mut String::new();
    let _ = json_file.read_to_string(json_content);
    let __ = json_content.trim();
    let vpk_json_content: Value = serde_json::from_str(json_content)?;
    Ok(vpk_json_content)
}

/// generate an .adastral file if the game was installed through other methods
/// based on the build's version file
fn generate_version_file(
    sourcemods_location: Option<String>
) -> Result<AdastralVersionFile, BeansError>
{
    let sm_path = match sourcemods_location
    {
        Some(x) => x,
        None => panic!("version::read_mod_version_file: Failed to get the sourcemods_location!")
    };

    let mod_version_content = read_mod_version_file(&sm_path.as_str())?;
    let data_json_content = open_json_file_content(&sm_path.as_str())?;
    // create a(n?) .adastral file with the version translation defined in the json
    let mod_version_translation = AdastralVersionFile {
        version: data_json_content["versions"][mod_version_content]["version"]
            .to_string()
            .replace("\"", "")
    };

    mod_version_translation.write(Some(sm_path.to_owned()))?;

    let mod_path = match get_mod_location(Some(sm_path))
    {
        Some(x) => x,
        None => panic!("version::generate_version_file: Failed to get the mod path!")
    };

    log::info!("Generated .adastral file at location {}", mod_path);

    Ok(mod_version_translation)
}

fn get_version_location(sourcemods_location: Option<String>) -> Option<String>
{
    get_mod_location(sourcemods_location).map(|v| format!("{}.adastral", v))
}

/// get the full location of the sourcemod mod directory.
fn get_mod_location(sourcemods_location: Option<String>) -> Option<String>
{
    let smp_x = match sourcemods_location
    {
        Some(v) => v,
        None => match find_sourcemod_path()
        {
            Ok(v) => v,
            Err(e) =>
            {
                sentry::capture_error(&e);
                debug!(
                    "[version::get_mod_location] {} {:#?}",
                    BeansError::SourceModLocationNotFound,
                    e
                );
                return None;
            }
        }
    };
    Some(helper::join_path(smp_x, crate::data_dir()))
}

/// migrate from old file (.revision) to new file (.adastral) in sourcemod mod
/// directory.
pub fn update_version_file(sourcemods_location: Option<String>) -> Result<(), BeansError>
{
    let install_state = helper::install_state(sourcemods_location.clone());

    match install_state
    {
        InstallType::NotInstalled =>
        {
            debug!(
                "[version::update_version_file] install_state is {:#?}, ignoring.",
                install_state
            );
        }
        InstallType::Adastral =>
        {
            debug!(
                "[version::update_version_file] install_state is {:#?}, ignoring.",
                install_state
            );
        }

        InstallType::OtherSourceManual =>
        {
            debug!(
                "[version::update_version_file] install_state is {:#?}, ignoring.",
                install_state
            );
        }
        InstallType::OtherSource =>
        {
            let smp_x = match sourcemods_location
            {
                Some(v) => v,
                None => match find_sourcemod_path()
                {
                    Ok(v) => v,
                    Err(e) =>
                    {
                        error!(
                            "[version::update_version_file] Could not find sourcemods folder! {:}",
                            e
                        );
                        debug!("{:#?}", e);
                        sentry::capture_error(&e);
                        return Err(e);
                    }
                }
            };

            let data_dir = helper::join_path(smp_x, crate::data_dir());

            let old_version_file_location = format!("{}.revision", &data_dir);
            let old_version_file_content = match read_to_string(&old_version_file_location)
            {
                Ok(v) => v,
                Err(e) =>
                {
                    debug!(
                        "[update_version_file] failed to read {}. {:#?}",
                        old_version_file_location, e
                    );
                    sentry::capture_error(&e);
                    return Err(BeansError::VersionFileReadFailure {
                        error: e,
                        location: old_version_file_location
                    });
                }
            };
            let old_version_idx = match old_version_file_content.parse::<usize>()
            {
                Ok(v) => v,
                Err(e) =>
                {
                    debug!(
                        "[update_version_file] Failed to parse content {} caused error {:}",
                        old_version_file_content, e
                    );
                    sentry::capture_error(&e);
                    return Err(BeansError::VersionFileParseFailure {
                        error: e,
                        old_location: old_version_file_location,
                        old_content: old_version_file_content
                    });
                }
            };

            let new_file_content = AdastralVersionFile {
                version: old_version_idx.to_string()
            };

            let new_version_file_location = format!("{}.adastral", &data_dir);
            let new_version_file_content = match serde_json::to_string(&new_file_content)
            {
                Ok(v) => v,
                Err(e) =>
                {
                    sentry::capture_error(&e);
                    return Err(BeansError::VersionFileSerialize {
                        error: e,
                        instance: new_file_content
                    });
                }
            };

            if let Err(e) =
                std::fs::write(new_version_file_location.clone(), new_version_file_content)
            {
                sentry::capture_error(&e);
                return Err(BeansError::VersionFileMigrationFailure {
                    error: e,
                    location: new_version_file_location
                });
            }
            if let Err(e) = std::fs::remove_file(old_version_file_location.clone())
            {
                sentry::capture_error(&e);
                return Err(BeansError::VersionFileMigrationDeleteFailure {
                    error: e,
                    location: old_version_file_location
                });
            }
        }
    }
    Ok(())
}

/// fetch the version list from `{crate::SOURCE_URL}versions.json`
pub async fn get_version_list() -> Result<RemoteVersionResponse, BeansError>
{
    let av = AppVarData::get();
    let response = match reqwest::get(&av.remote_info.versions_url).await
    {
        Ok(v) => v,
        Err(e) =>
        {
            error!(
                "[version::get_version_list] Failed to get available versions! {:}",
                e
            );
            sentry::capture_error(&e);
            return Err(BeansError::Reqwest {
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let response_text = response.text().await?;
    trace!(
        "[version::get_version_list] response text: {}",
        response_text
    );

    let data: RemoteVersionResponse = serde_json::from_str(&response_text)?;
    Ok(data)
}

/// Version file that is used as `.adastral` in the sourcemod mod folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdastralVersionFile
{
    pub version: String
}

impl AdastralVersionFile
{
    pub fn write(
        &self,
        sourcemods_location: Option<String>
    ) -> Result<(), BeansError>
    {
        match get_version_location(sourcemods_location)
        {
            Some(vl) =>
            {
                let f = match helper::file_exists(vl.clone())
                {
                    true => std::fs::File::create(vl.clone()),
                    false => std::fs::File::create_new(vl.clone())
                };
                match f
                {
                    Ok(mut file) => match serde_json::to_string(self)
                    {
                        Ok(ser) => match file.write_all(ser.as_bytes())
                        {
                            Ok(_) => Ok(()),
                            Err(e) => Err(BeansError::FileWriteFailure {
                                location: vl,
                                error: e
                            })
                        },
                        Err(e) => Err(e.into())
                    },
                    Err(e) => Err(BeansError::FileOpenFailure {
                        location: vl,
                        error: e
                    })
                }
            }
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
    /// Amount of file space required for temporary file. Assumed to be measured
    /// in bytes.
    pub tempreq: usize
}
