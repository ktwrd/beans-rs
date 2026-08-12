#[cfg(not(target_os = "windows"))]
mod linux;

use std::backtrace::Backtrace;

#[cfg(not(target_os = "windows"))]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;

use std::{collections::HashMap,
          io::Write};

use beans_core::{BeansError,
                 DownloadFailureReason,
                 GITHUB_RELEASES_URL,
                 GameinfoBackupCreateDirectoryFail,
                 GameinfoBackupFailureReason,
                 GameinfoBackupReadContentFail,
                 GameinfoBackupWriteFail,
                 appvar::AppVarData,
                 data_dir,
                 env::get_disable_aria2c,
                 get_user_agent,
                 path::{file_exists,
                        format_directory_path,
                        join_path,
                        parse_location,
                        remove_path_head}};
use futures::StreamExt;
use indicatif::{ProgressBar,
                ProgressStyle};
use log::{debug,
          error,
          trace,
          warn};
use reqwest::header::USER_AGENT;
#[cfg(target_os = "windows")]
pub use windows::*;

use crate::RunnerContext;

#[derive(Clone, Debug)]
pub enum InstallType
{
    /// when steam/sourcemods/open_fortress/ doesn't exist
    NotInstalled,
    /// when steam/sourcemods/open_fortress/.adastral exists
    Adastral,
    /// when either of the following exists;
    /// - steam/sourcemods/open_fortress/.revision
    /// - steam/sourcemods/open_fortress/gameinfo.txt
    OtherSource,
    /// when checking if InstallType equates to OtherSource, if the value is
    /// OtherSourceManual, then it will return true
    ///
    /// set when only steam/sourcemods/open_fortress/gameinfo.txt exists
    OtherSourceManual
}

impl PartialEq for InstallType
{
    fn eq(
        &self,
        other: &Self
    ) -> bool
    {
        match self
        {
            InstallType::NotInstalled => match other
            {
                InstallType::NotInstalled => true,
                InstallType::Adastral => false,
                InstallType::OtherSource => false,
                InstallType::OtherSourceManual => false
            },
            InstallType::Adastral => match other
            {
                InstallType::NotInstalled => false,
                InstallType::Adastral => true,
                InstallType::OtherSource => false,
                InstallType::OtherSourceManual => false
            },
            InstallType::OtherSource => match other
            {
                InstallType::NotInstalled => false,
                InstallType::Adastral => false,
                InstallType::OtherSource => true,
                InstallType::OtherSourceManual => true
            },
            InstallType::OtherSourceManual => match other
            {
                InstallType::NotInstalled => false,
                InstallType::Adastral => false,
                InstallType::OtherSource => false,
                InstallType::OtherSourceManual => true
            }
        }
    }
}

/// get the current type of installation.
pub fn install_state(sourcemods_location: Option<String>) -> InstallType
{
    let mut smp_x = match sourcemods_location
    {
        Some(v) => v,
        None => match find_sourcemod_path()
        {
            Ok(v) => v,
            Err(e) =>
            {
                sentry::capture_error(&e);
                debug!(
                    "[helper::install_state] {} {:#?}",
                    BeansError::SourceModLocationNotFound,
                    e
                );
                return InstallType::NotInstalled;
            }
        }
    };
    if smp_x.ends_with("/") || smp_x.ends_with("\\")
    {
        smp_x.pop();
    }

    let data_dir = join_path(smp_x, data_dir());

    if file_exists(format!("{}.adastral", data_dir))
    {
        return InstallType::Adastral;
    }
    else if file_exists(format!("{}.revision", data_dir))
    {
        return InstallType::OtherSource;
    }
    else if file_exists(format!("{}gameinfo.txt", data_dir))
    {
        return InstallType::OtherSourceManual;
    }
    InstallType::NotInstalled
}

/// get user input from terminal. prompt is displayed on the line above where
/// the user does input.
pub fn get_input(prompt: &str) -> String
{
    println!("{}", prompt);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input)
    {
        Ok(_goes_into_input_above) =>
        {}
        Err(_no_updates_is_fine) =>
        {}
    }
    input.trim().to_string()
}

/// Check if a process is running
///
/// name: Executable name (from `Process.name(&self)`)
/// argument_contains: Check if the arguments of the process has an item that
/// starts with this value (when some).
fn is_process_running(
    name: String,
    arguments_contains: Option<String>
) -> Option<sysinfo::Pid>
{
    find_process(move |proc: &sysinfo::Process| {
        if let Some(proc_name_str) = proc.name().to_str()
        {
            let proc_name = proc_name_str.to_string();
            if proc_name == name
            {
                if let Some(x) = arguments_contains.clone()
                {
                    for item in proc.cmd().iter()
                    {
                        if let Some(item_str) = item.to_str()
                        {
                            if item_str.starts_with(&x)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    })
}
/// Find a process with a selector filter.
///
/// Will return Some when a process is found, otherwise None.
pub fn find_process<TFilterSelector>(selector: TFilterSelector) -> Option<sysinfo::Pid>
where
    TFilterSelector: Fn(&sysinfo::Process) -> bool
{
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    for process in sys.processes().values()
    {
        if selector(process)
        {
            return Some(process.pid());
        }
    }
    None
}

/// Check if there are any processes running
///
/// Will return `true` if any of the cases are matched;
/// - If there are any processes called `hl2.exe` that contain the
///   `mod_directory` provided
/// - If there are any processes that contain the `mod_directory` in the
///   arguments  *that aren't* beans-rs.
///
/// Otherwise, `false` is returned.
pub fn is_game_running(mod_directory: String) -> Option<sysinfo::Pid>
{
    // check if running with the windows things
    if let Some(proc) = is_process_running(String::from("hl2.exe"), Some(mod_directory.clone()))
    {
        return Some(proc);
    }
    if let Some(proc) = is_process_running(
        String::from("hl2.exe"),
        Some(format!("\"{}\"", mod_directory.clone()))
    )
    {
        return Some(proc);
    }
    // check if any process has it in the arguments
    if let Some(proc) = find_process(move |proc| {
        for item in proc.cmd().iter()
        {
            if let Some(os_str) = item.to_str()
            {
                let os_string = os_str.to_string();
                if os_string.starts_with(&mod_directory)
                {
                    if let Some(proc_name_str) = proc.name().to_str()
                    {
                        let proc_name = proc_name_str.to_string().to_lowercase();
                        if proc_name != *"beans" && proc_name != *"beans-rs"
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    })
    {
        return Some(proc);
    }
    None
}

/// Get the amount of free space on the drive in the location provided.
pub fn get_free_space(location: String) -> Result<u64, BeansError>
{
    let mut data: HashMap<String, u64> = HashMap::new();
    for disk in sysinfo::Disks::new_with_refreshed_list().list()
    {
        if let Some(mp) = disk.mount_point().to_str()
        {
            debug!("[get_free_space] space: {} {}", mp, disk.available_space());
            data.insert(mp.to_string(), disk.available_space());
        }
    }

    let mut l = parse_location(location.clone());
    while !l.is_empty()
    {
        debug!("[get_free_space] Checking if {} is in data", l);
        if let Some(x) = data.get(&l)
        {
            return Ok(*x);
        }
        l = remove_path_head(l);
    }

    Err(BeansError::FreeSpaceCheckFailure {
        location: parse_location(location.clone())
    })
}

/// Check if the location provided has enough free space.
pub fn has_free_space(
    location: String,
    size: usize
) -> Result<bool, BeansError>
{
    Ok((size as u64) < get_free_space(location)?)
}

pub async fn download_with_progress(
    url: String,
    out_location: String
) -> Result<(), BeansError>
{
    debug!(
        "[helper::download_with_progress] url: {}, out_location: {}",
        url, out_location
    );
    if crate::aria2::can_use_aria2() && !get_disable_aria2c()
    {
        debug!("[helper::download_with_progress] using aria2c");
        crate::aria2::download_file(url, out_location).await?;
    }
    else
    {
        download_with_progress_reqwest(url, out_location).await?;
    }
    Ok(())
}
/// Download file at the URL provided to the output location provided
/// This function will also show a progress bar with indicatif.
async fn download_with_progress_reqwest(
    url: String,
    out_location: String
) -> Result<(), BeansError>
{
    let res = match reqwest::Client::new().get(&url).send().await
    {
        Ok(v) => v,
        Err(e) =>
        {
            sentry::capture_error(&e);
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::Reqwest {
                    url: url.clone(),
                    error: e
                },
                backtrace: std::backtrace::Backtrace::capture()
            });
        }
    };

    let total_size = res
        .content_length()
        .expect("Failed to get length of data to download");

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", &url));

    // download chunks
    let mut file = match std::fs::File::create(out_location.clone())
    {
        Ok(v) => v,
        Err(e) =>
        {
            sentry::capture_error(&e);
            return Err(BeansError::FileOpenFailure {
                location: out_location,
                error: e
            });
        }
    };
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await
    {
        let chunk = item.expect("Failed to write content to file");
        file.write_all(&chunk)
            .expect("Failed to write content to file");
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish();
    Ok(())
}

/// Format parameter `i` to a human-readable size.
pub fn format_size(i: usize) -> String
{
    let value = i.to_string();

    let decimal_points: usize = 3;
    let mut dec_l = decimal_points * 6;
    if i < 1_000
    {
        dec_l = 0
    }
    else if i < 1_000_000
    {
        dec_l = decimal_points
    }
    else if i < 1_000_000_000
    {
        dec_l = decimal_points * 2;
    }
    else if i < 1_000_000_000_000
    {
        dec_l = decimal_points * 3;
    }
    else if i < 1_000_000_000_000_000
    {
        dec_l = decimal_points * 4;
    }
    else if i < 1_000_000_000_000_000_000
    {
        dec_l = decimal_points * 5;
    }

    let dec: String = value.chars().rev().take(dec_l).collect();

    let mut dec_x: String = dec.chars().rev().take(decimal_points).collect();
    dec_x = dec_x.trim_end_matches('0').to_string();

    let whole_l = value.len() - dec_l;

    let mut whole: String = value.chars().take(whole_l).collect();
    if !dec_x.is_empty()
    {
        whole.push('.');
    }
    let pfx_data: Vec<(usize, &str)> = vec![
        (1_000, "b"),
        (1_000_000, "kb"),
        (1_000_000_000, "mb"),
        (1_000_000_000_000, "gb"),
        (1_000_000_000_000_000, "tb"),
    ];
    for (s, c) in pfx_data.into_iter()
    {
        if i < s
        {
            return format!("{}{}{}", whole, dec_x, c);
        }
    }
    format!("{}{}", whole, dec_x)
}

/// Check if there is an update available. When the latest release doesn't match
/// the current release.
pub async fn beans_has_update() -> Result<Option<GithubReleaseItem>, BeansError>
{
    let user_agent = get_user_agent();
    let rs = reqwest::Client::new()
        .get(GITHUB_RELEASES_URL)
        .header(USER_AGENT, &user_agent)
        .send()
        .await;
    let response = match rs
    {
        Ok(v) => v,
        Err(e) =>
        {
            let message =
                format!("Failed to get latest release from github: {GITHUB_RELEASES_URL:}");
            let err = BeansError::Reqwest {
                error_message: message,
                error: e,
                backtrace: Backtrace::capture()
            };
            trace!("[helper::beans_has_update] {:#?}", err);
            return Err(err);
        }
    };
    let response_text = response.text().await?;
    let data: GithubReleaseItem = match serde_json::from_str(&response_text)
    {
        Ok(v) => v,
        Err(e) =>
        {
            let error = BeansError::SerdeJson {
                error: e,
                content: Some(response_text),
                backtrace: Backtrace::capture()
            };
            trace!(
                "[beans_rs::beans_has_update] Failed to deserialize GithubReleaseItem from URL {GITHUB_RELEASES_URL:}\n{error:#?}"
            );
            return Err(error);
        }
    };
    trace!("[beans_rs::beans_has_update] response data from URL {GITHUB_RELEASES_URL:}\n{data:#?}");
    if !data.draft && !data.prerelease && data.tag_name != format!("v{}", beans_core::VERSION)
    {
        return Ok(Some(data.clone()));
    }
    Ok(None)
}

pub fn restore_gameinfo(
    ctx: &mut RunnerContext,
    data: Vec<u8>
) -> Result<(), BeansError>
{
    let loc = ctx.gameinfo_location();
    trace!("gameinfo location: {}", &loc);
    if let Ok(m) = std::fs::metadata(&loc)
    {
        trace!("gameinfo metadata: {:#?}", m);
    }
    if let Err(e) = ctx.gameinfo_perms()
    {
        error!(
            "[helper::restore_gameinfo] Failed to update permissions on gameinfo.txt {:}",
            e
        );
        sentry::capture_error(&e);
        return Err(e);
    }
    if let Err(e) = std::fs::write(&loc, data)
    {
        trace!("error: {:#?}", e);
        error!(
            "[helper::restore_gameinfo] Failed to write gameinfo.txt backup {:}",
            e
        );
    }
    if let Err(e) = ctx.gameinfo_perms()
    {
        error!(
            "[helper::restore_gameinfo] Failed to update permissions on gameinfo.txt {:}",
            e
        );
        sentry::capture_error(&e);
        return Err(e);
    }
    Ok(())
}

pub fn backup_gameinfo(ctx: &mut RunnerContext) -> Result<(), BeansError>
{
    let av = AppVarData::get();
    let gamedir = join_path(ctx.clone().sourcemod_path, av.mod_info.sourcemod_name);
    let backupdir = join_path(gamedir.clone(), String::from(GAMEINFO_BACKUP_DIRNAME));

    let current_time = chrono::Local::now();
    let current_time_formatted = current_time.format("%Y%m%d-%H%M%S").to_string();

    if !file_exists(backupdir.clone())
    {
        if let Err(e) = std::fs::create_dir(&backupdir)
        {
            debug!("backupdir: {}", backupdir);
            debug!("error: {:#?}", e);
            error!(
                "[helper::backup_gameinfo] Failed to create backup directory {:}",
                e
            );
            return Err(BeansError::GameinfoBackupFailure {
                reason: GameinfoBackupFailureReason::BackupDirectoryCreateFailure(
                    GameinfoBackupCreateDirectoryFail {
                        error: e,
                        location: backupdir
                    }
                )
            });
        }
    }
    let output_location = join_path(
        backupdir,
        format!(
            "{}-{}.txt",
            ctx.current_version.unwrap_or(0),
            current_time_formatted
        )
    );
    let current_location = join_path(gamedir, String::from("gameinfo.txt"));

    if !file_exists(current_location.clone())
    {
        debug!(
            "[helper::backup_gameinfo] can't backup since {} doesn't exist",
            current_location
        );
        return Ok(());
    }

    let content = match std::fs::read_to_string(&current_location)
    {
        Ok(v) => v,
        Err(e) =>
        {
            debug!("location: {}", current_location);
            debug!("error: {:#?}", e);
            error!(
                "[helper::backup_gameinfo] Failed to read content of gameinfo.txt {:}",
                e
            );
            return Err(BeansError::GameinfoBackupFailure {
                reason: GameinfoBackupFailureReason::ReadContentFail(
                    GameinfoBackupReadContentFail {
                        error: e,
                        proposed_location: output_location,
                        current_location: current_location.clone()
                    }
                )
            });
        }
    };

    if file_exists(output_location.clone())
    {
        if let Err(e) = std::fs::remove_file(&output_location)
        {
            warn!(
                "[helper::backup_gameinfo] Failed to delete existing file, lets hope things don't break. {:} {}",
                e,
                output_location.clone()
            );
        }
    }

    if let Err(e) = std::fs::write(&output_location, content)
    {
        debug!("location: {}", output_location);
        debug!("error: {:#?}", e);
        error!(
            "[helper::backup_gameinfo] Failed to write backup to {} ({:})",
            output_location, e
        );
        return Err(BeansError::GameinfoBackupFailure {
            reason: GameinfoBackupFailureReason::WriteFail(GameinfoBackupWriteFail {
                error: e,
                location: output_location
            })
        });
    }

    println!("[backup_gameinfo] Created backup at {}", output_location);

    Ok(())
}

const GAMEINFO_BACKUP_DIRNAME: &str = "gameinfo_backup";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GithubReleaseItem
{
    #[serde(rename = "id")]
    pub _id: u64,
    pub created_at: String,
    pub tag_name: String,
    pub url: String,
    pub html_url: String,
    pub draft: bool,
    pub prerelease: bool
}

/// Return `true` when `try_get_env_var` returns Some with a length greater than
/// `1`.
pub fn has_env_var(target_key: String) -> bool
{
    if let Some(x) = try_get_env_var(target_key)
    {
        return x.len() > 1;
    }
    false
}

/// Try and get a value from `std::env::vars()`
/// Will return `None` when not found
pub fn try_get_env_var(target_key: String) -> Option<String>
{
    for (key, value) in std::env::vars()
    {
        if key == target_key
        {
            return Some(value);
        }
    }
    None
}

/// Get the message used from the info passed to std::panic::set_hook.
pub fn payload_message(info: &std::panic::PanicHookInfo) -> String
{
    if let Some(s) = info.payload().downcast_ref::<&str>()
    {
        String::from(*s)
    }
    else if let Some(s) = info.payload().downcast_ref::<String>()
    {
        s.clone()
    }
    else
    {
        String::from("<unknown error> (unhandled downcast_ref in payload_message)")
    }
}

/// Check if a program exists in the PATH environment variable folders.
pub fn program_in_path(name: String) -> bool
{
    get_program_env_location(name).is_some()
}

/// Get the full executable path for an executable found in the PATH
/// environment variable.
///
/// Derived from https://stackoverflow.com/a/35046243/13037015
pub fn get_program_env_location(name: String) -> Option<String>
{
    if let Ok(path) = std::env::var("PATH")
    {
        for p in path.split(":")
        {
            let mut p_str = format_directory_path(p.to_string());
            p_str.push_str(name.as_str());
            if std::fs::metadata(&p_str).is_ok()
            {
                return Some(p_str);
            }
        }
    }
    None
}
