#[allow(dead_code, unused_macros, unused_imports)]


use std::{env, io};
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use winres::WindowsResource;
#[allow(unused_macros)]
macro_rules! print {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}
pub const OVERRIDE_ICON_LOCATION: Option<&'static str> = option_env!("ICON_LOCATION");
pub const RUST_FLAGS: Option<&'static str> = option_env!("RUSTFLAGS");

fn main() {
    windows_icon().expect("Failed to embed icon");
    fltk().expect("Failed to build fltk files");
}

fn fltk() -> Result<(), BuildError> {
    println!("cargo:rerun-if-changed=src/gui/download_ui.fl");
    println!("cargo:rerun-if-changed=src/gui/wizard_ui.fl");
    let g = fl2rust::Generator::default();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    if let Err(e) = g.in_out("src/gui/download_ui.fl", out_path.join("download_ui.rs").to_str().unwrap()) {
        return Err(BuildError::FLTK(format!("Failed to build download_ui.fl {:#?}", e)));
    }
    if let Err(e) = g.in_out("src/gui/wizard_ui.fl", out_path.join("wizard_ui.rs").to_str().unwrap()) {
        return Err(BuildError::FLTK(format!("Failed to build download_ui.fl {:#?}", e)));
    }

    Ok(())
}

#[allow(dead_code)]
fn path_exists(path: String) -> bool
{
    let p = std::path::Path::new(path.as_str());
    return p.exists();
}

#[cfg(target_os = "windows")]
fn windows_icon() -> Result<(), BuildError> {
    let icon_location = OVERRIDE_ICON_LOCATION.unwrap_or("icon.ico");
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        if !path_exists(icon_location.to_string())
        {
            print!("icon.ico not found. Not embedding icon");
            return Ok(());
        }
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon(icon_location)
            .compile()?;
        print!("successfully set icon");
    }
    else
    {
        print!("not on windows, can't embed icon");
    }
    Ok(())
}
#[cfg(not(target_os = "windows"))]
fn windows_icon() -> Result<(), BuildError> {
    Ok(())
}

#[derive(Debug)]
pub enum BuildError {
    IO(io::Error),
    FLTK(String)
}
impl From<io::Error> for BuildError {
    fn from (e: io::Error) -> Self {
        BuildError::IO(e)
    }
}