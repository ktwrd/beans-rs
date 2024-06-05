use std::backtrace::Backtrace;
use crate::{BeansError, depends, helper, version};
use crate::helper::{find_sourcemod_path, InstallType, parse_location};
use crate::version::{RemotePatch, RemoteVersion, RemoteVersionResponse};
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use log::{debug, error, info};

#[derive(Debug, Clone)]
pub struct RunnerContext
{
    pub sourcemod_path: String,
    pub remote_version_list: RemoteVersionResponse,
    pub current_version: Option<usize>,
    pub appvar: crate::appvar::AppVarData
}
impl RunnerContext
{
    pub async fn create_auto(sml_via: SourceModDirectoryParam) -> Result<Self, BeansError>
    {
        depends::try_write_deps();
        if let Err(e) = depends::try_install_vcredist().await {
            sentry::capture_error(&e);
            println!("Failed to install vcredist! {:}", e);
            debug!("[RunnerContext::create_auto] Failed to install vcredist! {:#?}", e);
        }
        let sourcemod_path = parse_location(match sml_via
        {
            SourceModDirectoryParam::AutoDetect => match find_sourcemod_path() {
                Ok(v) => v,
                Err(e) => {
                    sentry::capture_error(&e);
                    debug!("[RunnerContext::create_auto] Failed to find sourcemods folder. {:#?}", e);
                    return Err(BeansError::SourceModLocationNotFound);
                }
            },
            SourceModDirectoryParam::WithLocation(l) => {
                debug!("[RunnerContext::create_auto] Using specified location {}", l);
                l
            }
        });
        let version_list = version::get_version_list().await?;

        if helper::install_state(Some(sourcemod_path.clone())) == InstallType::OtherSource {
            version::update_version_file(Some(sourcemod_path.clone()))?;
        }

        return Ok(Self
        {
            sourcemod_path: parse_location(sourcemod_path.clone()),
            remote_version_list: version_list,
            current_version: crate::version::get_current_version(Some(sourcemod_path.clone())),
            appvar: crate::appvar::parse()
        });
    }
    /// Sets `remote_version_list` from `version::get_version_list()`
    pub async fn set_remote_version_list(&mut self) -> Result<(), BeansError>
    {
        self.remote_version_list = version::get_version_list().await?;
        Ok(())
    }

    /// Get the location of the sourcemod mod
    /// {sourcemod_dir}{crate::DATA_DIR}
    /// e.g; /home/kate/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/sourcemods/open_fortress/
    ///      C:\Games\Steam\steamapps\sourcemods\open_fortress\
    pub fn get_mod_location(&mut self) -> String
    {
        helper::join_path(self.sourcemod_path.clone(), crate::data_dir())
    }

    /// Get staging location for butler.
    /// {sourcemod_dir}{crate::STAGING_DIR}
    /// e.g; /home/kate/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/sourcemods/butler-staging
    ///      C:\Games\Steam\steamapps\sourcemods\butler-staging
    pub fn get_staging_location(&mut self) -> String {
        helper::join_path(self.sourcemod_path.clone(), crate::STAGING_DIR.to_string())
    }

    /// Get the latest item in `remote_version_list`
    pub fn latest_remote_version(&mut self) -> (usize, RemoteVersion)
    {
        let mut highest = usize::MIN;
        for (key, _) in self.remote_version_list.clone().versions.into_iter() {
            if key > highest {
                highest = key;
            }
        }
        let x = self.remote_version_list.versions.get(&highest).unwrap();
        (highest, x.clone())
    }

    /// Get the RemoteVersion that matches `self.current_version`
    pub fn current_remote_version(&mut self) -> Result<RemoteVersion, BeansError> {
        match self.current_version {
            Some(cv) => {
                for (v, i) in self.remote_version_list.clone().versions.into_iter() {
                    if v == cv {
                        return Ok(i.clone());
                    }
                }
                return Err(BeansError::RemoteVersionNotFound {
                    version: self.current_version
                });
            },
            None => {
                Err(BeansError::RemoteVersionNotFound {
                    version: self.current_version
                })
            }
        }
    }

    /// When self.current_version is some, iterate through patches and fetch the patch that is available
    /// to bring the current version in-line with the latest version.
    pub fn has_patch_available(&mut self) -> Option<RemotePatch>
    {
        let current_version = self.current_version.clone();
        let (remote_version, _) = self.latest_remote_version();
        match current_version {
            Some(cv) => {
                for (_, patch) in self.remote_version_list.clone().patches.into_iter() {
                    if patch.file == format!("{}-{}to{}.pwr", &self.appvar.mod_info.short_name, cv, remote_version) {
                        return Some(patch);
                    }
                }
                return None;
            },
            _ => None
        }
    }

    /// Read the contents of `gameinfo.txt` in directory from `self.get_mod_location()`
    pub fn read_gameinfo_file(&mut self) -> Result<Option<Vec<u8>>, BeansError> {
        let location = self.gameinfo_location();
        if helper::file_exists(location.clone()) == false {
            return Ok(None);
        }
        let file = std::fs::read(&location)?;
        Ok(Some(file))
    }

    /// Get the location of `gameinfo.txt` inside of the folder returned by `self.get_mod_location()`
    pub fn gameinfo_location(&mut self) -> String {
        let mut location =  self.get_mod_location();
        location.push_str("gameinfo.txt");
        location
    }

    /// Make sure that the permissions for gameinfo.txt on linux are 0644
    #[cfg(target_os = "linux")]
    pub fn gameinfo_perms(&mut self) -> Result<(), BeansError> {
        let location = self.gameinfo_location();
        if helper::file_exists(location.clone()) {
            let perm = std::fs::Permissions::from_mode(0o644 as u32);
            if let Err(e) = std::fs::set_permissions(&location, perm.clone()) {
                let xe = BeansError::GameInfoPermissionSetFail {
                    error: e,
                    permissions: perm.clone(),
                    location
                };
                sentry::capture_error(&xe);
                return Err(xe);
            }
            debug!("[RunnerContext::gameinfo_perms] set permissions on {location} to {:#?}", perm);
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    pub fn gameinfo_perms(&mut self) -> Result<(), BeansError> {
        Ok(())
    }

    /// Download package with Progress Bar.
    /// Ok is the location to where it was downloaded to.
    pub async fn download_package(version: RemoteVersion) -> Result<String, BeansError>
    {
        let av = crate::appvar::parse();
        let mut out_loc = std::env::temp_dir().to_str().unwrap_or("").to_string();

        if let Some(size) = version.pre_sz {
            if helper::has_free_space(out_loc.clone(), size)? == false {
                panic!("Not enough free space to install latest version!");
            }
        }

        let out_filename = format!("presz_{}", helper::generate_rand_str(12));
        out_loc = helper::join_path(out_loc, out_filename);

        info!("[RunnerContext::download_package] writing to {}", out_loc);
        helper::download_with_progress(
            format!("{}{}", &av.remote_info.base_url, version.file.expect("No URL for latest package!")),
            out_loc.clone()).await?;

        Ok(out_loc)
    }

    /// Extract zstd_location to the detected sourcemods directory.
    /// TODO replace unwrap/expect with match error handling
    pub fn extract_package(zstd_location: String, out_dir: String) -> Result<(), BeansError>
    {
        let tar_tmp_location = helper::get_tmp_file("data.tar".to_string());

        let zstd_file = std::fs::File::open(&zstd_location)?;
        let mut tar_tmp_file = std::fs::File::create_new(&tar_tmp_location)?;
        zstd::stream::copy_decode(zstd_file, &tar_tmp_file)?;
        tar_tmp_file = std::fs::File::open(&tar_tmp_location)?; // we do this again to make sure that the tar is properly opened.

        let mut archive = tar::Archive::new(&tar_tmp_file);
        let x = archive.unpack(&out_dir);
        if helper::file_exists(tar_tmp_location.clone()) {
            if let Err(e) = std::fs::remove_file(tar_tmp_location.clone()) {
                sentry::capture_error(&e);
                error!("[RunnerContext::extract_package] Failed to delete temporary file: {:}", e);
                debug!("[RunnerContext::extract_package] Failed to delete {}\n{:#?}", tar_tmp_location, e);
            }
        }
        match x {
            Err(e) => Err(BeansError::TarExtractFailure{
                src_file: tar_tmp_location,
                target_dir: out_dir,
                error: e,
                backtrace: Backtrace::capture()
            }),
            Ok(_) => Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    pub fn prepare_symlink(&mut self) -> Result<(), BeansError>
    {
        for pair in SYMLINK_FILES.into_iter() {
            let target: &str = pair[1];
            let mod_location = self.get_mod_location();
            let ln_location = format!("{}{}", mod_location, target);
            if helper::file_exists(ln_location.clone())
            && helper::is_symlink(ln_location.clone()) == false {
                std::fs::remove_file(&ln_location)?;
            }
        }

        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    pub fn prepare_symlink(&mut self) -> Result<(), BeansError>
    {
        // ignored since this symlink stuff is for linux only
        Ok(())
    }
}

pub const SYMLINK_FILES: &'static [&'static [&'static str; 2]] = &[
    &["bin/server.so", "bin/server_srv.so"]
];


#[derive(Clone, Debug)]
pub enum SourceModDirectoryParam
{
    /// Default value. Will autodetect location.
    AutoDetect,
    /// Use from the specified sourcemod location.
    WithLocation(String)
}
impl Default for SourceModDirectoryParam
{
    fn default() -> Self {
        SourceModDirectoryParam::AutoDetect
    }
}