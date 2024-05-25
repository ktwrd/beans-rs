#![feature(error_generic_member_access)]

use const_format::concatcp;

pub mod depends;
pub mod helper;
pub mod wizard;
pub mod version;
pub mod workflows;
mod ctx;
pub use ctx::*;
mod error;
pub use error::*;



#[cfg(not(windows))]
pub const DATA_DIR: &str = "/open_fortress/";
#[cfg(windows)]
pub const DATA_DIR: &str = "\\open_fortress\\";
#[cfg(not(windows))]
pub const STAGING_DIR: &str = "/butler-staging";
#[cfg(windows)]
pub const STAGING_DIR: &str = "\\butler-staging";

pub const SOURCE_URL: &str = "https://beans.adastral.net/";
pub const UPDATE_HASH_URL_WINDOWS: &str = concatcp!(SOURCE_URL, "beans_sha512sum_windows");
pub const UPDATE_HASH_URL_LINUX: &str = concatcp!(SOURCE_URL, "beans_sha512sum_linux");
#[cfg(windows)]
pub const ARIA2C_BINARY: &[u8] = include_bytes!("../Binaries/aria2c.exe");
#[cfg(not(windows))]
pub const ARIA2C_BINARY: &[u8] = include_bytes!("../Binaries/aria2c");

#[cfg(windows)]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler.exe");
#[cfg(not(windows))]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler");
