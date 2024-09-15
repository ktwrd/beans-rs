use std::backtrace::Backtrace;

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
                Err(e) =>
                {
                    Err(BeansError::RegistryKeyFailure {
                        msg: "Failed to find HKCU\\Software\\Valve. Steam might not be installed"
                            .to_string(),
                        error: e,
                        backtrace: Backtrace::capture()
                    })
                }
            }
        }
        Err(e) =>
        {
            Err(BeansError::RegistryKeyFailure {
                msg: "Failed to find HKCU\\Software\\Valve. Steam might not be installed"
                    .to_string(),
                error: e,
                backtrace: Backtrace::capture()
            })
        }
    }
}
