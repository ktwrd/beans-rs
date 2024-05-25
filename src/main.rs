use beans_rs::wizard;

pub mod depends;

#[tokio::main]
async fn main() {
    wizard::WizardContext::run().await;
}


