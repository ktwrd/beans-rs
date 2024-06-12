#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::backtrace::Backtrace;
use crate::{BeansError, BUTLER_BINARY, helper, butler};
#[allow(unused_imports)]
use log::{debug, error, info, trace};

/// try and write aria2c and butler if it doesn't exist
/// paths that are used will be fetched from binary_locations()
pub fn try_write_deps()
{
    safe_write_file(get_butler_location().as_str(), &**BUTLER_BINARY);
    #[cfg(not(target_os = "windows"))]
    if helper::file_exists(get_butler_location()) {
        let p = std::fs::Permissions::from_mode(0744 as u32);
        if let Err(e) = std::fs::set_permissions(&get_butler_location(), p) {
            sentry::capture_error(&e);
            error!("[depends::try_write_deps] Failed to set permissions for {}", get_butler_location());
            error!("[depends::try_write_deps] {:#?}", e);
        }
        debug!("[depends::try_write_deps] set perms on {}", get_butler_location());
    }

    if let Err(e) = butler::fetch_7z_libs() {
        error!("[depends::try_write_deps] Failed to fetch 7z libaries");
        error!("[depends::try_write_deps] {:#?}", e);
    }
}
fn safe_write_file(location: &str, data: &[u8]) {
    if !helper::file_exists(location.to_string())
    {
        if let Err(e) = std::fs::write(&location, data) {
            sentry::capture_error(&e);
            error!("[depends::try_write_deps] failed to extract {}", location);
            error!("[depends::try_write_deps] {:#?}", e);
        }
        else
        {
            debug!("[depends::try_write_deps] extracted {}", location);
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
    if !match winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey(String::from("Software\\Microsoft\\VisualStudio\\14.0\\VC\\Runtimes\\x64")) {
        Ok(v) => {
            let x: std::io::Result<u32> = v.get_value("Installed");
            match x {
                Ok(_) => false,
                Err(_) => true
            }
        },
        Err(_) => true
    } {
        debug!("[depends::try_install_vcredist] Seems like vcredist is already installed");
        return Ok(());
    }

    info!("Installing Visual C++ Redistributable");
    let mut out_loc = helper::get_tmp_dir();
    out_loc = helper::join_path(out_loc, "vc_redist.exe".to_string());
    trace!("[try_install_vcredist] downloading to {out_loc}");
    crate::download::with_progress(
        String::from("https://aka.ms/vs/17/release/vc_redist.x86.exe"),
        out_loc.clone(),
        "Downloading Visual C++ Redistributable".to_string()).await?;

    if std::path::Path::new(&out_loc).exists() == false {
        return  Err(BeansError::FileNotFound {
            location: out_loc.clone(),
            backtrace: Backtrace::capture()
        });
    }

    std::process::Command::new(&out_loc)
        .args(["/install","/passive","/norestart"])
        .spawn()
        .expect("Failed to install vsredist!")
        .wait()?;
    
    if helper::file_exists(out_loc.clone()) {
        if let Err(e) = std::fs::remove_file(&out_loc) {
            sentry::capture_error(&e);
            debug!("[depends::try_install_vcredist] Failed to remove installer {:#?}", e);
        }
    }
    
    Ok(())
}

pub fn butler_exists() -> bool {
    helper::file_exists(get_butler_location())
}

pub fn get_butler_location() -> String
{
    let mut path = helper::get_tmp_dir();
    path.push_str(BUTLER_LOCATION);
    path
}

#[cfg(target_os = "windows")]
const BUTLER_LOCATION: &str = "butler.exe";
#[cfg(not(target_os = "windows"))]
const BUTLER_LOCATION: &str = "butler";
