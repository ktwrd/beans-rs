use std::str::FromStr;
use clap::{Arg, Command};
use beans_rs::{helper, RunnerContext, wizard};
use beans_rs::workflows::InstallWorkflow;

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    let _ = winconsole::console::set_title(format!("beans v{}", beans_rs::VERSION).as_str());
    #[cfg(debug_assertions)]
    unsafe { beans_rs::FORCE_DEBUG = true; }

    let cmd = Command::new("beans-rs")
        .version(clap::crate_version!())
        .bin_name(clap::crate_name!())
        .arg(Arg::new("debug")
            .long("debug")
            .help("Enable debug logging")
            .action(clap::ArgAction::SetTrue))
        .subcommand(Command::new("manual_install")
            .about("Manually install by specifying archive location and version.")
            .arg(
                Arg::new("location")
                    .help(".tar.zstd file location")
                    .long("location")
                    .action(clap::ArgAction::Set)
                    .required(true)
            )
            .arg(
                Arg::new("version")
                    .help("Version number")
                    .long("version")
                    .action(clap::ArgAction::Set)
                    .required(true)
            ));
    let matches = cmd.get_matches();
    // println!("{:#?}", matches);
    if matches.get_flag("debug") {
        println!("[beans_rs::main] Debug mode enabled");
        unsafe { beans_rs::FORCE_DEBUG = true; }
    }

    match matches.subcommand() {
        Some(("manual_install", mi_matches)) => {
            let loc = mi_matches.get_one::<String>("location").unwrap();
            let v = mi_matches.get_one::<String>("version").unwrap();
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
            let ctx = match RunnerContext::create_auto().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{:}", e);
                    if helper::do_debug() {
                        eprintln!("======== Full Error ========");
                        eprintln!("{:#?}", e);
                    }
                    std::process::exit(0);
                }
            };
            let smp_x = ctx.sourcemod_path.clone();
            if let Err(e) = InstallWorkflow::install_from(loc.clone(), smp_x, Some(version)).await {
                println!("Failed to run InstallWorkflow::install_from! {:}", e);
                if helper::do_debug() {
                    eprintln!("{:#?}", e);
                }
            }
            let _ = helper::get_input("Press enter/return to exit");
            std::process::exit(0);
        },
        _ => {
            wizard::WizardContext::run().await;
        }
    }
}


