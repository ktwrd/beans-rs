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
pub mod butler;



#[cfg(not(target_os = "windows"))]
pub const DATA_DIR: &str = "/open_fortress/";
#[cfg(target_os = "windows")]
pub const DATA_DIR: &str = "\\open_fortress\\";
#[cfg(not(target_os = "windows"))]
pub const STAGING_DIR: &str = "/butler-staging";
#[cfg(target_os = "windows")]
pub const STAGING_DIR: &str = "\\butler-staging";

pub const SOURCE_URL: &str = "https://beans.adastral.net/";
pub const UPDATE_HASH_URL_WINDOWS: &str = concatcp!(SOURCE_URL, "beans_sha512sum_windows");
pub const UPDATE_HASH_URL_LINUX: &str = concatcp!(SOURCE_URL, "beans_sha512sum_linux");

#[cfg(target_os = "windows")]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler.exe");
#[cfg(not(target_os = "windows"))]
pub const BUTLER_BINARY: &[u8] = include_bytes!("../Binaries/butler");
#[cfg(target_os = "windows")]
pub const BUTLER_LIB_1: &[u8] = include_bytes!("../Binaries/7z.dll");
#[cfg(not(target_os = "windows"))]
pub const BUTLER_LIB_1: &[u8] = include_bytes!("../Binaries/7z.so");
#[cfg(target_os = "windows")]
pub const BUTLER_LIB_2: &[u8] = include_bytes!("../Binaries/c7zip.dll");
#[cfg(not(target_os = "windows"))]
pub const BUTLER_LIB_2: &[u8] = include_bytes!("../Binaries/libc7zip.so");
