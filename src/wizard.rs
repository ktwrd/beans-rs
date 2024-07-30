use std::backtrace::Backtrace;

use async_recursion::async_recursion;
use log::{debug,
          error,
          info,
          trace};

use crate::{depends,
            flags,
            flags::LaunchFlag,
            helper,
            helper::{find_sourcemod_path,
                     parse_location,
                     InstallType},
            workflows::{CleanWorkflow,
                        InstallWorkflow,
                        UninstallWorkflow,
                        UpdateWorkflow,
                        VerifyWorkflow},
            BeansError,
            RunnerContext,
            SourceModDirectoryParam};

#[derive(Debug, Clone)]
pub struct WizardContext
{
    pub context: RunnerContext,
    pub menu_trigger_count: u32
}
impl WizardContext
{
    /// run the wizard!
    pub async fn run(sml_via: SourceModDirectoryParam) -> Result<(), BeansError>
    {
        depends::try_write_deps();
        if let Err(e) = depends::try_install_vcredist().await
        {
            sentry::capture_error(&e);
            println!("Failed to install vcredist! {:}", e);
            debug!("[WizardContext::run] {:#?}", e);
        }
        let sourcemod_path = parse_location(match sml_via
        {
            SourceModDirectoryParam::AutoDetect =>
            {
                debug!("[WizardContext::run] Auto-detecting sourcemods directory");
                get_path()
            }
            SourceModDirectoryParam::WithLocation(loc) =>
            {
                debug!("[WizardContext::run] Using specified location {}", loc);
                loc
            }
        });
        let version_list = match crate::version::get_version_list().await
        {
            Ok(v) => v,
            Err(e) =>
            {
                trace!("[WizardContext::run] Failed to run version::get_version_list()");
                trace!("{:#?}", e);
                sentry::capture_error(&e);
                return Err(e);
            }
        };

        if helper::install_state(Some(sourcemod_path.clone())) == InstallType::OtherSource
        {
            crate::version::update_version_file(Some(sourcemod_path.clone()))?;
        }

        let ctx = RunnerContext {
            sourcemod_path: sourcemod_path.clone(),
            remote_version_list: version_list,
            current_version: crate::version::get_current_version(Some(sourcemod_path)),
            appvar: crate::appvar::parse()
        };

        let mut i = Self {
            context: ctx,
            menu_trigger_count: 0u32
        };
        i.menu().await;
        return Ok(());
    }

    /// Show the menu
    /// When an invalid option is selected, this will be re-called.
    #[async_recursion]
    pub async fn menu<'a>(&'a mut self)
    {
        let av = crate::appvar::AppVarData::get();
        if self.menu_trigger_count == 0
        {
            if let Some(cv) = self.context.current_version
            {
                let (rv, _) = self.context.latest_remote_version();
                if cv < rv
                {
                    println!(
                        "======== A new update for {} is available! (v{rv}) ========",
                        av.mod_info.name_stylized
                    );
                }
            }
        }
        println!();
        println!("1 - Install or reinstall the game");
        println!("2 - Check for and apply any available updates");
        println!("3 - Verify and repair game files");
        println!("c - Clean up temporary files used by beans.");
        println!("u - Uninstall {}", av.mod_info.name_stylized);
        println!();
        println!("q - Quit");
        let user_input = helper::get_input("-- Enter option below --");
        match user_input.to_lowercase().as_str()
        {
            "1" | "install" => WizardContext::menu_error_catch(self.task_install().await),
            "2" | "update" => WizardContext::menu_error_catch(self.task_update().await),
            "3" | "verify" => WizardContext::menu_error_catch(self.task_verify().await),
            "c" | "clean" => Self::menu_error_catch(CleanWorkflow::wizard(&mut self.context)),
            "u" | "uninstall" =>
            {
                Self::menu_error_catch(UninstallWorkflow::wizard(&mut self.context).await)
            }
            "d" | "debug" =>
            {
                flags::add_flag(LaunchFlag::DEBUG_MODE);
                info!("Debug mode enabled!");
                self.menu().await;
            }
            "panic" =>
            {
                panic!()
            }
            "q" => std::process::exit(0),
            _ =>
            {
                println!("Unknown option \"{}\"", user_input);
                self.menu_trigger_count += 1;
                self.menu().await;
            }
        };
    }
    fn menu_error_catch(v: Result<(), BeansError>)
    {
        if let Err(e) = v
        {
            let b = Backtrace::capture();
            sentry::capture_error(&e);
            panic!("backtrace: {:#?}\n\nerror: {:#?}", b, e);
        }
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
    find_sourcemod_path().unwrap_or_else(|e| {
        error!("[get_path] Failed to automatically detect sourcemods folder!");
        debug!("{:#?}", e);
        prompt_sourcemod_location()
    })
}
fn prompt_sourcemod_location() -> String
{
    let res = helper::get_input("Please provide your sourcemods folder, then press enter.");
    return if !helper::file_exists(res.clone())
    {
        eprintln!("The location you provided doesn't exist. Try again.");
        prompt_sourcemod_location()
    }
    else if !helper::is_directory(res.clone())
    {
        eprintln!("The location you provided isn't a folder. Try again.");
        prompt_sourcemod_location()
    }
    else
    {
        res
    };
}
