use rust_i18n::t;
rust_i18n::i18n!();

use std::str::FromStr;

use beans_core::{BeansError,
                 PANIC_MSG_CONTENT,
                 PAUSE_ONCE_DONE,
                 PROMPT_DO_WHATEVER,
                 appvar::AppVarData,
                 path::{dir_exists,
                        parse_location}};
use beans_rs::{RunnerContext,
               SourceModDirectoryParam,
               flags,
               flags::LaunchFlag,
               gui::DialogIconKind,
               helper,
               wizard,
               workflows::{CleanWorkflow,
                           InstallWorkflow,
                           UninstallWorkflow,
                           UpdateWorkflow,
                           VerifyWorkflow}};
use clap::{Arg,
           ArgAction,
           ArgMatches,
           Command};
use current_platform::COMPILED_ON;
use log::{LevelFilter,
          debug,
          error,
          info,
          trace,
          warn};

pub const DEFAULT_LOG_LEVEL_RELEASE: LevelFilter = LevelFilter::Info;
#[cfg(debug_assertions)]
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;
#[cfg(not(debug_assertions))]
pub const DEFAULT_LOG_LEVEL: LevelFilter = DEFAULT_LOG_LEVEL_RELEASE;

fn main()
{
    init_console();
    init_flags();
    // initialize sentry and custom panic handler for msgbox
    if cfg!(not(debug_assertions))
    {
        init_panic_handle();
        let sentry_opts = sentry::ClientOptions::new()
            .dsn(beans_core::SENTRY_URL)
            .maybe_release(sentry::release_name!())
            .debug(flags::has_flag(LaunchFlag::DEBUG_MODE))
            .max_breadcrumbs(200)
            .auto_session_tracking(true)
            .attach_stacktrace(true)
            .enable_logs(true)
            .send_default_pii(true);
        let _guard = sentry::init(sentry_opts);
        sentry::configure_scope(|scope| {
            let av = AppVarData::get();
            scope.set_tag("appvar.mod.mod_name", &av.mod_info.sourcemod_name);
            scope.set_tag("appvar.remote.base_url", &av.remote_info.base_url);
            scope.set_tag("appvar.remote.versions_url", &av.remote_info.versions_url);
        });
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            Launcher::run().await;
        });
}

#[cfg(target_os = "windows")]
fn init_console()
{
    winconsole::window::show(true);
    if let Err(e) =
        winconsole::console::set_title(format!("beans v{}", beans_core::VERSION).as_str())
    {
        trace!(
            "[init_console] {} {:#?}",
            t!("error.console.title.failure"),
            e
        );
    }
    if let Ok(mut input_mode) = winconsole::console::get_input_mode()
    {
        if input_mode.QuickEditMode
        {
            input_mode.QuickEditMode = false;
            if let Err(e) = winconsole::console::set_input_mode(input_mode)
            {
                debug!(
                    "[init_console] {} {:#?}",
                    t!("error.console.flag.failure", flag = "QuickEditMode"),
                    e
                );
                warn!(
                    "[init_console] {} QuickEditMode ({:})",
                    t!("error.console.flag.failure", flag = "QuickEditMode"),
                    e
                );
            }
        }
    }
}
#[cfg(not(target_os = "windows"))]
fn init_console()
{
    // do nothing
}

fn init_flags()
{
    flags::remove_flag(LaunchFlag::DEBUG_MODE);
    #[cfg(debug_assertions)]
    flags::add_flag(LaunchFlag::DEBUG_MODE);
    if beans_core::env::get_debug()
    {
        flags::add_flag(LaunchFlag::DEBUG_MODE);
    }
    flags::add_flag(LaunchFlag::STANDALONE_APP);
    beans_rs::logger::set_filter(DEFAULT_LOG_LEVEL);
    beans_rs::logger::log_to_stdout();
}

#[allow(dead_code)]
fn init_panic_handle()
{
    std::panic::set_hook(Box::new(move |info| {
        debug!("[panic::set_hook] {}", t!("error.msgbox"));
        let msg = beans_rs::helper::payload_message(info);
        info!("[panic] {}\n{:#?}", t!("error.fatal"), msg);
        custom_panic_handle(msg);
        debug!(
            "[panic::set_hook] {}",
            t!(
                "error.calling.handler",
                handler = "sentry_panic::panic_handler"
            )
        );
        sentry::integrations::panic::panic_handler(info);
        if flags::has_flag(LaunchFlag::DEBUG_MODE)
        {
            eprintln!("{:#?}", info);
        }
        logic_done();
    }));
}
#[allow(dead_code)]
fn custom_panic_handle(msg: String)
{
    unsafe {
        if !PAUSE_ONCE_DONE
        {
            return;
        }
    }
    let txt = PANIC_MSG_CONTENT
        .to_string()
        .replace("$err_msg", &msg)
        .replace("\\n", "\n");

    beans_rs::gui::DialogBuilder::new()
        .with_title(format!(
            "{} - {}",
            env!("CARGO_BIN_NAME"),
            t!("error.fatal")
        ))
        .with_icon(DialogIconKind::Error)
        .with_content(txt)
        .run();
}
#[warn(dead_code)]
/// should called once the logic flow is done!
/// will call `helper::get_input` when `PAUSE_ONCE_DONE` is `true`.
fn logic_done()
{
    unsafe {
        if PAUSE_ONCE_DONE
        {
            let _ = helper::get_input(&t!("info.press_enter"));
        }
    }
}

pub struct Launcher
{
    /// Output location. When none, `SourceModDirectoryParam::default()` will be
    /// used.
    pub to_location: Option<String>,
    /// Output of `Command.matches()`
    pub root_matches: ArgMatches
}

impl Launcher
{
    /// Create argument for specifying the location where the sourcemods
    /// directory is.
    fn create_location_arg() -> Arg
    {
        Arg::new("location")
            .long("location")
            .help(t!("args.location.about", program = env!("CARGO_BIN_NAME")))
            .required(false)
    }
    fn create_confirm_arg() -> Arg
    {
        Arg::new("confirm")
            .long("confirm")
            .help(t!("args.confirm.about"))
            .required(false)
            .action(ArgAction::SetTrue)
    }
    pub async fn run()
    {
        let cmd = Command::new(env!("CARGO_BIN_NAME"))
            .version(clap::crate_version!())
            .bin_name(clap::crate_name!())
            .subcommand(
                Command::new("wizard")
                    .about(t!("commands.wizard.about"))
                    .arg(Launcher::create_location_arg())
            )
            .subcommand(
                Command::new("install")
                    .about(t!("commands.install.about"))
                    .args([
                        Launcher::create_location_arg(),
                        Arg::new("from")
                            .long("from")
                            .help(t!("commands.install.from"))
                            .required(false),
                        Arg::new("target-version")
                            .long("target-version")
                            .help(t!("commands.install.target"))
                            .required(false),
                        Self::create_confirm_arg()
                    ])
            )
            .subcommand(
                Command::new("verify")
                    .about(t!("commands.verify.about"))
                    .arg(Launcher::create_location_arg())
            )
            .subcommand(
                Command::new("update")
                    .about(t!("commands.update.about"))
                    .arg(Launcher::create_location_arg())
            )
            .subcommand(Command::new("clean-tmp").about(t!("commands.clean.about")))
            .subcommand(
                Command::new("uninstall")
                    .about(t!("commands.uninstall.about"))
                    .args([Launcher::create_location_arg()])
            )
            .args([
                Arg::new("debug")
                    .long("debug")
                    .help(t!("args.debug.about"))
                    .action(ArgAction::SetTrue),
                Arg::new("no-debug")
                    .long("no-debug")
                    .help(t!("args.no-debug.about"))
                    .action(ArgAction::SetTrue),
                Arg::new("no-pause")
                    .long("no-pause")
                    .help(t!("args.no-pause.about"))
                    .action(ArgAction::SetTrue),
                Self::create_location_arg(),
                Self::create_confirm_arg()
            ]);
        println!(
            "{} v{} ({})",
            env!("CARGO_BIN_NAME"),
            beans_core::VERSION,
            COMPILED_ON
        );
        println!("Copyright (c) 2024 Kate Ward");
        println!("License {}", env!("CARGO_PKG_LICENSE"));
        println!();
        println!("{}", t!("intro.contributor_list"));
        println!("<{}/graphs/contributors>", env!("CARGO_PKG_REPOSITORY"));
        println!();

        let mut i = Self::new(&cmd.get_matches());
        if let Ok(Some(v)) = helper::beans_has_update().await
        {
            info!("{}", t!("intro.new_version"));
            info!("{}", v.html_url);
        }
        i.subcommand_processor().await;
    }
    pub fn new(matches: &ArgMatches) -> Self
    {
        let mut i = Self {
            to_location: None,
            root_matches: matches.clone()
        };
        i.set_debug();
        i.set_no_pause();
        i.set_prompt_do_whatever();
        i.to_location = Launcher::find_arg_sourcemods_location(&i.root_matches);

        i
    }

    /// add `LaunchFlag::DEBUG_MODE` to `flags` when the `--debug` parameter
    /// flag is used.
    pub fn set_debug(&mut self)
    {
        if self.root_matches.get_flag("no-debug")
        {
            flags::remove_flag(LaunchFlag::DEBUG_MODE);
            beans_rs::logger::set_filter(DEFAULT_LOG_LEVEL_RELEASE);
            info!("{}", t!("debug.disabled"));
        }
        else if self.root_matches.get_flag("debug")
        {
            flags::add_flag(LaunchFlag::DEBUG_MODE);
            beans_rs::logger::set_filter(LevelFilter::max());
            trace!("{}", t!("debug.enabled"));
        }
    }
    /// Set `PAUSE_ONCE_DONE` to `false` when `--no-pause` is provided.
    /// Otherwise, set it to `true`.
    pub fn set_no_pause(&mut self)
    {
        unsafe {
            PAUSE_ONCE_DONE = !self.root_matches.get_flag("no-pause");
        }
    }

    /// Set `self.to_location` when provided in the arguments.
    pub fn find_arg_sourcemods_location(matches: &ArgMatches) -> Option<String>
    {
        let mut sml_dir_manual: Option<String> = None;
        if let Some(x) = matches.get_one::<String>("location")
        {
            if !dir_exists(x.clone())
            {
                if let Err(e) = std::fs::create_dir(x)
                {
                    debug!("{:#?}", e);
                    error!(
                        "[Launcher::find_arg_sourcemods_location] {} {:?} ({:})",
                        t!("error.directory.create.failure"),
                        x,
                        e
                    );
                    panic!(
                        "[Launcher::find_arg_sourcemods_location] {} {:?}\n\n{:#?}",
                        t!("error.directory.create.failure"),
                        x,
                        e
                    )
                }
            }
            sml_dir_manual = Some(parse_location(x.to_string()));
            info!(
                "[Launcher::find_arg_sourcemods_location] {}",
                t!("info.found.args", item = x)
            );
        }
        sml_dir_manual
    }

    /// main handler for subcommand processing.
    pub async fn subcommand_processor(&mut self)
    {
        match self.root_matches.clone().subcommand()
        {
            Some(("install", i_matches)) =>
            {
                self.task_install(i_matches).await;
            }
            Some(("verify", v_matches)) =>
            {
                self.task_verify(v_matches).await;
            }
            Some(("update", u_matches)) =>
            {
                self.task_update(u_matches).await;
            }
            Some(("uninstall", ui_matches)) =>
            {
                self.task_uninstall(ui_matches).await;
            }
            Some(("wizard", wz_matches)) =>
            {
                self.to_location = Launcher::find_arg_sourcemods_location(wz_matches);
                self.task_wizard().await;
            }
            Some(("clean-tmp", _)) =>
            {
                self.task_clean_tmp().await;
            }
            _ =>
            {
                self.task_wizard().await;
            }
        }
    }

    pub fn set_prompt_do_whatever(&mut self)
    {
        if self.root_matches.get_flag("confirm")
        {
            unsafe {
                PROMPT_DO_WHATEVER = true;
            }
        }
    }

    /// Try and get `SourceModDirectoryParam`.
    /// Returns SourceModDirectoryParam::default() when `to_location` is `None`.
    fn try_get_smdp(&mut self) -> SourceModDirectoryParam
    {
        match &self.to_location
        {
            Some(v) => SourceModDirectoryParam::WithLocation(v.to_string()),
            None => SourceModDirectoryParam::default()
        }
    }

    /// handler for the `wizard` subcommand. it's also the default subcommand.
    pub async fn task_wizard(&mut self)
    {
        let x = self.try_get_smdp();
        if let Err(e) = wizard::WizardContext::run(x).await
        {
            panic!("{} {:#?}", t!("run.failure", task = "WizardContext"), e);
        }
        else
        {
            logic_done();
        }
    }

    /// handler for the `install` subcommand
    ///
    /// NOTE this function uses `panic!` when `InstallWorkflow::wizard` fails.
    /// panics are handled and are reported via sentry.
    pub async fn task_install(
        &mut self,
        matches: &ArgMatches
    )
    {
        self.to_location = Launcher::find_arg_sourcemods_location(matches);
        if matches.get_flag("confirm")
        {
            unsafe {
                PROMPT_DO_WHATEVER = true;
            }
        }

        let mut ctx = self.try_create_context().await;

        // call install_version when target-version is found.
        // we do this since target-version overrides the `from` parameter.
        //
        // `else if let` is used for checking the `--from` parameter,
        // so a return isn't required.
        if let Some(x) = matches.get_one::<String>("target-version")
        {
            self.task_install_version_specific(ctx, x.clone()).await;
        }
        // manually install from specific `.tar.zstd` file when the
        // --from parameter is provided. otherwise we install/reinstall
        // the latest version to whatever sourcemods directory is used
        else if let Some(x) = matches.get_one::<String>("from")
        {
            info!(
                "{}",
                t!(
                    "tasks.install.from_to",
                    previous = x.clone(),
                    current = ctx.sourcemod_path.clone()
                )
            );
            if let Err(e) =
                InstallWorkflow::install_from(x.clone(), ctx.sourcemod_path.clone(), None).await
            {
                error!(
                    "{}",
                    t!("error.run.failure", task = "InstallWorkflow::install_from")
                );
                sentry::capture_error(&e);
                panic!("{:#?}", e);
            }
            else
            {
                logic_done();
            }
        }
        else if let Err(e) = InstallWorkflow::wizard(&mut ctx).await
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "InstallWorkflow"),
                e
            );
        }
        else
        {
            logic_done();
        }
    }
    /// handler for the `install` subcommand where the `--target-version`
    /// parameter is provided.
    ///
    /// NOTE this function uses `expect` on `InstallWorkflow::install_version`.
    /// panics are handled and are reported via sentry.
    pub async fn task_install_version_specific(
        &mut self,
        ctx: RunnerContext,
        version_str: String
    )
    {
        let version = match usize::from_str(&version_str)
        {
            Ok(v) => v,
            Err(e) =>
            {
                sentry::capture_error(&e);
                error!(
                    "{} \"{version_str}\": {:#?}",
                    t!("error.parse.version.failure"),
                    e
                );
                logic_done();
                return;
            }
        };
        let mut wf = InstallWorkflow {
            context: ctx
        };
        if let Err(e) = wf.install_version(version).await
        {
            error!(
                "{}",
                t!(
                    "error.run.failure",
                    task = "InstallWorkflow::install_version"
                )
            );
            sentry::capture_error(&e);
            panic!("{:#?}", e);
        }
        else
        {
            logic_done();
        }
    }

    /// handler for the `verify` subcommand
    ///
    /// NOTE this function uses `panic!` when `VerifyWorkflow::wizard` fails.
    /// panics are handled and are reported via sentry.
    pub async fn task_verify(
        &mut self,
        matches: &ArgMatches
    )
    {
        self.to_location = Launcher::find_arg_sourcemods_location(matches);
        let mut ctx = self.try_create_context().await;

        if let Err(e) = VerifyWorkflow::wizard(&mut ctx).await
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "VerifyWorkflow"),
                e
            );
        }
        else
        {
            logic_done();
        }
    }

    /// handler for the `update` subcommand
    ///
    /// NOTE this function uses `panic!` when `UpdateWorkflow::wizard` fails.
    /// panics are handled and are reported via sentry.
    pub async fn task_update(
        &mut self,
        matches: &ArgMatches
    )
    {
        self.to_location = Launcher::find_arg_sourcemods_location(matches);
        let mut ctx = self.try_create_context().await;

        if let Err(e) = UpdateWorkflow::wizard(&mut ctx).await
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "UpdateWorkflow"),
                e
            );
        }
        else if let Err(e) = CleanWorkflow::wizard(&mut ctx)
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "CleanWorkflow"),
                e
            );
        }
        else
        {
            logic_done();
        }
    }

    /// Handler for the `clean-tmp` subcommand.
    ///
    /// NOTE this function uses `panic!` when `CleanWorkflow::wizard` fails.
    /// panics are handled and are reported via sentry.
    pub async fn task_clean_tmp(&mut self)
    {
        let mut ctx = self.try_create_context().await;
        if let Err(e) = CleanWorkflow::wizard(&mut ctx)
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "CleanWorkflow"),
                e
            );
        }
        else
        {
            logic_done();
        }
    }

    /// handler for the `uninstall` subcommand
    ///
    /// NOTE this function uses `panic!` when `UninstallWorkflow::wizard` fails.
    /// panics are handled and are reported via sentry.
    pub async fn task_uninstall(
        &mut self,
        matches: &ArgMatches
    )
    {
        self.to_location = Launcher::find_arg_sourcemods_location(matches);
        let mut ctx = self.try_create_context().await;

        if let Err(e) = UninstallWorkflow::wizard(&mut ctx).await
        {
            panic!(
                "{} {:#?}",
                t!("error.run.failure", task = "UninstallWorkflow"),
                e
            );
        }
        else
        {
            logic_done();
        }
    }

    /// try and create an instance of `RunnerContext` via the `create_auto`
    /// method while setting the `sml_via` parameter to the output of
    /// `self.try_get_smdp()`
    ///
    /// on failure, `panic!` is called. but that's okay because a dialog is
    /// shown (in `init_panic_handle`) and the error is reported via sentry.
    async fn try_create_context(&mut self) -> RunnerContext
    {
        match RunnerContext::create_auto(self.try_get_smdp()).await
        {
            Ok(v) => v,
            Err(e) =>
            {
                error!("[try_create_context] {:}", e);
                trace!("======== {} ========", t!("error.full"));
                trace!("{:#?}", &e);
                show_msgbox_error(format!("{:}", &e));

                let do_report = !matches!(
                    e,
                    BeansError::GameStillRunning { .. }
                        | BeansError::LatestVersionAlreadyInstalled { .. }
                        | BeansError::FreeSpaceCheckFailure { .. }
                );

                if do_report
                {
                    sentry::capture_error(&e);
                }
                logic_done();
                std::process::exit(1);
            }
        }
    }
}

fn show_msgbox_error(text: String)
{
    beans_rs::gui::DialogBuilder::new()
        .with_title(format!(
            "{} - {}",
            env!("CARGO_BIN_NAME"),
            t!("error.fatal")
        ))
        .with_icon(DialogIconKind::Error)
        .with_content(text.replace("\\n", "\n"))
        .run();
}
