use std::str::FromStr;
use clap::{Arg, ArgMatches, Command};
use log::{debug, info, LevelFilter, trace};
use beans_rs::{flags, helper, PANIC_MSG_CONTENT, RunnerContext, wizard};
use beans_rs::helper::parse_location;
use beans_rs::SourceModDirectoryParam;
use beans_rs::workflows::InstallWorkflow;

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    let _ = winconsole::console::set_title(format!("beans v{}", beans_rs::VERSION).as_str());
    #[cfg(debug_assertions)]
    unsafe { beans_rs::FORCE_DEBUG = true; }

    Launcher::run().await;
}

pub struct Launcher {
    /// Output location. When none, `SourceModDirectoryParam::default()` will be used.
    pub to_location: Option<String>,
    /// Do the arguments contain the `debug` flag?
    pub has_debug: bool,
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
            has_debug: false,
            root_matches: cmd.get_matches()
        };
        i.set_debug();
        i.to_location = Launcher::find_arg_to(&i.root_matches);
        i.subcommand_processor().await;
    }
    pub fn set_debug(&mut self)
    {
        if self.root_matches.get_flag("debug") {
            println!("[beans_rs::main] Debug mode enabled");
            unsafe { beans_rs::FORCE_DEBUG = true; }
        }
        unsafe {
            self.has_debug = beans_rs::FORCE_DEBUG;
        }
    }
    /// Set `self.to_location` when provided in the arguments.
    pub fn find_arg_to(matches: &ArgMatches) -> Option<String>
    {
        let mut sml_dir_manual: Option<String> = None;
        if let Some(x) = matches.get_one::<String>("to") {
            sml_dir_manual = Some(parse_location(x.to_string()));
            if helper::do_debug() {
                println!("[Launcher::set_to_location] Found in arguments! {}", x);
            }
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
                eprintln!("Failed to parse version argument: {:}", e);
                if helper::do_debug() {
                    eprintln!("{:#?}", e);
                }
                return;
            }
        };
        let ctx = self.try_create_context().await;
        let smp_x = ctx.sourcemod_path.clone();
        if let Err(e) = InstallWorkflow::install_from(loc.clone(), smp_x, Some(version)).await {
            println!("Failed to run InstallWorkflow::install_from! {:}", e);
            if helper::do_debug() {
                eprintln!("{:#?}", e);
            }
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
        wizard::WizardContext::run(x).await;
    }
    pub async fn task_install(&mut self, matches: &ArgMatches)
    {
        self.to_location = Launcher::find_arg_to(&matches);
        let mut ctx = self.try_create_context().await;
        if let Err(e) = InstallWorkflow::wizard(&mut ctx).await {
            eprintln!("Failed to install {:}", e);
            if helper::do_debug() {
                eprintln!("{:#?}", e);
            }
        }
    }
    async fn try_create_context(&mut self) -> RunnerContext {
        match RunnerContext::create_auto(self.try_get_smdp()).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{:}", e);
                if helper::do_debug() {
                    eprintln!("======== Full Error ========");
                    eprintln!("{:#?}", e);
                }
                std::process::exit(0);
            }
        }
    }
}


