#[cfg(target_os = "windows")]
use std::backtrace::Backtrace;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_os = "windows")]
use beans_bins::ARIA2C_BINARY;
use beans_bins::{BUTLER_BINARY,
                 BUTLER_LIB_1,
                 BUTLER_LIB_2};
use beans_core::{BeansError,
                 path::{file_exists,
                        format_directory_path,
                        get_tmp_dir,
                        join_path}};
use log::{debug,
          error};

use crate::helper;

/// try and write aria2c and butler if it doesn't exist
/// paths that are used will be fetched from binary_locations()
pub fn try_write_deps()
{
    safe_write_file(get_butler_location().as_str(), &BUTLER_BINARY);
    safe_write_file(get_butler_1_location().as_str(), &BUTLER_LIB_1);
    safe_write_file(get_butler_2_location().as_str(), &BUTLER_LIB_2);
    #[cfg(target_os = "windows")]
    if let Some(s) = get_aria2c_location()
    {
        safe_write_file(s.as_str(), &ARIA2C_BINARY);
    }
    #[cfg(not(target_os = "windows"))]
    if file_exists(get_butler_location())
    {
        let p = std::fs::Permissions::from_mode(0o0744_u32);
        if let Err(e) = std::fs::set_permissions(get_butler_location(), p)
        {
            sentry::capture_error(&e);
            error!(
                "[depends::try_write_deps] {} {}",
                t!("error.permissions.set"),
                get_butler_location()
            );
            error!("[depends::try_write_deps] {:#?}", e);
        }
        debug!(
            "[depends::try_write_deps] {}",
            t!("info.permissions.set", location = get_butler_location())
        );
    }
}
fn safe_write_file(
    location: &str,
    data: &[u8]
)
{
    if !file_exists(location.to_string())
    {
        if let Err(e) = std::fs::write(location, data)
        {
            sentry::capture_error(&e);
            error!(
                "[depends::try_write_deps] {} {}",
                t!("error.extract"),
                location
            );
            error!("[depends::try_write_deps] {:#?}", e);
        }
        else
        {
            debug!(
                "[depends::try_write_deps] {} {}",
                t!("info.extracted"),
                location
            );
        }
    }
}

/// will not do anything since this only runs on windows
#[cfg(not(target_os = "windows"))]
pub async fn try_install_vcredist() -> Result<(), BeansError>
{
    // ignored since we aren't windows :3
    Ok(())
}
/// try to download and install vcredist from microsoft via aria2c
/// TODO use request instead of aria2c for downloading this.
#[cfg(target_os = "windows")]
pub async fn try_install_vcredist() -> Result<(), BeansError>
{
    if !match winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey(String::from(
        "Software\\Microsoft\\VisualStudio\\14.0\\VC\\Runtimes\\x64"
    ))
    {
        Ok(v) =>
        {
            let x: std::io::Result<u32> = v.get_value("Installed");
            x.is_err()
        }
        Err(_) => true
    }
    {
        debug!(
            "[depends::try_install_vcredist] {}",
            t!(
                "dependency.exists",
                item = t!("dependency.software.vcredist")
            )
        );
        return Ok(());
    }

    log::info!(
        "{}",
        t!(
            "dependency.install",
            item = t!("dependency.software.vcredist")
        )
    );
    let mut out_loc = get_fmt_tmp_dir();
    out_loc = join_path(out_loc, "vc_redist.exe".to_string());

    helper::download_with_progress(
        String::from("https://aka.ms/vs/17/release/vc_redist.x86.exe"),
        out_loc.clone()
    )
    .await?;

    if !std::path::Path::new(&out_loc).exists()
    {
        return Err(BeansError::FileNotFound {
            location: out_loc.clone(),
            backtrace: Backtrace::capture()
        });
    }

    std::process::Command::new(&out_loc)
        .args(["/install", "/passive", "/norestart"])
        .spawn()
        .expect(
            "{}",
            t!("error.install", item = t!("dependency.software.vcredist"))
        )
        .wait()?;

    if file_exists(out_loc.clone())
    {
        if let Err(e) = std::fs::remove_file(&out_loc)
        {
            sentry::capture_error(&e);
            debug!(
                "[depends::try_install_vcredist] {} {:#?}", t!("error.remove.installer")
                e
            );
        }
    }

    Ok(())
}

pub fn butler_exists() -> bool
{
    file_exists(get_butler_location())
        && file_exists(get_butler_1_location())
        && file_exists(get_butler_2_location())
}

pub fn get_butler_location() -> String
{
    let mut path = get_fmt_tmp_dir();
    path.push_str(BUTLER_LOCATION);
    path
}
pub fn get_butler_1_location() -> String
{
    let mut path = get_fmt_tmp_dir();
    path.push_str(BUTLER_1);
    path
}
pub fn get_butler_2_location() -> String
{
    let mut path = get_fmt_tmp_dir();
    path.push_str(BUTLER_2);
    path
}
/// Will always return `Some()` on Windows, and `None` on any other platform.
pub fn get_aria2c_location() -> Option<String>
{
    if cfg!(target_os = "windows")
    {
        let mut path = get_fmt_tmp_dir();
        path.push_str(ARIA2C_LOCATION);
        return Some(path);
    }
    None
}
fn get_fmt_tmp_dir() -> String
{
    let path = get_tmp_dir();
    format_directory_path(path)
}

#[cfg(target_os = "windows")]
const BUTLER_LOCATION: &str = "butler.exe";
#[cfg(not(target_os = "windows"))]
const BUTLER_LOCATION: &str = "butler";

#[cfg(target_os = "windows")]
const BUTLER_1: &str = "7z.dll";
#[cfg(not(target_os = "windows"))]
const BUTLER_1: &str = "7z.so";
#[cfg(target_os = "windows")]
const BUTLER_2: &str = "c7zip.dll";
#[cfg(not(target_os = "windows"))]
const BUTLER_2: &str = "libc7zip.so";

const ARIA2C_LOCATION: &str = "aria2c.exe";
