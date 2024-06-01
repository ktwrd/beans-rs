#![feature(error_generic_member_access)]

use const_format::concatcp;
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

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// content to display when showing a message box on panic.
pub const PANIC_MSG_CONTENT: &str = include_str!("text/msgbox_panic_text.txt");
/// name of the mod to use.
/// e.g; `open_fortress`
pub const MOD_NAME: &str = include_str!("../.config/beans/mod_name.txt");
/// two-letter abbreviation that is used in `versions.json` for the game.
/// e.g; `of`
pub const MOD_NAME_SHORT: &str = include_str!("../.config/beans/mod_name_short.txt");
/// base URL for the versioning.
/// e.g; https://beans.adastral.net/
pub const SOURCE_URL: &str = include_str!("../.config/beans/remote_url_base.txt");
/// url where the version details are stored.
/// e.g; https://beans.adastral.net/versions.json
pub const VERSION_URL: &str = include_str!("../.config/beans/version_url.txt");

// ------------------------------------------------------------------------
// please dont change consts below unless you know what you're doing <3
//
// ------------------------------------------------------------------------

pub static mut FORCE_DEBUG: bool = false;

#[cfg(not(target_os = "windows"))]
pub const DATA_DIR: &str = formatcp!("/{}/", MOD_NAME);
#[cfg(target_os = "windows")]
pub const DATA_DIR: &str = formatcp!("\\{}\\", MOD_NAME);
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
