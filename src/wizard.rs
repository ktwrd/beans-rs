use crate::{BeansError, depends, helper, RunnerContext};
use crate::helper::{find_sourcemod_path, InstallType};
use async_recursion::async_recursion;
use crate::workflows::{InstallWorkflow, UpdateWorkflow, VerifyWorkflow};

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
            eprintln!("{:}", e);
            if helper::do_debug() {
                eprintln!("======== Full Error ========");
                eprintln!("{:#?}", e);
            }
        }
        std::process::exit(0)
    }

    /// Install the target game.
    pub async fn task_install(&mut self) -> Result<(), BeansError>
    {
        InstallWorkflow::wizard(&mut self.context).await
    }

    /// Check for any updates, and if there are any, we install them.
    pub async fn task_update(&mut self) -> Result<(), BeansError>
    {
        UpdateWorkflow::wizard(&mut self.context).await
    }
    /// Verify the current data for the target sourcemod.
    pub async fn task_verify(&mut self) -> Result<(), BeansError>
    {
        VerifyWorkflow::wizard(&mut self.context).await
    }
}



fn get_path() -> String
{
    let current_path = find_sourcemod_path();
    if let Ok(x) = current_path {
        println!("Found sourcemods directory!\n{}", x);
        return x;
    }
    if let Err(e) = current_path {
        if helper::do_debug() {
            eprintln!("[wizard::get_path] {} {:#?}", BeansError::SourceModLocationNotFound, e);
        }
    }
    todo!();
}