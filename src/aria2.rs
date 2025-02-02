use std::process::ExitStatus;

use log::{debug,
          error,
          info};

use crate::{depends,
            helper,
            BeansError,
            DownloadFailureReason};

pub fn can_use_aria2() -> bool
{
    get_executable_location().is_some()
}
pub fn get_executable_location() -> Option<String>
{
    if let Some(r) = helper::get_program_env_location(String::from("aria2c"))
    {
        return Some(r);
    }
    if let Some(r) = helper::get_program_env_location(String::from("aria2c.exe"))
    {
        return Some(r);
    }
    if let Some(x) = depends::get_aria2c_location()
        && helper::file_exists(x.clone())
    {
        return Some(x);
    }

    None
}

pub async fn download_file(
    url: String,
    out_location: String
) -> Result<ExitStatus, BeansError>
{
    let exe_location = match get_executable_location()
    {
        Some(x) => x,
        None =>
        {
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::MissingAria2cExecutable,
                backtrace: std::backtrace::Backtrace::capture()
            });
        }
    };
    let mut cmd = std::process::Command::new(exe_location);
    info!("[aria2::download_file] URL: {}", url);
    let output_directory = helper::remove_path_head(out_location.clone());
    let output_filename = helper::get_filename(out_location.clone());
    debug!(
        "[aria2::download_file] output_directory: {}",
        output_directory
    );
    debug!(
        "[aria2::download_file] output_filename: {}",
        output_filename
    );
    cmd.args([
        "-d",
        &output_directory,
        format!("--out={}", output_filename).as_str(),
        "-c",
        format!("--user-agent={}", crate::get_user_agent()).as_str(),
        &url
    ]);
    debug!("[aria2::download_file] spawn\n{:#?}", cmd);
    let cmd_string = format!("{:#?}", cmd);
    match cmd.spawn()
    {
        Err(e) =>
        {
            debug!("[aria2::download_file] failed to spawn process: {:#?}", e);
            error!("[aria2::download_file] Failed to spawn process ({e:})");
            Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::Aria2cSpawnError {
                    url,
                    output_file: out_location,
                    error: e
                },
                backtrace: std::backtrace::Backtrace::capture()
            })
        }
        Ok(mut child) =>
        {
            let wait_status = child.wait()?;
            debug!("[aria2::download_file] exited status {:#?}", wait_status);
            if let Some(code) = wait_status.code()
            {
                if let Some(error_code) = crate::error::Aria2cExitCodeReason::from_exit_code(code)
                {
                    return Err(BeansError::Aria2cExitCode {
                        reason: error_code,
                        cmd: cmd_string,
                        backtrace: std::backtrace::Backtrace::capture()
                    });
                }
            }
            Ok(wait_status)
        }
    }
}
