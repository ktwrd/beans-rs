#[cfg(not(target_os = "windows"))]
mod linux;
#[cfg(not(target_os = "windows"))]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;


use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};
use futures::StreamExt;
use crate::{BeansError, DownloadFailureReason};
use rand::{distributions::Alphanumeric, Rng};

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


/// check if a file exists
pub fn file_exists(location: String) -> bool
{
    std::path::Path::new(&location).exists()
}

/// Check if the file at the location provided is a symlink.
pub fn is_symlink(location: String) -> bool {
    match std::fs::symlink_metadata(&location) {
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
/// Check if the location provided has enough free space.
pub fn has_free_space(location: String, size: usize) -> Result<bool, BeansError>
{
    let space = get_free_space(location)?;
    return Ok((size as u64) < space);
}
/// Check if the sourcemod mod folder has enough free space.
pub fn sml_has_free_space(size: usize) -> Result<bool, BeansError>
{
    match find_sourcemod_path() {
        Some(v) => {
            has_free_space(v, size)
        },
        None => {
            Err(BeansError::SourceModLocationNotFound)
        }
    }
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
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::Reqwest {
                    url: url.clone(),
                    error: e
                }
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

/// Format parameter `i` to a human-readable size.
pub fn format_size(i: usize) -> String {
    let value = i.to_string();

    let decimal_points: usize = 3;
    let mut dec_l = decimal_points * 6;
    if i < 1_000 {
        dec_l = decimal_points * 0;
    } else if i < 1_000_000 {
        dec_l = decimal_points * 1;
    } else if i < 1_000_000_000 {
        dec_l = decimal_points * 2;
    } else if i < 1_000_000_000_000 {
        dec_l = decimal_points * 3;
    } else if i < 1_000_000_000_000_000 {
        dec_l = decimal_points * 4;
    } else if i < 1_000_000_000_000_000_000 {
        dec_l = decimal_points * 5;
    }

    let dec: String = value.chars()
        .into_iter()
        .rev()
        .take(dec_l as usize)
        .collect();

    let mut dec_x: String = dec.chars().into_iter().rev().take(decimal_points).collect();
    dec_x = dec_x.trim_end_matches('0').to_string();

    let whole_l = value.len() - dec_l;

    let mut whole: String = value.chars().into_iter().take(whole_l).collect();
    if dec_x.len() > 0 {
        whole.push('.');
    }
    let pfx_data: Vec<(usize, &str)> = vec![
        (1_000, "b"),
        (1_000_000, "kb"),
        (1_000_000_000, "mb"),
        (1_000_000_000_000, "gb"),
        (1_000_000_000_000_000, "tb")];
    for (s, c) in pfx_data.into_iter() {
        if i < s {
            return format!("{}{}{}", whole, dec_x, c);
        }
    }
    return format!("{}{}", whole, dec_x);
}