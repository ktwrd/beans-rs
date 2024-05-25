use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;
use crate::helper::generate_rand_str;

/// TODO use windows registry to get the SourceModInstallPath
/// HKEY_CURRENT_USER\Software\Value\Steam
/// Key: SourceModInstallPath
pub fn find_sourcemod_path() -> Option<String>
{
    todo!()
}

pub fn get_tmp_file(filename: String) -> String
{
    todo!()
}