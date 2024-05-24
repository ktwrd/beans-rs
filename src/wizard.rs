use crate::helper;
use crate::helper::{find_sourcemod_path, InstallType};
use crate::version::{AdastralVersionFile, RemotePatch, RemoteVersion, RemoteVersionResponse};
use async_recursion::async_recursion;

pub struct WizardContext
{
    pub sourcemod_path: String,
    pub remote_version_list: RemoteVersionResponse,
    pub current_version: Option<usize>
}
impl WizardContext
{
    /// run the wizard!
    pub async fn run()
    {
        helper::try_write_deps();
        helper::try_install_vcredist();
        let sourcemod_path = get_path();
        let version_list = crate::version::get_version_list().await;

        if helper::install_state() == InstallType::OtherSource {
            crate::version::update_version_file();
        }

        let mut i = Self
        {
            sourcemod_path,
            remote_version_list: version_list,
            current_version: crate::version::get_current_version()
        };
        i.menu().await;
    }

    /// Show the menu
    /// When an invalid option is selected, this will be re-called.
    #[async_recursion]
    pub async fn menu<'a>(&'a mut self)
    {
        println!();
        println!("1 - Install or reinstall the game");
        println!("2 - Check for and apply and available updates");
        println!("3 - Verify and repair game files");
        println!();
        println!("q - Quit");
        let user_input = helper::get_input("-- Enter option below --");
        match user_input.to_lowercase().as_str() {
            "1" => WizardContext::menu_error_catch(self.task_install().await),
            "2" => WizardContext::menu_error_catch(self.task_update().await),
            "3" => WizardContext::menu_error_catch(self.task_verify().await),
            "q" => std::process::exit(0),
            _ => {
                println!("Unknown option \"{}\"", user_input);
                self.menu().await;
                std::process::exit(0)
            }
        };
    }
    fn menu_error_catch(v: Result<(), BeansError>) -> ! {
        if let Err(e) = v {
            eprintln!("Failed to run action!");
            panic!("{:#?}", e);
        }
        std::process::exit(0)
    }

    fn latest_remote_version(&mut self) -> (usize, RemoteVersion)
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

    /// Install the target game.
    pub async fn task_install(&mut self) -> Result<(), BeansError>
    {
        let (latest_remote_id, latest_remote) = self.latest_remote_version();
        if let Some(cv) = self.current_version {
            if latest_remote_id < cv {
                println!("Installed version is newer than the latest remote version? (local: {}, remote: {})", cv, latest_remote_id);
                return Ok(());
            }
            if latest_remote_id == cv {
                println!("You've got the latest version installed already! (local: {}, remote: {})", cv, latest_remote_id);
                return Ok(());
            }
        }
        let presz_loc = Self::download_package(latest_remote).await?;
        if helper::file_exists(presz_loc.clone()) == false {
            eprintln!("Failed to find downloaded file!");
            std::process::exit(1);
        }

        match find_sourcemod_path() {
            Some(v) => {
                println!("Extracting game");
                Self::extract_package(presz_loc, v)?;
                AdastralVersionFile {
                    version: latest_remote_id.to_string()
                }.write()?;
                println!("Done! Make sure to restart steam before playing");
                Ok(())
            },
            None => {
                panic!("Failed to find sourcemod folder!");
            }
        }
    }

    fn extract_package(zstd_location: String, out_dir: String) -> Result<(), BeansError>
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
            Err(e) => Err(BeansError::TarExtractFailure(tar_tmp_location, out_dir, e)),
            Ok(_) => Ok(())
        }
    }

    async fn download_package(version: RemoteVersion) -> Result<String, BeansError>
    {
        if let Some(size) = version.pre_sz {
            if Self::has_free_space(size)? == false {
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

    /// Check if the sourcemod mod folder has enough free space.
    fn has_free_space(size: usize) -> Result<bool, BeansError>
    {
        match find_sourcemod_path() {
            Some(v) => {
                let space = helper::get_free_space(v)?;
                return Ok((size as u64) < space);
            },
            None => {
                Err(BeansError::SourceModLocationNotFound)
            }
        }
    }
    /// Check for any updates, and if there are any, we install them.
    pub async fn task_update(&mut self) -> Result<(), BeansError>
    {
        todo!()
    }
    /// Verify the current data for the target sourcemod.
    pub async fn task_verify(&mut self) -> Result<(), BeansError>
    {
        todo!()
    }
}



fn get_path() -> String
{
    let current_path = find_sourcemod_path();
    if let Some(x) = current_path {
        println!("Found sourcemods directory!\n{}", x);
        return x;
    }
    todo!();
}
#[derive(Debug)]
pub enum BeansError
{
    /// Failed to check if there is free space. Value is the location
    FreeSpaceCheckFailure(String),
    /// Failed to find the sourcemod mod folder.
    SourceModLocationNotFound,
    FileOpenFailure(String, std::io::Error),
    FileWriteFailure(String, std::io::Error),
    TarExtractFailure(String, String, std::io::Error),
    DownloadFailure(String, reqwest::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error)
}

impl<E> From<E> for BeansError
    where
        E: Into<reqwest::Error>,
{
    fn from(err: E) -> Self {
        BeansError::Reqwest(err.into())
    }
}