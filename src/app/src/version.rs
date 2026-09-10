use std::{backtrace::Backtrace,
          collections::HashMap,
          fs::{File,
               read_to_string},
          io::{BufWriter,
               Read,
               Write}};

use beans_core::{BeansError,
                 appvar::AppVarData,
                 data_dir,
                 get_user_agent,
                 path::{file_exists,
                        join_path,
                        path_exists},
                 version::AdastralVersionFile};
use log::{debug,
          error,
          trace};
use serde_json::{self};
use valve_pak::{VPK,
                VPKFile};

use crate::helper::{self,
                    InstallType,
                    find_sourcemod_path};

/// get the current version installed via the .adastral file in the sourcemod
/// mod folder. will parse the value of `version` as usize.
pub async fn get_current_version(sourcemods_location: Option<String>) -> Option<usize>
{
    // TODO change function to return a BeansError
    let install_state = helper::install_state(sourcemods_location.clone());

    if install_state == InstallType::NotInstalled
    {
        return None;
    }

    if install_state != InstallType::Adastral
    {
        // generate an .adastral file at this location
        let data: AdastralVersionFile = match generate_version_file(sourcemods_location.clone()?)
            .await
        {
            Ok(v) => v,
            Err(e) =>
            {
                trace!("{:#?}", e);
                sentry::capture_error(&e);
                panic!(
                    "[WizardContext::run] {} {:#?}",
                    t!("error.run", task = "version::generate_version_file"),
                    e
                );
            }
        };
        let parsed = match data.version.parse::<usize>()
        {
            Ok(v) => v,
            Err(e) =>
            {
                let ex = BeansError::VersionFileParseFailure {
                    error: e,
                    old_location: sourcemods_location.clone()?,
                    old_content: data.version
                };
                debug!("{:#?}", ex);
                sentry::capture_error(&ex);
                panic!(
                    "[version::get_current_version] {} {:#?}",
                    t!("error.version.usize"),
                    ex
                );
            }
        };
        return Some(parsed);
    }
    match get_mod_location(sourcemods_location.clone())
    {
        Some(smp_x) =>
        {
            let location = format!("{}.adastral", smp_x);
            let content = match read_to_string(&location)
            {
                Ok(v) => v,
                Err(e) =>
                {
                    let ex = BeansError::VersionFileReadFailure {
                        error: e,
                        location: location.clone()
                    };
                    debug!("{:#?}", ex);
                    sentry::capture_error(&ex);
                    panic!("{} {:#?}", t!("error.open.item", item = location), ex);
                }
            };
            let data: AdastralVersionFile = match serde_json::from_str(&content)
            {
                Ok(v) => v,
                Err(e) =>
                {
                    let ex = BeansError::SerdeJson {
                        content: Some(content),
                        error: e,
                        backtrace: Backtrace::capture()
                    };
                    debug!("{:#?}", ex);
                    sentry::capture_error(&ex);
                    panic!(
                        "[version::get_current_version] {} {:#?}",
                        t!("error.deserialize.data", location = location),
                        ex
                    )
                }
            };
            let parsed = match data.version.parse::<usize>()
            {
                Ok(v) => v,
                Err(e) =>
                {
                    let ex = BeansError::VersionFileParseFailure {
                        error: e,
                        old_location: location.clone(),
                        old_content: data.version.clone()
                    };
                    debug!("{:#?}", ex);
                    sentry::capture_error(&ex);
                    panic!(
                        "[version::get_current_version] {} {:#?}",
                        t!("error.version.usize.item", item = data.version),
                        ex
                    )
                }
            };

            Some(parsed)
        }
        None => None
    }
}

/// Read a version file from either version file in mod folder or in specified
/// pack file.
async fn read_mod_version_file(
    sourcemods_location: &str,
    files: &RemoteFiles
) -> Result<String, BeansError>
{
    let mod_path = match get_mod_location(Some(sourcemods_location.to_owned()))
    {
        Some(x) => x,
        None => return Err(BeansError::SourceModLocationNotFound)
    };

    // get the filename and pak directory from the remote json file
    let mod_version_full_path = mod_path.clone() + &files.version_file;
    let mod_pak_full_path = mod_path.clone() + &files.pack_file;

    // Check regular sourcemod directory
    if path_exists(mod_version_full_path.clone())
    {
        let mut mod_version_file = File::open(mod_version_full_path.clone())?;
        let version_content = &mut String::new();
        match mod_version_file.read_to_string(version_content)
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!(
                    "[version::read_mod_verion_file] {} {:}",
                    t!("error.read.file", file = mod_version_full_path.clone()),
                    e
                );
                debug!("{:#?}", e);
                return Err(BeansError::FileOpenFailure {
                    location: files.version_file.clone(),
                    error: e
                });
            }
        };

        return Ok(version_content.trim().to_owned().clone());
    }
    // else check inside vpk
    else if path_exists(mod_pak_full_path.clone())
    {
        let mod_pack_file: VPK = match VPK::open(mod_pak_full_path.clone())
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!(
                    "[version::read_mod_version_file] {} {:}",
                    t!("error.vpk.missing"),
                    e
                );
                debug!("{:#?}", e);
                return Err(BeansError::VpkOpenFailure {
                    location: files.version_file.clone(),
                    error: e,
                    backtrace: Backtrace::capture()
                });
            }
        };
        let mut mod_pack_file_in_vpk: VPKFile = match mod_pack_file
            .get_file(files.version_file.as_str())
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!(
                    "[version::read_mod_version_file] {} {:}",
                    t!(
                        "error.vpk.item",
                        item = files.version_file,
                        pack = files.pack_file
                    ),
                    e
                );
                debug!("{:#?}", e);
                return Err(BeansError::VpkReadFailure {
                    location: files.version_file.clone(),
                    error: e,
                    backtrace: Backtrace::capture()
                });
            }
        };
        let pak_version_content = &mut String::new();
        match mod_pack_file_in_vpk.read_to_string(pak_version_content)
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!(
                    "[version::read_mod_version_file] {} {:}",
                    t!("error.open.item", item = files.version_file),
                    e
                );
                debug!("{:#?}", e);
                return Err(BeansError::VpkInternalFileReadFailure {
                    location: files.version_file.clone(),
                    error: e,
                    backtrace: Backtrace::capture()
                });
            }
        };
        return Ok(pak_version_content.trim().to_owned().clone());
    }

    // error out if we can't find the vpk file (last file we checked for)
    Err(BeansError::FileNotFound {
        location: mod_path.clone() + &files.pack_file,
        backtrace: Backtrace::capture()
    })
}

/// generate an .adastral file if the game was installed through other methods
/// based on the build's version file
async fn generate_version_file(
    sourcemods_location: String
) -> Result<AdastralVersionFile, BeansError>
{
    let file_map_list = match get_file_map().await
    {
        Ok(v) => v,
        Err(e) =>
        {
            error!(
                "[WizardContext::run] {} {:#?}",
                t!("error.run", task = "version::get_file_map()"),
                e
            );
            trace!("{:#?}", e);
            sentry::capture_error(&e);
            return Err(e);
        }
    };

    let mod_version_file_content =
        match read_mod_version_file(sourcemods_location.as_str(), &file_map_list.files).await
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!(
                    "[version::read_mod_version_file] {} {:#?}",
                    t!("error.read.version"),
                    e
                );
                trace!("{:#?}", e);
                sentry::capture_error(&e);
                return Err(e);
            }
        };
    // create a(n?) .adastral file with the version translation defined in the json
    // I think it's an? it is "ah"-dastral.. right? -Dani
    let mut adastral_value = String::new();

    for (mod_version, adastral_version) in file_map_list.versions.iter()
    {
        if mod_version == &mod_version_file_content
        {
            adastral_value = String::from(adastral_version);
            break;
        }
    }

    if adastral_value.is_empty()
    {
        let ex = BeansError::RemoteFileMapLocalVersionNotFound {
            expected: mod_version_file_content.clone()
        };
        debug!("{:#?}", ex);
        error!(
            "[version::generate_version_file] {} {:#?}",
            t!("error.filemap.local.version"),
            ex
        );
        sentry::capture_error(&ex);
        return Err(ex);
    }

    let mod_version_translation = AdastralVersionFile {
        // If this blows something up, my bad -Dani
        // Things no longer blow up :) -Dani
        version: adastral_value.clone()
    };
    match adastral_version_write(&mod_version_translation, Some(sourcemods_location.clone()))
    {
        Ok(v) => v,
        Err(e) =>
        {
            debug!("{:#?}", e);
            error!(
                "[version::generate_version_file] {} {:#?}",
                t!("error.adastral.set", version = adastral_value.clone()),
                e
            );
            sentry::capture_error(&e);
            return Err(e);
        }
    }

    let mod_path = match get_mod_location(Some(sourcemods_location.clone()))
    {
        Some(x) => x,
        None => return Err(BeansError::SourceModLocationNotFound)
    };

    log::info!("{}", t!("adastral.generated", location = mod_path));

    Ok(mod_version_translation)
}
/// set the version in the `.adastral` file in the sourcemod folder.
/// will silently fail when install_state is not InstallType::Adastral, or the
/// sourcemod isn't installed.
pub fn set_current_version(
    sourcemods_location: Option<String>,
    new_version: usize
) -> Result<(), BeansError>
{
    let install_state = helper::install_state(sourcemods_location.clone());
    if install_state != InstallType::Adastral
    {
        return Ok(());
    }
    match get_mod_location(sourcemods_location)
    {
        Some(smp_x) =>
        {
            // TODO generate BeansError instead of using panic
            let location = format!("{}.adastral", smp_x);
            let file = match file_exists(location.clone())
            {
                false => match File::create(&location)
                {
                    Ok(v) => v,
                    Err(e) =>
                    {
                        return Err(BeansError::VersionFileWriteFailure {
                            location: location.clone(),
                            error: e
                        });
                    }
                },
                true => match File::open(&location)
                {
                    Ok(v) => v,
                    Err(e) =>
                    {
                        return Err(BeansError::VersionFileOpenFailure {
                            location: location.clone(),
                            error: e
                        });
                    }
                }
            };
            let data = AdastralVersionFile {
                version: format!("{new_version}")
            };
            debug!(
                "[set_current_version] {}",
                t!("adastral.data", location = location, data = data.version)
            );
            let mut writer = BufWriter::new(file);
            match serde_json::to_writer(&mut writer, &data)
            {
                Ok(_) => Ok(()),
                Err(e) => Err(BeansError::VersionFileSerializeFailure {
                    location: location.clone(),
                    instance: data.clone(),
                    error: e
                })
            }
        }
        None => Ok(()) // silently fail
    }
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
    Some(join_path(smp_x, data_dir()))
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
                "[version::update_version_file] {}",
                t!("adstral.state", state = format!("{:#?}", install_state))
            );
        }
        InstallType::Adastral =>
        {
            debug!(
                "[version::update_version_file] {}",
                t!("adstral.state", state = format!("{:#?}", install_state))
            );
        }

        InstallType::OtherSourceManual =>
        {
            debug!(
                "[version::update_version_file] {}",
                t!("adstral.state", state = format!("{:#?}", install_state))
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
                            "[version::update_version_file] {} {:}",
                            t!("error.folder", folder = "sourcemod"),
                            e
                        );
                        debug!("{:#?}", e);
                        sentry::capture_error(&e);
                        return Err(e);
                    }
                }
            };

            let data_dir = join_path(smp_x, data_dir());

            let old_version_file_location = format!("{}.revision", &data_dir);
            let old_version_file_content = match read_to_string(&old_version_file_location)
            {
                Ok(v) => v,
                Err(e) =>
                {
                    debug!(
                        "[version::update_version_file] {} {:#?}",
                        t!("error.read.file", file = old_version_file_location),
                        e
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
                        "[version::update_version_file] {} {:}",
                        t!("error.parse.item", item = old_version_file_content),
                        e
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
    let user_agent = get_user_agent();
    let client = match reqwest::Client::builder().user_agent(&user_agent).build()
    {
        Ok(v) => v,
        Err(e) =>
        {
            let message = format!("{}", t!("error.reqwest.create", agent = user_agent));
            error!("[version::get_version_list] {:} {:}", message, e);
            let error = BeansError::Reqwest {
                error_message: message,
                error: e,
                backtrace: Backtrace::capture()
            };
            trace!("[version::get_version_list] {:#?}", error);
            sentry::capture_error(&error);
            return Err(error);
        }
    };

    let response = match client.get(&av.remote_info.versions_url).send().await
    {
        Ok(v) => v,
        Err(e) =>
        {
            let message = format!(
                "{:}",
                t!(
                    "error.filemap.remote.version",
                    url = av.remote_info.versions_url
                )
            );
            error!("[version::get_version_list] {:} {:}", message, e);
            let error = BeansError::Reqwest {
                error_message: message,
                error: e,
                backtrace: Backtrace::capture()
            };
            trace!("[version::get_version_list] {:#?}", error);
            sentry::capture_error(&error);
            return Err(error);
        }
    };
    let response_text = response.text().await?;
    trace!(
        "[version::get_version_list] {} {}\n{}",
        t!("info.response"),
        av.remote_info.versions_url,
        response_text
    );

    match serde_json::from_str(&response_text)
    {
        Ok(v) => Ok(v),
        Err(e) =>
        {
            let error = BeansError::SerdeJson {
                content: Some(response_text),
                error: e,
                backtrace: Backtrace::capture()
            };
            trace!(
                "[version::get_version_list] {}\n{:#?}",
                t!(
                    "error.deserialize.data",
                    location = av.remote_info.versions_url
                ),
                error
            );
            Err(error)
        }
    }
}

/// fetch the file map list from `{crate::SOURCE_URL}filemap.json`
pub async fn get_file_map() -> Result<RemoteFileMapResponse, BeansError>
{
    let av = AppVarData::get();
    let response = match reqwest::get(&av.remote_info.filemap_url).await
    {
        Ok(v) => v,
        Err(e) =>
        {
            let message = format!(
                "{:}",
                t!(
                    "error.filemap.remote.version",
                    url = av.remote_info.versions_url
                )
            );
            error!("[version::get_file_map] {} {:}", message, e);
            sentry::capture_error(&e);
            return Err(BeansError::Reqwest {
                error_message: message,
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let response_text = response.text().await?;
    trace!(
        "[version::get_file_map] {} {}",
        t!("info.response"),
        response_text
    );

    let data: RemoteFileMapResponse = serde_json::from_str(&response_text)?;
    Ok(data)
}

pub fn adastral_version_write(
    instance: &AdastralVersionFile,
    sourcemods_location: Option<String>
) -> Result<(), BeansError>
{
    let version_location = match get_version_location(sourcemods_location)
    {
        Some(v) => v,
        None =>
        {
            return Err(BeansError::SourceModLocationNotFound);
        }
    };

    let f = match file_exists(version_location.clone())
    {
        true => File::create(version_location.clone()),
        false => File::create_new(version_location.clone())
    };
    match f
    {
        Ok(mut file) => match serde_json::to_string(instance)
        {
            Ok(ser) => match file.write_all(ser.as_bytes())
            {
                Ok(_) => Ok(()),
                Err(e) => Err(BeansError::FileWriteFailure {
                    location: version_location,
                    error: e
                })
            },
            Err(e) => Err(e.into())
        },
        Err(e) => Err(BeansError::FileOpenFailure {
            location: version_location,
            error: e
        })
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

/// `filemap.json` response content from remote server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteFileMapResponse
{
    pub files: RemoteFiles,
    pub versions: HashMap<String, String>
}
/// Value of the `files` property in `RemoteFileMapResponse`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteFiles
{
    pub version_file: String,
    pub pack_file: String
}
