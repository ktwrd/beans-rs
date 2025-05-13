use std::{backtrace::Backtrace,
          ffi::OsStr,
          os::windows::fs::MetadataExt};

use bitflags::bitflags;
use log::debug;
use widestring::U16String;
use windows::{core::PCWSTR,
              Win32::Storage::FileSystem::*};
use winreg::{enums::HKEY_CURRENT_USER,
             RegKey};

use crate::{helper::format_directory_path,
            BeansError};

/// TODO use windows registry to get the SourceModInstallPath
/// HKEY_CURRENT_USER\Software\Value\Steam
/// Key: SourceModInstallPath
pub fn find_sourcemod_path() -> Result<String, BeansError>
{
    match RegKey::predef(HKEY_CURRENT_USER).open_subkey(String::from("Software\\Valve\\Steam"))
    {
        Ok(rkey) =>
        {
            let x: std::io::Result<String> = rkey.get_value("SourceModInstallPath");
            match x
            {
                Ok(val) => Ok(format_directory_path(val)),
                Err(e) => Err(BeansError::RegistryKeyFailure {
                    msg: "Failed to find HKCU\\Software\\Valve. Steam might not be installed"
                        .to_string(),
                    error: e,
                    backtrace: Backtrace::capture()
                })
            }
        }
        Err(e) => Err(BeansError::RegistryKeyFailure {
            msg: "Failed to find HKCU\\Software\\Valve. Steam might not be installed".to_string(),
            error: e,
            backtrace: Backtrace::capture()
        })
    }
}

/// Unmark a file as readonly. For some reason, `gameinfo.txt` is always
/// readonly, and trying to replace it fails. Unmarking it when this function
/// when extracting should fix the issue.
pub fn unmark_readonly(location: String) -> Result<(), BeansError>
{
    if !crate::helper::file_exists(location.clone())
    {
        debug!("[windows::unmark_readonly] file does not exist: {location:}");
        return Ok(());
    }

    let s = location.as_str();
    let previous = match get_windows_file_attributes(&s)
    {
        Ok(a) => a,
        Err(e) =>
        {
            return Err(BeansError::ReadFileAttributesError {
                error: e,
                location: s.to_string(),
                backtrace: Backtrace::capture()
            });
        }
    };

    if !previous.contains(WindowsFileAttribute::ReadOnly)
    {
        return Ok(());
    }

    let mut new_attr = previous - WindowsFileAttribute::ReadOnly;
    if new_attr.contains(WindowsFileAttribute::Archive)
    {
        new_attr -= WindowsFileAttribute::Archive;
    }

    if new_attr.is_empty()
    {
        new_attr = WindowsFileAttribute::Normal;
    }

    match set_file_attributes_win(&s, new_attr)
    {
        Ok(_) => Ok(()),
        Err(e) =>
        {
            let hr = e.code().0;
            debug!("file.name={s}");
            debug!("file.attr={new_attr:#?}");
            Err(BeansError::WindowsSetFileAttributeError {
                hresult: hr,
                hresult_msg: e.message(),
                location: s.to_string(),
                backtrace: Backtrace::capture()
            })
        }
    }
}

fn set_file_attributes_win<P: AsRef<OsStr>>(
    location: P,
    attr: WindowsFileAttribute
) -> windows::core::Result<()>
{
    if let Some(location_str) = location.as_ref().to_str()
    {
        if crate::helper::file_exists(format!("{location_str}"))
        {
            let s = U16String::from_str(location_str);
            let mut a = attr.clone();
            if a.is_empty()
            {
                a = WindowsFileAttribute::Normal;
            }
            let win32_attr = FILE_FLAGS_AND_ATTRIBUTES(a.bits());
            unsafe {
                let win32_loc = PCWSTR(s.as_ptr());
                SetFileAttributesW(win32_loc, win32_attr)?;
            }
        }
    }
    Ok(())
}

fn get_windows_file_attributes<P: AsRef<std::path::Path>>(
    location: &P
) -> std::io::Result<WindowsFileAttribute>
{
    let metadata = std::fs::metadata(location)?;
    let attr = metadata.file_attributes();
    if let Some(flags) = WindowsFileAttribute::from_bits(attr)
    {
        return Ok(flags);
    }
    else
    {
        return Ok(WindowsFileAttribute::empty());
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WindowsFileAttribute : u32 {
        const ReadOnly = 1;
        const Hidden = 2;
        const System = 4;
        const Archive = 32;
        const Normal = 128;
        const Temporary = 256;
        const Offline = 4096;
        const NonContentIndexed = 8192;
    }
}
