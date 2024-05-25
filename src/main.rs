use const_format::concatcp;

pub mod wizard;
pub mod helper;
mod version;

#[cfg(not(windows))]
pub const DATA_DIR: &str = "/open_fortress/";
#[cfg(windows)]
pub const DATA_DIR: &str = "\\open_fortress\\";

pub const SOURCE_URL: &str = "https://beans.adastral.net/";
pub const UPDATE_HASH_URL_WINDOWS: &str = concatcp!(SOURCE_URL, "beans_sha512sum_windows");
pub const UPDATE_HASH_URL_LINUX: &str = concatcp!(SOURCE_URL, "beans_sha512sum_linux");
pub mod depends;

#[tokio::main]
async fn main() {
    wizard::WizardContext::run().await;
}



#[cfg(windows)]
pub const ARIA2C_BINARY: &[u8] = include_bytes!("../Binaries/aria2c.exe");
#[cfg(not(windows))]
pub const ARIA2C_BINARY: &[u8] = include_bytes!("../Binaries/aria2c");

#[cfg(windows)]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler.exe");
#[cfg(not(windows))]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler");