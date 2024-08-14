#![feature(error_generic_member_access)]
#![feature(panic_info_message)]
// todo
// https://rust-lang.github.io/rust-clippy/master/index.html#/result_large_err
// https://github.com/ktwrd/beans-rs/pull/30
#![allow(clippy::result_large_err)]

use include_flate::flate;

mod ctx;
pub mod depends;
pub mod helper;
pub mod version;
pub mod wizard;
pub mod workflows;

pub use ctx::*;

mod error;

pub use error::*;

pub mod appvar;
pub mod butler;
pub mod extract;
pub mod flags;
pub mod gui;
pub mod logger;

/// NOTE do not change, fetches from the version of beans-rs on build
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// sentry url, change on fork please.
pub const SENTRY_URL: &str = "https://9df80170f0a4411bb9c834ac54734380@sentry.kate.pet/1";
/// content to display when showing a message box on panic.
pub const PANIC_MSG_CONTENT: &str = include_str!("text/msgbox_panic_text.txt");
/// once everything is done, do we wait for the user to press enter before
/// exiting?
///
/// just like the `pause` thing in batch.
pub static mut PAUSE_ONCE_DONE: bool = false;
/// When `true`, everything that prompts the user for Y/N should use the default
/// option.
pub static mut PROMPT_DO_WHATEVER: bool = false;

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

/// Temporary directory which is specified by `ADASTRAL_TMPDIR`.
///
/// Will return `None` when the environment variable couldn't be found, or it's
/// an empty string.
pub fn env_custom_tmpdir() -> Option<String>
{
    let s = helper::try_get_env_var(String::from("ADASTRAL_TMPDIR"));
    match s
    {
        Some(x) => match x.trim().is_empty()
        {
            true => None,
            false => Some(x)
        },
        None => s
    }
}
/// Return `true` when the environment variable `BEANS_DEBUG` or
/// `ADASTRAL_DEBUG` exists and equals `1` or `true`.
pub fn env_debug() -> bool
{
    check_env_bool("BEANS_DEBUG") || check_env_bool("ADASTRAL_DEBUG")
}
/// Return `true` when the environment variable `BEANS_HEADLESS` or
/// `ADASTRAL_HEADLESS` exists and equals `1` or `true`.
pub fn env_headless() -> bool
{
    check_env_bool("BEANS_HEADLESS") || check_env_bool("ADASTRAL_HEADLESS")
}

/// Return `true` when the environment variable exists, and it's value equals
/// `1` or `true (when trimmed and made lowercase).
fn check_env_bool<K: AsRef<std::ffi::OsStr>>(key: K) -> bool
{
    std::env::var(key).is_ok_and(|x| {
        let y = x.trim().to_lowercase();
        y == "1" || y == "true"
    })
}

/// Check if we have GUI support enabled. Will always return `false` when
/// `PAUSE_ONCE_DONE` is `false`.
///
/// Will return `true` when
/// - Running on Windows
/// - Running on macOS
/// - Running on Linux AND the `DISPLAY` or `XDG_SESSION_DESKTOP` environment
///   variables are set.
pub fn has_gui_support() -> bool
{
    unsafe {
        if !PAUSE_ONCE_DONE
        {
            return false;
        }
    }

    if env_headless()
    {
        return true;
    }

    match std::env::consts::OS
    {
        "windows" | "macos" => true,
        "linux" =>
        {
            if helper::has_env_var("DISPLAY".to_string())
            {
                return true;
            }
            if let Some(x) = helper::try_get_env_var("XDG_SESSION_DESKTOP".to_string())
            {
                if x.len() >= 3usize
                {
                    return true;
                }
            }
            false
        }
        _ =>
        {
            log::warn!("Unsupported platform for GUI {}", std::env::consts::OS);
            false
        }
    }
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
