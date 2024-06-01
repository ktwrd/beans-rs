use std::backtrace::Backtrace;
use crate::{BeansError, depends, DownloadFailureReason, helper};

pub fn verify(
    signature_url: String,
    gamedir: String,
    remote: String
) -> Result<(), BeansError> {
    match std::process::Command::new(&depends::get_butler_location())
        .args([
            "verify",
            &signature_url,
            &gamedir,
            format!("--heal=archive,{}", remote).as_str()
        ])
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
            v.wait()?;
            Ok(())
        }
    }
}
pub async fn patch_dl(
    dl_url: String,
    staging_dir: String,
    patch_filename: String,
    gamedir: String
) -> Result<(), BeansError> {
    if helper::file_exists(staging_dir.clone()) {
        std::fs::remove_dir_all(&staging_dir)?;
    }
    let tmp_file = helper::get_tmp_file(patch_filename);
    println!("[butler::patch_dl] downloading {} to {}", dl_url, tmp_file);
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
) -> Result<(), BeansError> {
    println!("[butler::patch] patching directory {} with {}", gamedir, patchfile_location);
    match std::process::Command::new(&depends::get_butler_location())
        .args([
            "apply",
            &format!("--staging-dir={}", staging_dir),
            &patchfile_location,
            &gamedir
        ])
        .spawn() {
        Err(e) => {
            let xe = BeansError::ButlerPatchFailure {
                patchfile_location,
                gamedir,
                error: e,
                backtrace: Backtrace::capture()
            };
            sentry::capture_error(&xe);
            Err(xe)
        },
        Ok(mut v) => {
            v.wait()?;
            Ok(())
        }
    }
}