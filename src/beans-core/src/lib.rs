#![feature(error_generic_member_access)]
pub mod appvar;
use crate::appvar::AppVarData;
mod error;
pub use error::*;
pub mod env;
pub mod helper;
pub mod path;
pub mod version;

#[macro_use]
extern crate rust_i18n;
i18n!();

/// NOTE do not change, fetches from the version of beans-rs on build
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// sentry url, change on fork please.
pub const SENTRY_URL: &str = "https://9df80170f0a4411bb9c834ac54734380@sentry.kate.pet/1";
/// github releases url for the repository this is for. used to check for
/// updates
pub const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repositories/805393469/releases/latest";
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
    let av = AppVarData::get();
    format!("{}{}{}", PATH_SEP, av.mod_info.sourcemod_name, PATH_SEP)
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

    if env::get_headless()
    {
        return true;
    }

    match std::env::consts::OS
    {
        "windows" | "macos" => true,
        "linux" =>
        {
            if env::has("DISPLAY".to_string())
            {
                return true;
            }
            if let Some(x) = env::try_get("XDG_SESSION_DESKTOP".to_string())
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
            log::warn!(
                "{}",
                t!("error.bad_gui_platform", platform = std::env::consts::OS)
            );
            false
        }
    }
}

/// User agent for downloading files or sending web requests.
pub fn get_user_agent() -> String
{
    let av = AppVarData::get();
    format!("beans-rs/{:} ({:})", VERSION, av.mod_info.sourcemod_name)
}

pub fn staging_dir() -> String
{
    let av = AppVarData::get();
    #[cfg(not(target_os = "windows"))]
    {
        format!("/butler-staging-{}", av.mod_info.short_name)
    }
    #[cfg(target_os = "windows")]
    {
        format!("\\butler-staging-{}", av.mod_info.short_name)
    }
}
