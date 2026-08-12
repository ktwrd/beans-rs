use std::path::PathBuf;

use log::{debug,
          trace,
          warn};

use crate::helper::{generate_rand_str,
                    is_steamdeck,
                    use_custom_tmpdir};

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

/// check if a path location exists
pub fn path_exists(path: String) -> bool
{
    std::path::Path::new(&path).exists()
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

/// Get the filename of the location provided.
///
/// If the result is an empty string, then the location provided is invalid, and
/// you should check that yourself :3
pub fn get_filename(location: String) -> String
{
    let x = location.to_string().replace(['/', '\\'], crate::PATH_SEP);
    let xr = x.split(crate::PATH_SEP);
    if let Some(p) = xr.last()
    {
        p.to_string()
    }
    else
    {
        String::new()
    }
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
                            "[core::path::parse_location] Failed to parse location to string {}",
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
                        "[core::path::parse_location] Failed to canonicalize location {}",
                        location
                    );
                    eprintln!("[core::path::parse_location] {:}", e);
                    debug!("{:#?}", e);
                    return location;
                }
            }
        }
        None =>
        {
            debug!(
                "[core::path::parse_location] Failed to parse location {}",
                location
            );
            return location;
        }
    };
    real_location
}

/// Generate a full file location for a temporary file.
pub fn get_tmp_file(filename: String) -> String
{
    let head = format!("{}_{}", generate_rand_str(8), filename);
    join_path(get_tmp_dir(), head)
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
        trace!(
            "[core::path::get_tmp_dir] Detected that we are running on a steam deck. Using ~/.tmp/beans-rs"
        );
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
                    trace!("[core::path::get_tmp_dir] Failed to convert PathBuf to &str");
                }
            },
            None =>
            {
                trace!("[core::path::get_tmp_dir] Failed to get home directory.");
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
            trace!("[core::path::get_tmp_dir] {:#?}", e);
            warn!(
                "[core::path::get_tmp_dir] failed to make tmp directory at {} ({:})",
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
            trace!("[core::path::get_tmp_dir] {:#?}", e);
            warn!(
                "[core::path::get_tmp_dir] failed to make tmp directory at {} ({:})",
                dir, e
            );
            sentry::capture_error(&e);
        }
        else
        {
            trace!("[core::path::get_tmp_dir] created directory {}", dir);
        }
    }

    dir
}
