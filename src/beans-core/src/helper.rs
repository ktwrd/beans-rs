use log::{trace,
          warn};
use rand::{RngExt,
           distr::Alphanumeric};

use crate::{env::get_custom_tmpdir,
            path::dir_exists};

pub fn generate_rand_str(length: usize) -> String
{
    let s: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    s.to_uppercase()
}

/// Check if we should use the custom temporary directory, which is stored in
/// the environment variable defined in `CUSTOM_TMPDIR_NAME`.
///
/// ## Return
/// `Some` when the environment variable is set, and the directory exist.
/// Otherwise `None` is returned.
pub fn use_custom_tmpdir() -> Option<String>
{
    if let Some(x) = get_custom_tmpdir()
    {
        let s = x.to_string();
        if dir_exists(s.clone())
        {
            return Some(s);
        }
        else
        {
            warn!(
                "[core::helper::use_custom_tmp_dir] Custom temporary directory \"{}\" doesn't exist",
                s
            );
        }
    }
    None
}

/// Check if the content of `uname -r` contains `valve` (Linux Only)
///
/// ## Returns
/// - `true` when;
///   - The output of `uname -r` contains `valve`
/// - `false` when;
///   - `target_os` is not `linux`
///   - Failed to run `uname -r`
///   - Failed to parse the stdout of `uname -r` as a String.
///
/// ## Note
/// Will always return `false` when `cfg!(not(target_os = "linux"))`.
///
/// This function will write to `log::trace` with the full error details before
/// writing it to `log::warn` or `log::error`. Since errors from this
/// aren't significant, `sentry::capture_error` will not be called.
pub fn is_steamdeck() -> bool
{
    if cfg!(not(target_os = "linux"))
    {
        return false;
    }

    match std::process::Command::new("uname").arg("-r").output()
    {
        Ok(cmd) =>
        {
            trace!("[core::helper::is_steamdeck] exit status: {}", &cmd.status);
            let stdout = &cmd.stdout.to_vec();
            let stderr = &cmd.stderr.to_vec();
            if let Ok(x) = String::from_utf8(stderr.clone())
            {
                trace!("[core::helper::is_steamdeck] stderr: {}", x);
            }
            match String::from_utf8(stdout.clone())
            {
                Ok(x) =>
                {
                    trace!("[helper::is_steamdeck] stdout: {}", x);
                    x.contains("valve")
                }
                Err(e) =>
                {
                    trace!("[helper::is_steamdeck] Failed to parse as utf8 {:#?}", e);
                    false
                }
            }
        }
        Err(e) =>
        {
            trace!("[core::helper::is_steamdeck] {:#?}", e);
            warn!("[core::helper::is_steamdeck] Failed to detect {:}", e);
            false
        }
    }
}
