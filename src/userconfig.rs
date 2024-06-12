use lazy_static::lazy_static;
use log::debug;
use crate::{BeansError, helper};
use crate::appvar::AppVarData;

pub fn get_cfg_dir() -> String
{
    let cfg = dirs::config_local_dir();
    match cfg {
        Some(d) => {
            match d.to_str() {
                Some(s) => {
                    helper::join_path(s.to_string(), String::from("brioche"))
                },
                None => {
                    debug!("[userconfig::get_cfg_dir] Failed to turn dirs::config_local_dir into &str");
                    return helper::parse_location(String::from("./.config/"))
                }
            }
        },
        None => {
            debug!("[userconfig::get_cfg_dir] Failed to get config directory from dirs::config_local_dir()");
            return helper::parse_location(String::from("./.config/"))
        }
    }
}
pub fn get_cfg_location() -> String
{
    helper::join_path(get_cfg_dir(), String::from("config.json"))
}

lazy_static! {
    static ref JSON_DATA: RwLock<String> = RwLock::new(String::from("{}"));
    static ref UCD_INSTANCE: RwLock<Option<UserConfigData>> = RwLock::new(None);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UserConfigData
{
    pub sourcemods_location: String
}
impl UserConfigData {
    /// Parse `JSON_DATA` to AppVarData. Should only be called by `reset()`.
    ///
    /// NOTE panics when `serde_json::from_str()` is Err, or when `JSON_DATA.read()` is Err.
    /// REMARKS does not set `UCD_INSTANCE` to generated data, since this is only done by
    /// `UserConfigData::set_instance()`.
    pub fn parse() -> Self {
        debug!("[UserConfigData::parse] trying to get JSON_DATA");
        let x = JSON_DATA.read();
        if let Ok(data) = x {
            debug!("[UserConfigData::parse] JSON_DATA= {:#?}", data);
            return serde_json::from_str(&data).expect("Failed to deserialize JSON_DATA");
        }
        if let Err(e) = x {
            panic!("[UserConfigData::parse] Failed to read JSON_DATA {:#?}", e);
        }
        unreachable!();
    }

    /// Try and read the data from `UCD_INSTANCE` and return when some.
    /// Otherwise, when it's none, we return `UserConfigData::set_instance()`
    ///
    /// NOTE this function panics when Err on `UCD_INSTANCE.read()`.
    pub fn get() -> Self {
        let avd_read = UCD_INSTANCE.read();
        if let Ok(v) = avd_read {
            let vc = v.clone();
            if let Some(x) = vc {
                debug!("[UserConfigData::get] Instance exists in UCD_INSTANCE, so lets return that.");
                return x;
            }
        }
        else if let Err(e) = avd_read {
            panic!("[UserConfigData::get] Failed to read UCD_INSTANCE {:#?}", e);
        }

        Self::set_instance()
    }

    /// Set the content of `UCD_INSTANCE` to the result of `UserConfigData::parse()`
    ///
    /// NOTE this function panics when Err on `UCD_INSTANCE.write()`
    pub fn set_instance() -> Self {
        let instance = Self::parse();

        match UCD_INSTANCE.write() {
            Ok(mut data) => {
                *data = Some(instance.clone());
                debug!("[set_instance] set content of AVD_INSTANCE to {:#?}", instance);
            },
            Err(e) => {
                panic!("[set_instance] Failed to set AVD_INSTANCE! {:#?}", e);
            }
        }

        instance
    }
}