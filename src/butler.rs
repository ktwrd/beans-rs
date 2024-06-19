use std::backtrace::Backtrace;
use std::io::BufReader;
use std::process::ExitStatus;
use log::{debug, error, info, trace, warn};
use crate::{BeansError, depends, DownloadFailureReason, helper};

pub fn verify(
    signature_url: String,
    gamedir: String,
    remote: String
) -> Result<ExitStatus, BeansError> {
    verify_with_events(signature_url, gamedir, remote, move |_| {})
}
pub fn verify_with_events<EF>(
    signature_url: String,
    gamedir: String,
    remote: String,
    event_callback: EF
) -> Result<ExitStatus, BeansError>
    where EF: Fn(ButlerMessage) + Send + 'static {
    let mut cmd = std::process::Command::new(&depends::get_butler_location());
    cmd.args([
        "verify",
        "--json",
        &signature_url,
        &gamedir,
        format!("--heal=archive,{}", remote).as_str()
    ]);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    debug!("[butler::verify_with_events] {:#?}", cmd);
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
            if let Some(x) = v.stdout.take() {
                let m = helper::tee_hook(std::io::BufReader::new(x), std::io::stdout(), move |line|
                    {
                        debug!("[butler::verify_with_events] received line!\n{line}");
                        let xe: serde_json::error::Result<ButlerMessage> = serde_json::from_str(&line);
                        match xe
                        {
                            Ok(d) => {
                                trace!("[butler::verify_with_events] {:#?}", d);
                                event_callback(d.clone());
                            },
                            Err(e) => {
                                trace!("{:#?}", e);
                                warn!("[butler::verify_with_events] Failed to deserialize output: {:}\n{}", e, &line)
                            }
                        }
                    });
                if let Err(e) = m.join() {
                    error!("[butler::verify_with_events] Failed to hook stdout {:#?}", e);
                }
            }
            else
            {
                warn!("[butler::verify_with_events] failed to pipe stdout :/");
            }
            if let Some(x) = v.stderr.take() {
                let m = helper::tee(BufReader::new(x), std::io::stderr());
                if let Err(e) = m.join() {
                    warn!("[butler::verify_with_events] Failed to hook stderr {:#?}", e);
                }
            }
            else
            {
                debug!("[butler::verify_with_events] failed to pipe stderr");
            }
            let w = v.wait()?;
            debug!("[butler::verify_with_events] Exited with {:#?}", w);
            if let Some(c) = w.code() {
                if c != 0 {
                    error!("[butler::verify_with_events] exited with code {c}, which isn't good!");
                    panic!("[butler::verify_with_events] exited with code {c}");
                }
            }
            event_callback(ButlerMessage::Done {
                exit_status: w.code()
            });
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
    patch_with_events(patchfile_location, staging_dir, gamedir, |_| {})
}
pub fn patch_with_events<EF>(
    patchfile_location: String,
    staging_dir: String,
    gamedir: String,
    event_callback: EF
) -> Result<ExitStatus, BeansError>
where EF: Fn(ButlerMessage) + Send + 'static {
    let mut cmd = std::process::Command::new(&depends::get_butler_location());
    cmd.args([
        "apply",
        "--json",
        &format!("--staging-dir={}", &staging_dir),
        &patchfile_location,
        &gamedir
    ]);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    debug!("[butler::patch_with_events] {:#?}", &cmd);
    match cmd
        .spawn() {
        Err(e) => {
            let xe = BeansError::ButlerPatchFailure {
                patchfile_location,
                gamedir,
                error: e,
                backtrace: Backtrace::capture()
            };
            error!("[butler::patch_with_events] {:#?}", xe);
            sentry::capture_error(&xe);
            Err(xe)
        },
        Ok(mut v) => {
            if let Some(x) = v.stderr.take() {
                let m = helper::tee_hook(std::io::BufReader::new(x), std::io::stderr(), move |line|
                {
                    debug!("[butler::patch_with_events] received line!\n{line}");
                    let xe: serde_json::error::Result<ButlerMessage> = serde_json::from_str(&line);
                    match xe
                    {
                        Ok(d) => {
                            trace!("[butler::patch_with_events] {:#?}", d);
                            event_callback(d.clone());
                        },
                        Err(e) => {
                            trace!("{:#?}", e);
                            warn!("[butler::patch_with_events] Failed to deserialize output: {:}\n{}", e, &line)
                        }
                    }
                });
                if let Err(e) = m.join() {
                    error!("[butler::patch_with_events] Failed to hook stderr {:#?}", e);
                }
            }
            else
            {
                warn!("[butler::patch_with_events] failed to pipe stderr :/");
            }
            if let Some(x) = v.stderr.take() {
                let m = helper::tee(BufReader::new(x), std::io::stderr());
                if let Err(e) = m.join() {
                    warn!("[butler::patch_with_events] Failed to hook stderr {:#?}", e);
                }
            }
            else
            {
                debug!("[butler::patch_with_events] failed to pipe stderr");
            }
            let w = v.wait()?;
            debug!("Exited with {:#?}", w);
            if let Some(c) = w.code() {
                if c != 0 {
                    error!("[butler::patch_with_events] exited with code {c}, which isn't good!");
                    panic!("[butler::patch_with_events] exited with code {c}");
                }
            }
            event_callback(ButlerMessage::Done {
                exit_status: w.code()
            });
            Ok(w)
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ButlerMessage
{
    #[serde(rename = "log")]
    Log {
        level: String,
        message: String,
        time: u64
    },
    #[serde(rename = "progress")]
    Progress {
        #[serde(rename = "bps")]
        speed_bps: f64,
        eta: f64,
        progress: f64,
        time: u64
    },
    Done {
        exit_status: Option<i32>
    }
}