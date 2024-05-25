use std::backtrace::Backtrace;
use crate::{BeansError, depends, helper, version};
use crate::helper::{find_sourcemod_path, InstallType};
use crate::version::{RemotePatch, RemoteVersion, RemoteVersionResponse};
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone)]
pub struct RunnerContext
{
    pub sourcemod_path: String,
    pub remote_version_list: RemoteVersionResponse,
    pub current_version: Option<usize>
}
impl RunnerContext
{
    pub async fn create_auto() -> Result<Self, BeansError>
    {
        depends::try_write_deps();
        depends::try_install_vcredist();
        let sourcemod_path = match find_sourcemod_path() {
            Some(v) => v,
            None => {
                return Err(BeansError::SourceModLocationNotFound);
            }
        };
        let version_list = crate::version::get_version_list().await;

        if helper::install_state() == InstallType::OtherSource {
            version::update_version_file();
        }

        return Ok(Self
        {
            sourcemod_path,
            remote_version_list: version_list,
            current_version: crate::version::get_current_version()
        });
    }

    /// Get the location of the sourcemod mod
    /// {sourcemod_dir}{crate::DATA_DIR}
    /// e.g; /home/kate/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/sourcemods/open_fortress/
    ///      C:\Games\Steam\steamapps\sourcemods\open_fortress\
    pub fn get_mod_location(&mut self) -> String
    {
        let mut smp_x = self.sourcemod_path.clone();
        if smp_x.ends_with("/") || smp_x.ends_with("\\") {
            smp_x.pop();
        }
        smp_x.push_str(crate::DATA_DIR);
        smp_x
    }

    /// Get staging location for butler.
    /// {sourcemod_dir}{crate::STAGING_DIR}
    /// e.g; /home/kate/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/sourcemods/butler-staging
    ///      C:\Games\Steam\steamapps\sourcemods\butler-staging
    pub fn get_staging_location(&mut self) -> String {
        let mut smp_x = self.sourcemod_path.clone();
        if smp_x.ends_with("/") || smp_x.ends_with("\\") {
            smp_x.pop();
        }
        smp_x.push_str(crate::STAGING_DIR);
        smp_x
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
                    if patch.file == format!("of-{}to{}.pwr", cv, remote_version) {
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
            let perm = std::fs::Permissions::from_mode(0644 as u32);
            std::fs::set_permissions(&location, perm)?;
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
        if let Some(size) = version.pre_sz {
            if helper::sml_has_free_space(size)? == false {
                panic!("Not enough free space to install latest version!");
            }
        }

        let mut out_loc = std::env::temp_dir().to_str().unwrap_or("").to_string();
        if out_loc.ends_with("/") == false {
            out_loc.push_str("/");
        }
        out_loc.push_str(format!("presz_{}", helper::generate_rand_str(12)).as_str());

        println!("[debug] writing output file to {}", out_loc);
        helper::download_with_progress(
            format!("{}{}", crate::SOURCE_URL, version.file.expect("No URL for latest package!")),
            out_loc.clone()).await?;

        Ok(out_loc)
    }

    /// Extract zstd_location to the detected sourcemods directory.
    /// TODO replace unwrap/expect with match error handling
    pub fn extract_package(zstd_location: String, out_dir: String) -> Result<(), BeansError>
    {
        let zstd_content = std::fs::read(&zstd_location).unwrap();
        let zstd_decoded: Vec<u8> = zstd::decode_all(zstd_content.as_slice()).unwrap();
        let tar_tmp_location = helper::get_tmp_file("data.tar".to_string());
        if let Err(e) = std::fs::write(&tar_tmp_location, zstd_decoded) {
            return Err(BeansError::FileWriteFailure(tar_tmp_location.clone(), e));
        }

        let tar_tmp_file = match std::fs::File::open(tar_tmp_location.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Err(BeansError::FileOpenFailure(tar_tmp_location.clone(), e));
            }
        };
        let mut archive = tar::Archive::new(tar_tmp_file);
        match archive.unpack(&out_dir) {
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