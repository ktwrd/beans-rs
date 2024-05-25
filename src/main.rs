use beans_rs::wizard;

#[tokio::main]
async fn main() {
    wizard::WizardContext::run().await;
}


