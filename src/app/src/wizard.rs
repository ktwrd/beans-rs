use std::backtrace::Backtrace;

use async_recursion::async_recursion;
use beans_core::{BeansError,
                 appvar::AppVarData,
                 env::get_disable_aria2c,
                 path::{file_exists,
                        is_directory,
                        parse_location}};
use log::{debug,
          error,
          info,
          trace};

use crate::{RunnerContext,
            SourceModDirectoryParam,
            depends,
            flags,
            flags::LaunchFlag,
            helper,
            helper::{InstallType,
                     find_sourcemod_path},
            workflows::{CleanWorkflow,
                        InstallWorkflow,
                        UninstallWorkflow,
                        UpdateWorkflow,
                        VerifyWorkflow}};

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
        WizardContext::check_aria();
        if let Err(e) = depends::try_install_vcredist().await
        {
            sentry::capture_error(&e);

            println!(
                "{} {:}",
                t!("error.install", item = t!("dependency.software.vcredist")),
                e
            );
            debug!("[WizardContext::run] {:#?}", e);
        }
        let sourcemod_path = parse_location(match sml_via
        {
            SourceModDirectoryParam::AutoDetect =>
            {
                debug!("[WizardContext::run] {}", t!("tasks.detecting_sm_dir"));
                get_path()
            }
            SourceModDirectoryParam::WithLocation(loc) =>
            {
                debug!(
                    "[WizardContext::run] {}",
                    t!("args.location", location = loc)
                );
                loc
            }
        });
        let version_list = match crate::version::get_version_list().await
        {
            Ok(v) => v,
            Err(e) =>
            {
                trace!(
                    "[WizardContext::run] {}",
                    t!("error.run", task = "version::get_version_list()")
                );
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
            current_version: crate::version::get_current_version(Some(sourcemod_path)).await,
            appvar: AppVarData::get()
        };

        let mut i = Self {
            context: ctx,
            menu_trigger_count: 0u32
        };
        i.menu().await;
        Ok(())
    }

    fn check_aria()
    {
        if !get_disable_aria2c()
        {
            if crate::aria2::get_executable_location().is_none()
            {
                info!("{}", t!("aria2.missing"));
            }
        }

        if get_disable_aria2c() && crate::aria2::get_executable_location().is_some()
        {
            info!("== {} ==", t!("aria2.disable"));
        }
    }

    /// Show the menu
    /// When an invalid option is selected, this will be re-called.
    #[async_recursion]
    pub async fn menu<'a>(&'a mut self)
    {
        let av = AppVarData::get();
        if self.menu_trigger_count == 0
        {
            if let Some(cv) = self.context.current_version
            {
                let (rv, _) = self.context.latest_remote_version();
                if cv < rv
                {
                    println!(
                        "======== {} ========",
                        t!(
                            "info.update_available",
                            game = av.mod_info.name_stylized,
                            remote = rv,
                            current = cv
                        )
                    );
                    println!();
                }
            }
        }
        println!("{}", t!("text.menu", game = av.mod_info.name_stylized));
        let user_input = helper::get_input(format!("-- {} --", t!("text.extra.choice")).as_str());
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
                info!("{}", t!("debug.enabled"));
                self.menu().await;
            }
            "panic" =>
            {
                panic!()
            }
            "q" => std::process::exit(0),
            _ =>
            {
                println!("{} \"{}\"", t!("error.bad_choice"), user_input);
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
        if let Err(e) = UpdateWorkflow::wizard(&mut self.context).await
        {
            error!("{} {:#?}", t!("error.run", task = "UpdateWorkflow"), e);
            return Err(e);
        }

        if let Err(e) = CleanWorkflow::wizard(&mut self.context)
        {
            error!("{} {:#?}", t!("error.run", task = "CleanWorkflow"), e);
            return Err(e);
        }
        Ok(())
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
        error!("[get_path] {}", t!("error.missing_sm_dir"));
        debug!("{:#?}", e);
        prompt_sourcemod_location()
    })
}

fn prompt_sourcemod_location() -> String
{
    let res = helper::get_input(&t!("args.sm_dir"));
    if !file_exists(res.clone())
    {
        eprintln!("{}", t!("error.directory.missing"));
        prompt_sourcemod_location()
    }
    else if !is_directory(res.clone())
    {
        eprintln!("{}", t!("error.directory.bad"));
        prompt_sourcemod_location()
    }
    else
    {
        res
    }
}
