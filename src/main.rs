use beans_rs::wizard;

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    let _ = winconsole::console::set_title(format!("beans v{}", beans_rs::VERSION).as_str());

    wizard::WizardContext::run().await;
}


