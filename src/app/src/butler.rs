use std::{backtrace::Backtrace,
          process::ExitStatus};

use beans_core::{BeansError,
                 DownloadFailureReason,
                 path::{file_exists,
                        get_tmp_file}};
use log::{debug,
          error,
          info};

use crate::{depends,
            helper};

pub fn verify(
    signature_url: String,
    gamedir: String,
    remote: String
) -> Result<ExitStatus, BeansError>
{
    let mut cmd = std::process::Command::new(depends::get_butler_location());
    cmd.args([
        "verify",
        &signature_url,
        &gamedir,
        format!("--heal=archive,{}", remote).as_str()
    ]);
    debug!("[butler::verify] {:#?}", cmd);
    match cmd.spawn()
    {
        Err(e) => Err(BeansError::ButlerVerifyFailure {
            signature_url,
            gamedir,
            remote,
            error: e,
            backtrace: Backtrace::capture()
        }),
        Ok(mut v) =>
        {
            let w = v.wait()?;
            debug!("[butler::verify] {} {:#?}", t!("butler.exit"), w);
            if let Some(c) = w.code()
            {
                if c != 0
                {
                    error!(
                        "[butler::verify] {} {}, {}",
                        t!("error.butler.code"),
                        c,
                        t!("error.butler.panic")
                    );
                    panic!("[butler::verify] {} {}", t!("error.butler.code"), c);
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
) -> Result<ExitStatus, BeansError>
{
    if file_exists(staging_dir.clone())
    {
        std::fs::remove_dir_all(&staging_dir)?;
    }
    let tmp_file = get_tmp_file(patch_filename);
    info!(
        "[butler::patch_dl] {}",
        t!("tasks.download", url = dl_url, file = tmp_file)
    );
    helper::download_with_progress(dl_url, tmp_file.clone()).await?;

    if !file_exists(tmp_file.clone())
    {
        return Err(BeansError::DownloadFailure {
            reason: DownloadFailureReason::FileNotFound {
                location: tmp_file
            },
            backtrace: std::backtrace::Backtrace::capture()
        });
    }

    patch(tmp_file, staging_dir, gamedir)
}

pub fn patch(
    patchfile_location: String,
    staging_dir: String,
    gamedir: String
) -> Result<ExitStatus, BeansError>
{
    let mut cmd = std::process::Command::new(depends::get_butler_location());
    cmd.args([
        "apply",
        &format!("--staging-dir={}", &staging_dir),
        &patchfile_location,
        &gamedir
    ]);
    debug!("[butler::patch] {:#?}", &cmd);
    match cmd.spawn()
    {
        Err(e) =>
        {
            let xe = BeansError::ButlerPatchFailure {
                patchfile_location,
                gamedir,
                error: e,
                backtrace: Backtrace::capture()
            };
            error!("[butler::patch] {:#?}", xe);
            sentry::capture_error(&xe);
            Err(xe)
        }
        Ok(mut v) =>
        {
            let w = v.wait()?;
            debug!("Exited with {:#?}", w);
            if let Some(c) = w.code()
            {
                if c != 0
                {
                    error!(
                        "[butler::patch] {} {}, {}",
                        t!("error.butler.code"),
                        c,
                        t!("error.butler.panic")
                    );
                    panic!("[butler::patch] {} {}", t!("error.butler.code"), c);
                }
            }
            Ok(w)
        }
    }
}
