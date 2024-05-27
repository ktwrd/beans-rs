#[allow(dead_code, unused_macros, unused_imports)]


use std::{env, io};
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
#[cfg(target_os = "windows")]
fn main() -> io::Result<()> {
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
fn main() -> io::Result<()> {
    Ok(())
}
#[allow(dead_code)]
fn path_exists(path: String) -> bool
{
    let p = std::path::Path::new(path.as_str());
    return p.exists();
}