use clap::{Arg, Command};
use beans_rs::wizard;

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
            .action(clap::ArgAction::SetTrue));
    let matches = cmd.get_matches();
    // println!("{:#?}", matches);
    if matches.get_flag("debug") {
        println!("[beans_rs::main] Debug mode enabled");
        unsafe { beans_rs::FORCE_DEBUG = true; }
    }

    wizard::WizardContext::run().await;
}


