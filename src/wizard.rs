use crate::{BeansError, depends, helper, RunnerContext};
use crate::helper::{find_sourcemod_path, InstallType};
use crate::version::{AdastralVersionFile, RemoteVersion};
use async_recursion::async_recursion;

#[derive(Debug, Clone)]
pub struct WizardContext
{
    pub context: RunnerContext
}
impl WizardContext
{
    /// run the wizard!
    pub async fn run()
    {
        depends::try_write_deps();
        depends::try_install_vcredist();
        let sourcemod_path = get_path();
        let version_list = crate::version::get_version_list().await;

        if helper::install_state() == InstallType::OtherSource {
            crate::version::update_version_file();
        }

        let ctx = RunnerContext {
            sourcemod_path,
            remote_version_list: version_list,
            current_version: crate::version::get_current_version()
        };

        let mut i = Self
        {
            context: ctx
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

    /// Install the target game.
    pub async fn task_install(&mut self) -> Result<(), BeansError>
    {
        let (latest_remote_id, latest_remote) = self.context.latest_remote_version();
        if let Some(cv) = self.context.current_version {
            if latest_remote_id < cv {
                println!("Installed version is newer than the latest remote version? (local: {}, remote: {})", cv, latest_remote_id);
                return Ok(());
            }
            if latest_remote_id == cv {
                println!("You've got the latest version installed already! (local: {}, remote: {})", cv, latest_remote_id);
                return Ok(());
            }
        }
        let presz_loc = RunnerContext::download_package(latest_remote).await?;
        if helper::file_exists(presz_loc.clone()) == false {
            eprintln!("Failed to find downloaded file!");
            std::process::exit(1);
        }

        match find_sourcemod_path() {
            Some(v) => {
                println!("Extracting game");
                RunnerContext::extract_package(presz_loc, v)?;
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