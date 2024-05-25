use crate::{ARIA2C_BINARY, BUTLER_BINARY};

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