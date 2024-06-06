#![feature(panic_info_message)]

use std::str::FromStr;
use clap::{Arg, ArgAction, ArgMatches, Command};
use log::{debug, error, info, LevelFilter, trace};
use beans_rs::{flags, helper, PANIC_MSG_CONTENT, RunnerContext, wizard};
use beans_rs::flags::LaunchFlag;
use beans_rs::helper::parse_location;
use beans_rs::SourceModDirectoryParam;
use beans_rs::workflows::{InstallWorkflow, UpdateWorkflow, VerifyWorkflow};

pub const DEFAULT_LOG_LEVEL_RELEASE: LevelFilter = LevelFilter::Info;
#[cfg(debug_assertions)]
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;
#[cfg(not(debug_assertions))]
pub const DEFAULT_LOG_LEVEL: LevelFilter = DEFAULT_LOG_LEVEL_RELEASE;

fn main() {
    #[cfg(target_os = "windows")]
    let _ = winconsole::console::set_title(format!("beans v{}", beans_rs::VERSION).as_str());

    init_flags();
    // initialize sentry and custom panic handler for msgbox
    let _guard = sentry::init((beans_rs::SENTRY_URL, sentry::ClientOptions {
        release: sentry::release_name!(),
        debug: flags::has_flag(LaunchFlag::DEBUG_MODE),
        max_breadcrumbs: 100,
        auto_session_tracking: true,
        ..Default::default()
    }));
    init_panic_handle();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            Launcher::run().await;
        });
}
fn init_flags()
{
    flags::remove_flag(LaunchFlag::DEBUG_MODE);
    #[cfg(debug_assertions)]
    flags::add_flag(LaunchFlag::DEBUG_MODE);
    if std::env::var("BEANS_DEBUG").is_ok_and(|x| x == "1") {
        flags::add_flag(LaunchFlag::DEBUG_MODE);
    }
    flags::add_flag(LaunchFlag::STANDALONE_APP);
    beans_rs::logger::set_filter(DEFAULT_LOG_LEVEL);
    beans_rs::logger::log_to_stdout();
}
fn init_panic_handle()
{
    std::panic::set_hook(Box::new(move |info| {
        debug!("[panic::set_hook] showing msgbox to notify user");
        let mut x = String::new();
        if let Some(m) = info.message() {
            x = format!("{:#?}", m);
        }
        custom_panic_handle(x);
        debug!("[panic::set_hook] calling sentry_panic::panic_handler");
        sentry::integrations::panic::panic_handler(&info);
        if flags::has_flag(LaunchFlag::DEBUG_MODE) {
            eprintln!("{:#?}", info);
        }
        logic_done();
    }));
}
#[cfg(target_os = "windows")]
fn fix_msgbox_txt(txt: String) -> String {
    txt.replace("\\n", "\r\n")
}
#[cfg(not(target_os = "windows"))]
fn fix_msgbox_txt(txt: String) -> String {
    txt
}
fn custom_panic_handle(msg: String)
{
    unsafe {
        if beans_rs::HEADLESS == false {
            let mut txt = PANIC_MSG_CONTENT.to_string().replace("$err_msg", &msg);
            txt = fix_msgbox_txt(txt);
            std::thread::spawn(move || {

                let d = native_dialog::MessageDialog::new()
                    .set_type(native_dialog::MessageType::Error)
                    .set_title("beans - fatal error!")
                    .set_text(&txt)
                    .show_alert();
                if let Err(e) = d {
                    sentry::capture_error(&e);
                    eprintln!("Failed to show MessageDialog {:#?}", e);
                    eprintln!("[msgbox_panic] Come on, we failed to show a messagebox? Well, the error has been reported and we're on it.");
                    eprintln!("[msgbox_panic] PLEASE report this to kate@dariox.club with as much info as possible <3");
                }
            });
        } else {
            info!("This error has been reported to the developers");
        }
    }
}
/// should called once the logic flow is done!
/// will call `helper::get_input` when `PAUSE_ONCE_DONE` is `true`.
fn logic_done()
{
    unsafe {
        if beans_rs::HEADLESS == false {
            let _ = helper::get_input("Press enter/return to exit");
        }
    }
}
pub struct Launcher {
    /// Output location. When none, `SourceModDirectoryParam::default()` will be used.
    pub to_location: Option<String>,
    /// Output of `Command.matches()`
    pub root_matches: ArgMatches
}
impl Launcher
{
    /// Create argument for specifying the location where the sourcemods directory is.
    fn create_location_arg() -> Arg
    {
        Arg::new("location")
            .long("location")
            .help("Manually specify sourcemods directory. When not provided, beans-rs will automatically detect the sourcemods directory.")
            .required(false)
    }
    pub async fn run()
    {
        let cmd = Command::new("beans-rs")
            .version(clap::crate_version!())
            .bin_name(clap::crate_name!())
            .subcommand(Command::new("wizard")
                .about("Use the wizard to install. (Default subcommand)")
                .arg(Launcher::create_location_arg()))
            .subcommand(Command::new("install")
                .about("Install to a custom location.")
                .args([
                    Launcher::create_location_arg(),
                    Arg::new("from")
                        .long("from")
                        .help("Location to where the .tar.zstd file is that you want to install from.")
                        .required(false),
                    Arg::new("target-version")
                        .long("target-version")
                        .help("Specify the version to install. Ignored when [--from] is used.")
                        .required(false)]))
            .subcommand(Command::new("verify")
                .about("Verify your current installation")
                .arg(Launcher::create_location_arg()))
            .subcommand(Command::new("update")
                .about("Update your installation")
                .arg(Launcher::create_location_arg()))
            .args([
                Arg::new("debug")
                    .long("debug")
                    .help("Enable debug logging")
                    .action(ArgAction::SetTrue),
                Arg::new("no-debug")
                    .long("no-debug")
                    .help("Disable mode. Mainly used for debug builds to not spew into the console.")
                    .action(ArgAction::SetTrue),
                Arg::new("headless")
                    .long("headless")
                    .help("Provide this when you are using this application in an environment where this is being used in an automated script or there is no X11 display or Wayland session associated with the process that started this.")
                    .action(ArgAction::SetTrue),
                Launcher::create_location_arg()
            ]);

        let mut i = Self::new(&cmd.get_matches());
        i.subcommand_processor().await;
    }
    pub fn new(matches: &ArgMatches) -> Self {
        let mut i = Self {
            to_location: None,
            root_matches: matches.clone()
        };
        i.set_debug();
        i.set_no_pause();
        i.to_location = Launcher::find_arg_sourcemods_location(&i.root_matches);

        return i;
    }

    /// add `LaunchFlag::DEBUG_MODE` to `flags` when the `--debug` parameter flag is used.
    pub fn set_debug(&mut self)
    {
        if self.root_matches.get_flag("no-debug") {
            flags::remove_flag(LaunchFlag::DEBUG_MODE);
            beans_rs::logger::set_filter(DEFAULT_LOG_LEVEL_RELEASE);
            info!("Disabled Debug Mode");
        }
        else if self.root_matches.get_flag("debug") {
            flags::add_flag(LaunchFlag::DEBUG_MODE);
            beans_rs::logger::set_filter(LevelFilter::max());
            trace!("Debug mode enabled");
        }
    }
    /// Set `PAUSE_ONCE_DONE` to `false` when `--no-pause` is provided. Otherwise, set it to `true`.
    pub fn set_no_pause(&mut self)
    {
        unsafe {
            beans_rs::HEADLESS = self.root_matches.get_flag("headless");
        }
    }

    /// Set `self.to_location` when provided in the arguments.
    pub fn find_arg_sourcemods_location(matches: &ArgMatches) -> Option<String>
    {
        let mut sml_dir_manual: Option<String> = None;
        if let Some(x) = matches.get_one::<String>("location") {
            sml_dir_manual = Some(parse_location(x.to_string()));
            info!("[Launcher::set_to_location] Found in arguments! {}", x);
        }
        sml_dir_manual
    }

    /// main handler for subcommand processing.
    pub async fn subcommand_processor(&mut self)
    {
        match self.root_matches.clone().subcommand() {
            Some(("install", i_matches)) => {
                self.task_install(i_matches).await;
            },
            Some(("verify", v_matches)) => {
                self.task_verify(v_matches).await;
            },
            Some(("update", u_matches)) => {
                self.task_update(u_matches).await;
            },
            Some(("wizard", wz_matches)) => {
                self.to_location = Launcher::find_arg_sourcemods_location(wz_matches);
                self.task_wizard().await;
            },
            _ => {
                self.task_wizard().await;
            }
        }
    }

    /// Try and get `SourceModDirectoryParam`.
    /// Returns SourceModDirectoryParam::default() when `to_location` is `None`.
    fn try_get_smdp(&mut self) -> SourceModDirectoryParam
    {
        match &self.to_location {
            Some(v) => {
                SourceModDirectoryParam::WithLocation(v.to_string())
            },
            None => SourceModDirectoryParam::default()
        }
    }

    /// handler for the `wizard` subcommand. it's also the default subcommand.
    pub async fn task_wizard(&mut self)
    {
        let x = self.try_get_smdp();
        if let Err(e) = wizard::WizardContext::run(x).await {
            panic!("Failed to run WizardContext {:#?}", e);
        } else {
            logic_done();
        }
    }

    /// handler for the `install` subcommand
    ///
    /// NOTE this function uses `panic!` when `InstallWorkflow::wizard` fails. panics are handled
    /// and are reported via sentry.
    pub async fn task_install(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_sourcemods_location(&matches);
        let mut ctx = self.try_create_context().await;

        // call install_version when target-version is found.
        // we do this since target-version overrides the `from` parameter.
        //
        // `else if let` is used for checking the `--from` parameter,
        // so a return isn't required.
        if let Some(x) = matches.get_one::<String>("target-version") {
            self.task_install_version_specific(ctx, x.clone()).await;
        }

        // manually install from specific `.tar.zstd` file when the
        // --from parameter is provided. otherwise we install/reinstall
        // the latest version to whatever sourcemods directory is used
        else if let Some(x) = matches.get_one::<String>("from") {
            info!("Manually installing from {} to {}", x.clone(), ctx.sourcemod_path.clone());
            if let Err(e) = InstallWorkflow::install_from(x.clone(), ctx.sourcemod_path.clone(), None).await {
                error!("Failed to run InstallWorkflow::install_from");
                sentry::capture_error(&e);
                panic!("{:#?}", e);
            } else {
                logic_done();
            }
        } else {
            if let Err(e) = InstallWorkflow::wizard(&mut ctx).await {
                panic!("Failed to run InstallWorkflow {:#?}", e);
            } else {
                logic_done();
            }
        }
    }
    /// handler for the `install` subcommand where the `--target-version`
    /// parameter is provided.
    ///
    /// NOTE this function uses `expect` on `InstallWorkflow::install_version`. panics are handled
    /// and are reported via sentry.
    pub async fn task_install_version_specific(&mut self, ctx: RunnerContext, version_str: String)
    {
        let version = match usize::from_str(&version_str) {
            Ok(v) => v,
            Err(e) => {
                sentry::capture_error(&e);
                error!("Failed to parse version argument \"{version_str}\": {:#?}", e);
                logic_done();
                return;
            }
        };
        let mut wf = InstallWorkflow
        {
            context: ctx
        };
        if let Err(e) = wf.install_version(version).await {
            error!("Failed to run InstallWorkflow::install_version");
            sentry::capture_error(&e);
            panic!("{:#?}", e);
        } else {
            logic_done();
        }
    }

    /// handler for the `verify` subcommand
    ///
    /// NOTE this function uses `panic!` when `VerifyWorkflow::wizard` fails. panics are handled
    /// and are reported via sentry.
    pub async fn task_verify(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_sourcemods_location(&matches);
        let mut ctx = self.try_create_context().await;

        if let Err(e) = VerifyWorkflow::wizard(&mut ctx).await {
            panic!("Failed to run VerifyWorkflow {:#?}", e);
        } else {
            logic_done();
        }
    }

    /// handler for the `update` subcommand
    ///
    /// NOTE this function uses `panic!` when `UpdateWorkflow::wizard` fails. panics are handled
    /// and are reported via sentry.
    pub async fn task_update(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_sourcemods_location(&matches);
        let mut ctx = self.try_create_context().await;

        if let Err(e) = UpdateWorkflow::wizard(&mut ctx).await {
            panic!("Failed to run UpdateWorkflow {:#?}", e);
        } else {
            logic_done();
        }
    }

    /// try and create an instance of `RunnerContext` via the `create_auto` method while setting
    /// the `sml_via` parameter to the output of `self.try_get_smdp()`
    ///
    /// on failure, `panic!` is called. but that's okay because a dialog is shown (in
    /// `init_panic_handle`) and the error is reported via sentry.
    async fn try_create_context(&mut self) -> RunnerContext {
        match RunnerContext::create_auto(self.try_get_smdp()).await {
            Ok(v) => v,
            Err(e) => {
                error!("[try_create_context] {:}", e);
                trace!("======== Full Error ========");
                trace!("{:#?}", &e);
                show_msgbox_error(format!("{:}", &e));
                sentry::capture_error(&e);
                logic_done();
                std::process::exit(1);
            }
        }
    }
}
fn show_msgbox_error(text: String) {
    unsafe {
        if beans_rs::HEADLESS == false {
            std::thread::spawn(move || {
                let d = native_dialog::MessageDialog::new()
                    .set_type(native_dialog::MessageType::Error)
                    .set_title("beans - fatal error!")
                    .set_text(&format!("{}", fix_msgbox_txt(text)))
                    .show_alert();
                if let Err(e) = d {
                    sentry::capture_error(&e);
                    eprintln!("Failed to show MessageDialog {:#?}", e);
                }
            });
        }
    }
}


