#![feature(error_generic_member_access)]

use include_flate::flate;

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
pub mod flags;
pub mod appvar;
pub mod logger;
pub mod download;

/// NOTE do not change, fetches from the version of beans-rs on build
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// sentry url, change on fork please.
pub const SENTRY_URL: &str = "https://e6e260fa3408f2cf934b4b3c4be71c74@sentry.kate.pet/2";
/// content to display when showing a message box on panic.
pub const PANIC_MSG_CONTENT: &str = include_str!("text/msgbox_panic_text.txt");
/// when `true`, gui elements will be used.
pub static mut HEADLESS: bool = true;


// ------------------------------------------------------------------------
// please dont change consts below unless you know what you're doing <3
//
// ------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
pub const PATH_SEP: &str = "/";
#[cfg(target_os = "windows")]
pub const PATH_SEP: &str = "\\";

pub fn data_dir() -> String
{
    let av = appvar::parse();
    format!("{}{}{}", PATH_SEP, av.mod_info.sourcemod_name, PATH_SEP)
}

#[cfg(not(target_os = "windows"))]
pub const STAGING_DIR: &str = "/butler-staging";
#[cfg(target_os = "windows")]
pub const STAGING_DIR: &str = "\\butler-staging";

#[cfg(target_os = "windows")]
flate!(pub static BUTLER_BINARY: [u8] from "Binaries/butler.exe");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_BINARY: [u8] from "Binaries/butler");
#[cfg(target_os = "windows")]
flate!(pub static BUTLER_LIB_1: [u8] from "Binaries/7z.dll");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_LIB_1: [u8] from "Binaries/7z.so");
#[cfg(target_os = "windows")]
flate!(pub static BUTLER_LIB_2: [u8] from "Binaries/c7zip.dll");
#[cfg(not(target_os = "windows"))]
flate!(pub static BUTLER_LIB_2: [u8] from "Binaries/libc7zip.so");
