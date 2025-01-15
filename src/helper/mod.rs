#[cfg(not(target_os = "windows"))]
mod linux;

use std::backtrace::Backtrace;

#[cfg(not(target_os = "windows"))]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;

use std::{collections::HashMap,
          io::Write,
          path::PathBuf};

use futures::StreamExt;
use indicatif::{ProgressBar,
                ProgressStyle};
use log::{debug,
          error,
          trace,
          warn};
use rand::{distributions::Alphanumeric,
           Rng};
use reqwest::header::USER_AGENT;
#[cfg(target_os = "windows")]
pub use windows::*;

use crate::{appvar::AppVarData,
            BeansError,
            DownloadFailureReason,
            GameinfoBackupCreateDirectoryFail,
            GameinfoBackupFailureReason,
            GameinfoBackupReadContentFail,
            GameinfoBackupWriteFail,
            RunnerContext};

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

    let data_dir = join_path(smp_x, crate::data_dir());

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

/// check if a file exists
pub fn file_exists(location: String) -> bool
{
    std::path::Path::new(&location).exists()
}

/// Check if the location provided exists and it's a directory.
pub fn dir_exists(location: String) -> bool
{
    file_exists(location.clone()) && is_directory(location.clone())
}

pub fn is_directory(location: String) -> bool
{
    let x = PathBuf::from(&location);
    x.is_dir()
}

/// Check if the file at the location provided is a symlink.
pub fn is_symlink(location: String) -> bool
{
    match std::fs::symlink_metadata(&location)
    {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false
    }
}

pub fn generate_rand_str(length: usize) -> String
{
    let s: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    s.to_uppercase()
}

/// Join the path, using `tail` as the base, and `head` as the thing to add on
/// top of it.
///
/// This will also convert backslashes/forwardslashes to the compiled separator
/// in `crate::PATH_SEP`
pub fn join_path(
    tail: String,
    head: String
) -> String
{
    let mut h = head
        .to_string()
        .replace("/", crate::PATH_SEP)
        .replace("\\", crate::PATH_SEP);
    while h.starts_with(crate::PATH_SEP)
    {
        h.remove(0);
    }

    format!("{}{}", format_directory_path(tail), h)
}

pub fn remove_path_head(location: String) -> String
{
    if let Some(Some(m)) = std::path::Path::new(&location).parent().map(|p| p.to_str())
    {
        return m.to_string();
    }
    String::new()
}

/// Make sure that the location provided is formatted as a directory (ends with
/// `crate::PATH_SEP`).
pub fn format_directory_path(location: String) -> String
{
    let mut x = location.to_string().replace(['/', '\\'], crate::PATH_SEP);

    while x.ends_with(crate::PATH_SEP)
    {
        x.pop();
    }
    if !x.ends_with(crate::PATH_SEP)
    {
        x.push_str(crate::PATH_SEP);
    }
    x
}

#[cfg(not(target_os = "windows"))]
pub fn canonicalize(location: &str) -> Result<PathBuf, std::io::Error>
{
    std::fs::canonicalize(location)
}

#[cfg(target_os = "windows")]
pub fn canonicalize(location: &str) -> Result<PathBuf, std::io::Error>
{
    dunce::canonicalize(location)
}

pub fn parse_location(location: String) -> String
{
    let path = std::path::Path::new(&location);
    let real_location = match path.to_str()
    {
        Some(v) =>
        {
            let p = canonicalize(v);
            match p
            {
                Ok(x) => match x.clone().to_str()
                {
                    Some(m) => m.to_string(),
                    None =>
                    {
                        debug!(
                            "[helper::parse_location] Failed to parse location to string {}",
                            location
                        );
                        return location;
                    }
                },
                Err(e) =>
                {
                    if format!("{:}", e).starts_with("No such file or directory")
                    {
                        return location;
                    }
                    sentry::capture_error(&e);
                    eprintln!(
                        "[helper::parse_location] Failed to canonicalize location {}",
                        location
                    );
                    eprintln!("[helper::parse_location] {:}", e);
                    debug!("{:#?}", e);
                    return location;
                }
            }
        }
        None =>
        {
            debug!(
                "[helper::parse_location] Failed to parse location {}",
                location
            );
            return location;
        }
    };
    real_location
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
        if let Some(proc_name_str) = proc.name().to_str() {
            let proc_name = proc_name_str.to_string();
            if proc_name == name {
                if let Some(x) = arguments_contains.clone()
                {
                    for item in proc.cmd().iter()
                    {
                        if let Some(item_str) = item.to_str() {
                            if item_str.starts_with(&x) {
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
            if let Some(os_str) = item.to_str() {
                let os_string = os_str.to_string();
                if os_string.starts_with(&mod_directory)
                {
                    if let Some(proc_name_str) = proc.name().to_str() {
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

/// Download file at the URL provided to the output location provided
/// This function will also show a progress bar with indicatif.
pub async fn download_with_progress(
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

/// Check if we should use the custom temporary directory, which is stored in
/// the environment variable defined in `CUSTOM_TMPDIR_NAME`.
///
/// ## Return
/// `Some` when the environment variable is set, and the directory exist.
/// Otherwise `None` is returned.
pub fn use_custom_tmpdir() -> Option<String>
{
    if let Some(x) = crate::env_custom_tmpdir()
    {
        let s = x.to_string();
        if dir_exists(s.clone())
        {
            return Some(s);
        }
        else
        {
            warn!(
                "[use_custom_tmp_dir] Custom temporary directory \"{}\" doesn't exist",
                s
            );
        }
    }
    None
}

/// Create directory in temp directory with name of "beans-rs"
pub fn get_tmp_dir() -> String
{
    let mut dir = std::env::temp_dir().to_str().unwrap_or("").to_string();
    if let Some(x) = use_custom_tmpdir()
    {
        dir = x;
    }
    else if is_steamdeck()
    {
        trace!("[helper::get_tmp_dir] Detected that we are running on a steam deck. Using ~/.tmp/beans-rs");
        match simple_home_dir::home_dir()
        {
            Some(v) => match v.to_str()
            {
                Some(k) =>
                {
                    dir = format_directory_path(k.to_string());
                    dir = join_path(dir, String::from(".tmp"));
                }
                None =>
                {
                    trace!("[helper::get_tmp_dir] Failed to convert PathBuf to &str");
                }
            },
            None =>
            {
                trace!("[helper::get_tmp_dir] Failed to get home directory.");
            }
        };
    }
    else if cfg!(target_os = "android")
    {
        dir = String::from("/data/var/tmp");
    }
    else if cfg!(not(target_os = "windows"))
    {
        dir = String::from("/var/tmp");
    }
    dir = format_directory_path(dir);
    if !dir_exists(dir.clone())
    {
        if let Err(e) = std::fs::create_dir(&dir)
        {
            trace!("[helper::get_tmp_dir] {:#?}", e);
            warn!(
                "[helper::get_tmp_dir] failed to make tmp directory at {} ({:})",
                dir, e
            );
        }
    }
    dir = join_path(dir, String::from("beans-rs"));
    dir = format_directory_path(dir);

    if !dir_exists(dir.clone())
    {
        if let Err(e) = std::fs::create_dir(&dir)
        {
            trace!("[helper::get_tmp_dir] {:#?}", e);
            warn!(
                "[helper::get_tmp_dir] failed to make tmp directory at {} ({:})",
                dir, e
            );
            sentry::capture_error(&e);
        }
        else
        {
            trace!("[helper::get_tmp_dir] created directory {}", dir);
        }
    }

    dir
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
            trace!("[helper::is_steamdeck] exit status: {}", &cmd.status);
            let stdout = &cmd.stdout.to_vec();
            let stderr = &cmd.stderr.to_vec();
            if let Ok(x) = String::from_utf8(stderr.clone())
            {
                trace!("[helper::is_steamdeck] stderr: {}", x);
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
            trace!("[helper::is_steamdeck] {:#?}", e);
            warn!("[helper::is_steamdeck] Failed to detect {:}", e);
            false
        }
    }
}

/// Generate a full file location for a temporary file.
pub fn get_tmp_file(filename: String) -> String
{
    let head = format!("{}_{}", generate_rand_str(8), filename);
    join_path(get_tmp_dir(), head)
}

/// Check if there is an update available. When the latest release doesn't match
/// the current release.
pub async fn beans_has_update() -> Result<Option<GithubReleaseItem>, BeansError>
{
    let rs = reqwest::Client::new()
        .get(GITHUB_RELEASES_URL)
        .header(USER_AGENT, &format!("beans-rs/{}", crate::VERSION))
        .send()
        .await;
    let response = match rs
    {
        Ok(v) => v,
        Err(e) =>
        {
            trace!("Failed get latest release from github \nerror: {:#?}", e);
            return Err(BeansError::Reqwest {
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let response_text = response.text().await?;
    let data: GithubReleaseItem = match serde_json::from_str(&response_text)
    {
        Ok(v) => v,
        Err(e) =>
        {
            trace!(
                "Failed to deserialize GithubReleaseItem\nerror: {:#?}\ncontent: {:#?}",
                e,
                response_text
            );
            return Err(BeansError::SerdeJson {
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    trace!("{:#?}", data);
    if !data.draft && !data.prerelease && data.tag_name != format!("v{}", crate::VERSION)
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
            warn!("[helper::backup_gameinfo] Failed to delete existing file, lets hope things don't break. {:} {}", e, output_location.clone());
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
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repositories/805393469/releases/latest";

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
pub fn payload_message(info: &std::panic::PanicHookInfo) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        String::from(*s)
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("<unknown error> (unhandled downcast_ref in payload_message)")
    }
}
