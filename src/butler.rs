use std::backtrace::Backtrace;
use std::process::ExitStatus;
use log::{debug, error, info, warn};
use crate::{BeansError, depends, DownloadFailureReason, helper};

pub fn verify(
    signature_url: String,
    gamedir: String,
    remote: String
) -> Result<ExitStatus, BeansError> {
    let mut cmd = std::process::Command::new(&depends::get_butler_location());
    cmd.args([
        "verify",
        &signature_url,
        &gamedir,
        format!("--heal=archive,{}", remote).as_str()
    ]);
    debug!("[butler::verify] {:#?}", cmd);
    match cmd
        .spawn() {
        Err(e) => {
            Err(BeansError::ButlerVerifyFailure {
                signature_url,
                gamedir,
                remote,
                error: e,
                backtrace: Backtrace::capture()
            })
        },
        Ok(mut v) => {
            let w = v.wait()?;
            debug!("[butler::verify] Exited with {:#?}", w);
            if let Some(c) = w.code() {
                if c != 0 {
                    error!("[butler::verify] exited with code {c}, which isn't good!");
                    panic!("[butler::verify] exited with code {c}");
                }
            }
            Ok(w)
        }
    }
}
pub async fn patch_dl(
    dl_url: String,
    staging_dir: String,
    patch_filename: String,
    gamedir: String
) -> Result<ExitStatus, BeansError> {
    if helper::file_exists(staging_dir.clone()) {
        std::fs::remove_dir_all(&staging_dir)?;
    }
    let tmp_file = helper::get_tmp_file(patch_filename);
    info!("[butler::patch_dl] downloading {} to {}", dl_url, tmp_file);
    helper::download_with_progress(dl_url, tmp_file.clone()).await?;

    if helper::file_exists(tmp_file.clone()) == false {
        return Err(BeansError::DownloadFailure {
            reason: DownloadFailureReason::FileNotFound {
                location: tmp_file
            }
        });
    }

    patch(tmp_file, staging_dir, gamedir)
}

pub fn patch(
    patchfile_location: String,
    staging_dir: String,
    gamedir: String
) -> Result<ExitStatus, BeansError> {
    let mut cmd = std::process::Command::new(&depends::get_butler_location());
    cmd.args([
        "apply",
        &format!("--staging-dir={}", &staging_dir),
        &patchfile_location,
        &gamedir
    ]);
    debug!("[butler::patch] {:#?}", &cmd);
    match cmd
        .spawn() {
        Err(e) => {
            let xe = BeansError::ButlerPatchFailure {
                patchfile_location,
                gamedir,
                error: e,
                backtrace: Backtrace::capture()
            };
            error!("[butler::patch] {:#?}", xe);
            sentry::capture_error(&xe);
            Err(xe)
        },
        Ok(mut v) => {
            let w = v.wait()?;
            debug!("Exited with {:#?}", w);
            if let Some(c) = w.code() {
                if c != 0 {
                    error!("[butler::patch] exited with code {c}, which isn't good!");
                    panic!("[butler::patch] exited with code {c}");
                }
            } else {
                if helper::file_exists(staging_dir.clone()) {
                    if let Err(e) = std::fs::remove_dir_all(&staging_dir) {
                        warn!("[butler::patch] Failed to delete staging directory after patch {:}", e);
                        debug!("[butler::patch] {} {:#?}", staging_dir, e);
                    }
                }
            }
            Ok(w)
        }
    }
}