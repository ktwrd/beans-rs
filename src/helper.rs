use std::fs::read_to_string;
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};
use futures::StreamExt;
use crate::{ARIA2C_BINARY, BUTLER_BINARY};
use crate::wizard::BeansError;

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

impl PartialEq for InstallType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            InstallType::NotInstalled => {
                match other {
                    InstallType::NotInstalled => true,
                    InstallType::Adastral => false,
                    InstallType::OtherSource => false,
                    InstallType::OtherSourceManual => false
                }
            },
            InstallType::Adastral => {
                match other {
                    InstallType::NotInstalled => false,
                    InstallType::Adastral => true,
                    InstallType::OtherSource => false,
                    InstallType::OtherSourceManual => false
                }
            },
            InstallType::OtherSource => {
                match other {
                    InstallType::NotInstalled => false,
                    InstallType::Adastral => false,
                    InstallType::OtherSource => true,
                    InstallType::OtherSourceManual => true
                }
            },
            InstallType::OtherSourceManual => {
                match other {
                    InstallType::NotInstalled => false,
                    InstallType::Adastral => false,
                    InstallType::OtherSource => false,
                    InstallType::OtherSourceManual => true
                }
            }
        }
    }
}

/// get the current type of installation.
pub fn install_state() -> InstallType
{
    let smp = find_sourcemod_path();
    if smp.is_none() {
        return InstallType::NotInstalled;
    }
    let mut smp_x = smp.unwrap();
    if smp_x.ends_with("/") || smp_x.ends_with("\\") {
        smp_x.pop();
    }

    if file_exists(format!("{}{}.adastral", &smp_x, crate::DATA_DIR)) {
        return InstallType::Adastral;
    }
    else if file_exists(format!("{}{}.revision", &smp_x, crate::DATA_DIR)) {
        return InstallType::OtherSource;
    }
    else if file_exists(format!("{}{}gameinfo.txt", &smp_x, crate::DATA_DIR)) {
        return InstallType::OtherSourceManual;
    }
    return InstallType::NotInstalled;
}

/// get user input from terminal. prompt is displayed on the line above where the user does input.
pub fn get_input(prompt: &str) -> String{
    println!("{}",prompt);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {},
        Err(_no_updates_is_fine) => {},
    }
    input.trim().to_string()
}

/// all possible known directory where steam *might* be
/// only is used on linux, since windows will use the registry.
#[cfg(not(linux))]
pub const STEAM_POSSIBLE_DIR:  &'static [&'static str] = &[
    "~/.steam/registry.vdf",
    "~/.var/app/com.valvesoftware.Steam/.steam/registry.vdf"
];

/// find sourcemod path on linux.
/// fetches the fake registry that steam uses from find_steam_reg_path
/// and gets the value of Registry/HKCU/Software/Valve/Steam/SourceModInstallPath
#[cfg(not(linux))]
pub fn find_sourcemod_path() -> Option<String>
{
    let reg_path = match find_steam_reg_path()
    {
        Some(v) => v,
        None => {return None;}
    };

    let reg_content = match read_to_string(reg_path.as_str())
    {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to open file {}\n\n{:#?}", reg_path, e);
        }
    };

    for line in reg_content.lines() {
        if line.contains("SourceModInstallPath")
        {
            let split = &line.split("\"SourceModInstallPath\"");
            let mut last = split.clone()
                .last()
                .expect("Failed to find SourceModInstallPath")
                .trim()
                .replace("\\\\", "/")
                .replace("\\", "/")
                .replace("\"", "");
            if last.ends_with("/") == false {
                last.push_str("/");
            }
            return Some(last);
        }
    }

    return None;
}
/// returns the first item in STEAM_POSSIBLE_DIR that exists. otherwise None
#[cfg(not(linux))]
fn find_steam_reg_path() -> Option<String>
{
    for x in STEAM_POSSIBLE_DIR.into_iter() {
        let mut h = simple_home_dir::home_dir().expect("Failed to get home directory").to_str().expect("Failed to get home directory (as &str)").to_string();
        if h.ends_with("/") {
            h.pop();
        }
        let reg_loc = x.replace("~", h.as_str());
        if file_exists(reg_loc.clone())
        {
            return Some(reg_loc);
        }
    }
    return None;
}
/// TODO use windows registry to get the SourceModInstallPath
/// HKEY_CURRENT_COMPUTER\Software\Value\Steam
/// Key: SourceModInstallPath
#[cfg(windows)]
pub fn find_sourcemod_path()
{
    todo!();
}

/// check if a file exists
pub fn file_exists(location: String) -> bool
{
    std::path::Path::new(&location).exists()
}
use rand::{distributions::Alphanumeric, Rng};
pub fn generate_rand_str(length: usize) -> String
{
    let s: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    s.to_uppercase()
}

/// Get the amount of free space on the drive in the location provided.
pub fn get_free_space(location: String) -> Result<u64, BeansError>
{
    for disk in sysinfo::Disks::new_with_refreshed_list().list() {
        if let Some(mp) = disk.mount_point().to_str() {
            if location.starts_with(&mp) {
                return Ok(disk.available_space())
            }
        }
    }

    Err(BeansError::FreeSpaceCheckFailure(location))
}

/// Download file at the URL provided to the output location provided
/// This function will also show a progress bar with indicatif.
pub async fn download_with_progress(url: String, out_location: String) -> Result<(), BeansError>
{
    let res = match reqwest::Client::new()
        .get(&url)
        .send()
        .await {
        Ok(v) => v,
        Err(e) => {
            return Err(BeansError::DownloadFailure(url, e));
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
    let mut file = match std::fs::File::create(out_location.clone()) {
        Ok(v) => v,
        Err(e) => {
            return Err(BeansError::FileOpenFailure(out_location, e));
        }
    };
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
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

/// try and write aria2c and butler if it doesn't exist
/// paths that are used will be fetched from binary_locations()
pub fn try_write_deps()
{
    let (a2c_path, b_path) = binary_locations();
    let (a2c_exists, b_exists) = binaries_exist();

    if !a2c_exists
    {
        if let Err(e) = std::fs::write(&a2c_path, ARIA2C_BINARY) {
            eprintln!("[try_write_deps] Failed to extract aria2c to {}\n{:#?}", a2c_path, e);
        }
        else {
            println!("[try_write_deps] extracted aria2c");
        }
    }
    if !b_exists
    {
        if let Err(e) = std::fs::write(&b_path, BUTLER_BINARY) {
            eprintln!("[try_write_deps] Failed to extract butler to {}\n{:#?}", b_path, e);
        }
        else {
            println!("[try_write_deps] extracted butler");
        }
    }
}

/// will not do anything since this only runs on windows
#[cfg(not(windows))]
pub fn try_install_vcredist()
{
    // ignored since we aren't windows :3
}
/// try to download and install vcredist from microsoft via aria2c
/// TODO use request instead of aria2c for downloading this.
#[cfg(windows)]
pub fn try_install_vcredist()
{
    let (a2c_path, _) = binary_locations();
    let tempdir = std::env::temp_dir().to_str().unwrap_or("").to_string();
    std::process::Command::new(&a2c_path)
        .args(["https://aka.ms/vs/17/release/vc_redist.x86.exe",
            "--check-certificate=false",
            "-d",
            &tempdir])
        .output()
        .expect("Failed to install vcredist");

    let mut out_loc = tempdir.clone();
    if out_loc.ends_with("\\") == false {
        out_loc.push_str("\\");
    }
    out_loc.push_str("vc_redist.x86.exe");

    if std::path::Path::new(&out_loc).exists() == false {
        panic!("Couldn't find {}", &out_loc);
    }

    std::process::Command::new(&out_loc)
        .output()
        .expect("Failed to install vsredist!");
}

/// (aria2c_exists, butler_exists)
pub fn binaries_exist() -> (bool, bool)
{
    let (aria2c, butler) = binary_locations();
    (std::path::Path::new(&aria2c).exists(), std::path::Path::new(&butler).exists())
}

/// (aria2c, butler)
#[cfg(windows)]
pub fn binary_locations() -> (String, String)
{
    (String::from("Binaries/aria2c.exe"), String::from("Binaries/butler.exe"))
}
/// (aria2c, butler)
#[cfg(not(windows))]
pub fn binary_locations() -> (String, String)
{
    (String::from("Binaries/aria2c"), String::from("Binaries/butler"))
}