use crate::{env,
            path::format_directory_path};

/// Temporary directory which is specified by `ADASTRAL_TMPDIR`.
///
/// Will return `None` when the environment variable couldn't be found, or it's
/// an empty string.
pub fn get_custom_tmpdir() -> Option<String>
{
    let s = env::try_get(String::from("ADASTRAL_TMPDIR"));
    match s
    {
        Some(x) => match x.trim().is_empty()
        {
            true => None,
            false => Some(x)
        },
        None => s
    }
}
/// Return `true` when the environment variable `BEANS_DEBUG` or
/// `ADASTRAL_DEBUG` exists and equals `1` or `true`.
pub fn get_debug() -> bool
{
    check_bool("BEANS_DEBUG") || check_bool("ADASTRAL_DEBUG")
}
/// Return `true` when the environment variable `BEANS_HEADLESS` or
/// `ADASTRAL_HEADLESS` exists and equals `1` or `true`.
pub fn get_headless() -> bool
{
    check_bool("BEANS_HEADLESS") || check_bool("ADASTRAL_HEADLESS")
}

/// Return `true` when the environment variable `BEANS_DISABLE_ARIA2C` or
/// `ADASTRAL_DISABLE_ARIA2C` exists and equals `1` or `true`.
pub fn get_disable_aria2c() -> bool
{
    check_bool("BEANS_DISABLE_ARIA2C") || check_bool("ADASTRAL_DISABLE_ARIA2C")
}

/// Will return the content of either of the following environment variables
/// if they exist and there is at least 1 character in it;
/// - `BEANS_ARIA2C_ARGS_OVERRIDE`
/// - `ADASTRAL_ARIA2C_ARGS_OVERRIDE`
///
/// This string will be used as the launch arguments for aria2c. It'll replace
/// the following content with;
///
/// | Look For | Replace With |
/// | -------- | ------------ |
/// | `%OUT_DIR%` | Output directory (argument `-d` with aria2c) |
/// | `%OUT_FILENAME%` | Output Filename (argument `--out=` with aria2c) |
/// | `%USER_AGENT%` | Used for the `--user-agent=` aria2c argument|
/// | `%URL%` | URL to download from
pub fn get_aria2c_override_args() -> Option<String>
{
    if let Some(val) = env::try_get("BEANS_ARIA2C_ARGS_OVERRIDE".to_string())
    {
        if val.len() > 0
        {
            return Some(val);
        }
    }
    if let Some(val) = env::try_get("ADASTRAL_ARIA2C_ARGS_OVERRIDE".to_string())
    {
        if val.len() > 0
        {
            return Some(val);
        }
    }
    return None;
}

/// Will return the content of either of the following environment variables
/// if they exist and there is at least 1 character in it;
/// - `BEANS_ARIA2C_ARGS`
/// - `ADASTRAL_ARIA2C_ARGS`
///
/// This string will be put in the launch argument for starting the aria2c
/// instance.
pub fn get_aria2c_extra_args() -> Option<String>
{
    if let Some(val) = env::try_get("BEANS_ARIA2C_ARGS".to_string())
    {
        if val.len() > 0
        {
            return Some(val);
        }
    }
    if let Some(val) = env::try_get("ADASTRAL_ARIA2C_ARGS".to_string())
    {
        if val.len() > 0
        {
            return Some(val);
        }
    }
    return None;
}

/// Return `true` when the environment variable exists, and it's value equals
/// `1` or `true (when trimmed and made lowercase).
fn check_bool<K: AsRef<std::ffi::OsStr>>(key: K) -> bool
{
    std::env::var(key).is_ok_and(|x| {
        let y = x.trim().to_lowercase();
        y == "1" || y == "true"
    })
}

/// Return `true` when `try_get_env_var` returns Some with a length greater than
/// `1`.
pub fn has(target_key: String) -> bool
{
    if let Some(x) = try_get(target_key)
    {
        return x.len() > 1;
    }
    false
}

/// Try and get a value from `std::env::vars()`
/// Will return `None` when not found
pub fn try_get(target_key: String) -> Option<String>
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
