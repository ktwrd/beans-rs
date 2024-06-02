use std::str::FromStr;
use clap::{Arg, ArgAction, ArgMatches, Command};
use log::{debug, error, info, LevelFilter, trace};
use beans_rs::{flags, helper, PANIC_MSG_CONTENT, RunnerContext, wizard};
use beans_rs::flags::LaunchFlag;
use beans_rs::helper::parse_location;
use beans_rs::SourceModDirectoryParam;
use beans_rs::workflows::InstallWorkflow;

#[cfg(debug_assertions)]
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;
#[cfg(not(debug_assertions))]
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() {
    #[cfg(target_os = "windows")]
    let _ = winconsole::console::set_title(format!("beans v{}", beans_rs::VERSION).as_str());

    init_flags();
    // initialize sentry and custom panic handler for msgbox
    let _guard = sentry::init((beans_rs::SENTRY_URL, sentry::ClientOptions {
        release: sentry::release_name!(),
        debug: flags::has_flag(LaunchFlag::DEBUG_MODE),
        max_breadcrumbs: 100,
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
    #[cfg(debug_assertions)]
    flags::add_flag(LaunchFlag::DEBUG_MODE);
    if std::env::var("BEANS_DEBUG").is_ok_and(|x| x == "1") {
        flags::add_flag(LaunchFlag::DEBUG_MODE);
    }
    flags::add_flag(LaunchFlag::STANDALONE_APP);
    simple_logging::log_to_stderr(DEFAULT_LOG_LEVEL);
}
fn init_panic_handle()
{
    std::panic::set_hook(Box::new(move |info| {
        debug!("[panic::set_hook] showing msgbox to notify user");
        custom_panic_handle();
        debug!("[panic::set_hook] calling sentry_panic::panic_handler");
        sentry::integrations::panic::panic_handler(&info);
        if flags::has_flag(LaunchFlag::DEBUG_MODE) {
            eprintln!("{:#?}", info);
        }
        logic_done();
    }));
}
fn custom_panic_handle()
{
    std::thread::spawn(|| {
        let d = native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_title("beans - fatal error!")
            .set_text(PANIC_MSG_CONTENT)
            .show_alert();
        if let Err(e) = d {
            sentry::capture_error(&e);
            eprintln!("Failed to show MessageDialog {:#?}", e);
            eprintln!("[msgbox_panic] Come on, we failed to show a messagebox? Well, the error has been reported and we're on it.");
            eprintln!("[msgbox_panic] PLEASE report this to kate@dariox.club with as much info as possible <3");
        }
    });
}
/// should called once the logic flow is done!
/// will call `helper::get_input` when `PAUSE_ONCE_DONE` is `true`.
fn logic_done()
{
    unsafe {
        if PAUSE_ONCE_DONE {
            let _ = helper::get_input("Press enter/return to exit");
        }
    }
}
/// once everything is done, do we wait for the user to press enter before exiting?
///
/// just like the `pause` thing in batch.
pub static mut PAUSE_ONCE_DONE: bool = false;
pub struct Launcher {
    /// Output location. When none, `SourceModDirectoryParam::default()` will be used.
    pub to_location: Option<String>,
    /// Output of `Command.matches()`
    pub root_matches: ArgMatches
}
impl Launcher
{
    pub async fn run()
    {
        let arg_to = Arg::new("to")
            .long("to")
            .help("Manually specify sourcemods directory. When not provided, beans-rs will automatically detect the sourcemods directory.")
            .required(false);
        let cmd = Command::new("beans-rs")
            .version(clap::crate_version!())
            .bin_name(clap::crate_name!())
            .arg(Arg::new("debug")
                .long("debug")
                .help("Enable debug logging")
                .action(clap::ArgAction::SetTrue))
            .arg(arg_to.clone())
            .subcommand(Command::new("manual_install")
                .about("Manually install by specifying archive location and version.")
                .arg(
                    Arg::new("location")
                        .help(".tar.zstd file location")
                        .long("location")
                        .action(clap::ArgAction::Set)
                        .required(true),
                )
                .arg(
                    Arg::new("version")
                        .help("Version number")
                        .long("version")
                        .action(clap::ArgAction::Set)
                        .required(true)
                ))
            .subcommand(Command::new("wizard")
                .about("Use the wizard to install. (Default subcommand)")
                .arg(arg_to.clone()))
            .subcommand(Command::new("install")
                .about("Install to a custom location.")
                .arg(arg_to.clone()));

        let mut i = Self {
            to_location: None,
            root_matches: cmd.get_matches()
        };
        i.set_debug();
        i.to_location = Launcher::find_arg_to(&i.root_matches);
        i.subcommand_processor().await;
    }
    pub fn set_debug(&mut self)
    {
        if self.root_matches.get_flag("debug") {
            flags::add_flag(LaunchFlag::DEBUG_MODE);
            simple_logging::log_to_stderr(LevelFilter::Trace);
            trace!("Debug mode enabled");
        }
    }
    /// Set `self.to_location` when provided in the arguments.
    pub fn find_arg_to(matches: &ArgMatches) -> Option<String>
    {
        let mut sml_dir_manual: Option<String> = None;
        if let Some(x) = matches.get_one::<String>("to") {
            sml_dir_manual = Some(parse_location(x.to_string()));
            info!("[Launcher::set_to_location] Found in arguments! {}", x);
        }
        sml_dir_manual
    }
    pub async fn subcommand_processor(&mut self)
    {
        match self.root_matches.clone().subcommand() {
            Some(("manual_install", mi_matches)) => {
                self.task_manual_install(mi_matches).await;
            },
            Some(("wizard", wz_matches)) => {
                self.to_location = Launcher::find_arg_to(wz_matches);
                self.task_wizard().await;
            },
            _ => {
                self.task_wizard().await;
            }
        }
    }

    pub async fn task_manual_install(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_to(&matches);
        let loc = matches.get_one::<String>("location").unwrap();
        let v = matches.get_one::<String>("version").unwrap();
        let version = match usize::from_str(v) {
            Ok(v) => v,
            Err(e) => {
                sentry::capture_error(&e);
                eprintln!("Failed to parse version argument: {:#?}", e);
                return;
            }
        };
        let ctx = self.try_create_context().await;
        let smp_x = ctx.sourcemod_path.clone();
        if let Err(e) = InstallWorkflow::install_from(loc.clone(), smp_x, Some(version)).await {
            sentry::capture_error(&e);
            panic!("Failed to run InstallWorkflow::install_from {:#?}", e);
        }
        let _ = helper::get_input("Press enter/return to exit");
        std::process::exit(0);
    }
    fn try_get_smdp(&mut self) -> SourceModDirectoryParam
    {
        match &self.to_location {
            Some(v) => {
                SourceModDirectoryParam::WithLocation(v.to_string())
            },
            None => SourceModDirectoryParam::default()
        }
    }
    pub async fn task_wizard(&mut self)
    {
        let x = self.try_get_smdp();
        if let Err(e) = wizard::WizardContext::run(x).await {
            panic!("Failed to run WizardContext {:#?}", e);
        }
    }
    pub async fn task_install(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_to(&matches);
        let mut ctx = self.try_create_context().await;
        if let Err(e) = InstallWorkflow::wizard(&mut ctx).await {
            panic!("Failed to run InstallWorkflow {:#?}", e);
        }
    }
    async fn try_create_context(&mut self) -> RunnerContext {
        match RunnerContext::create_auto(self.try_get_smdp()).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{:}", e);
                trace!("======== Full Error ========");
                trace!("{:#?}", e);
                let _ = helper::get_input("Press enter/return to exit");
                panic!("Failed to create RunnerContext {:#?}", e);
            }
        }
    }
}


