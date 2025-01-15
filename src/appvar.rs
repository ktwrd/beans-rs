use std::sync::RwLock;

use lazy_static::lazy_static;
use log::{debug,
          error};

use crate::BeansError;

/// Default `appvar.json` to use.
pub const JSON_DATA_DEFAULT: &str = include_str!("appvar.json");
lazy_static! {
    static ref JSON_DATA: RwLock<String> = RwLock::new(JSON_DATA_DEFAULT.to_string());
    static ref AVD_INSTANCE: RwLock<Option<AppVarData>> = RwLock::new(None);
}

/// Configuration for the compiled application.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppVarData
{
    #[serde(rename = "mod")]
    pub mod_info: AppVarMod,
    #[serde(rename = "remote")]
    pub remote_info: AppVarRemote
}
impl AppVarData
{
    /// Parse `JSON_DATA` to AppVarData. Should only be called by
    /// `reset_appvar()`.
    ///
    /// NOTE panics when `serde_json::from_str()` is Err, or when
    /// `JSON_DATA.read()` is Err. REMARKS does not set `AVD_INSTANCE` to
    /// generated data, since this is only done by `AppVarData::reset()`.
    pub fn parse() -> Self
    {
        debug!("[AppVarData::parse] trying to get JSON_DATA");
        let x = JSON_DATA.read();
        if let Ok(data) = x
        {
            debug!("[AppVarData::parse] JSON_DATA= {:#?}", data);
            return serde_json::from_str(&data).expect("Failed to deserialize JSON_DATA");
        }
        if let Err(e) = x
        {
            panic!("[AppVarData::parse] Failed to read JSON_DATA {:#?}", e);
        }
        unreachable!();
    }

    /// Substitute values in the `source` string for what is defined in here.
    pub fn sub(
        &self,
        source: String
    ) -> String
    {
        source
            .clone()
            .replace("$MOD_NAME_STYLIZED", &self.mod_info.name_stylized)
            .replace("$MOD_NAME_SHORT", &self.mod_info.short_name)
            .replace("$MOD_NAME", &self.mod_info.sourcemod_name)
            .replace("$URL_BASE", &self.remote_info.base_url)
            .replace("$URL_VERSIONS", &self.remote_info.versions_url)
    }

    /// Try and read the data from `AVD_INSTANCE` and return when some.
    /// Otherwise, when it's none, we return `AppVarData::reset()`
    ///
    /// NOTE this function panics when Err on `AVD_INSTANCE.read()`.
    pub fn get() -> Self
    {
        let avd_read = AVD_INSTANCE.read();
        if let Ok(v) = avd_read
        {
            let vc = v.clone();
            if let Some(x) = vc
            {
                #[cfg(debug_assertions)]
                debug!("[AppVarData::get] Instance exists in AVD_INSTANCE, so lets return that.");
                return x;
            }
        }
        else if let Err(e) = avd_read
        {
            panic!("[AppVarData::get] Failed to read AVD_INSTANCE {:#?}", e);
        }

        Self::reset()
    }

    /// Set the content of `AVD_INSTANCE` to the result of `AppVarData::parse()`
    ///
    /// NOTE this function panics when Err on `AVD_INSTANCE.write()`
    pub fn reset() -> Self
    {
        let instance = AppVarData::parse();

        match AVD_INSTANCE.write()
        {
            Ok(mut data) =>
            {
                *data = Some(instance.clone());
                debug!(
                    "[reset_appvar] set content of AVD_INSTANCE to {:#?}",
                    instance
                );
            }
            Err(e) =>
            {
                panic!("[AppVarData::reset] Failed to set AVD_INSTANCE! {:#?}", e);
            }
        }

        instance
    }

    /// Serialize `data` into JSON, then set the content of `JSON_DATA` to the
    /// serialize content. Once that is done, `AppVarData::reset()` will be
    /// called.
    ///
    /// If `serde_json::to_string` fails, an error is printed in console and
    /// `sentry::capture_error` is called.
    pub fn set_json_data(data: AppVarData) -> Result<(), BeansError>
    {
        debug!("[AppVarData::set_json_data] {:#?}", data);
        match serde_json::to_string(&data)
        {
            Ok(v) =>
            {
                if let Ok(mut ms) = JSON_DATA.write()
                {
                    *ms = v.to_string();
                    debug!("[AppVarData::set_json_data] successfully set data, calling reset_appvar()");
                }
                Self::reset();
                Ok(())
            }
            Err(e) =>
            {
                error!(
                    "[AppVarData::set_json_data] Failed to serialize data to string! {:}",
                    e
                );
                debug!("{:#?}", e);
                sentry::capture_error(&e);

                Err(BeansError::AppVarDataSerializeFailure {
                    error: e,
                    data: data.clone()
                })
            }
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppVarMod
{
    /// name of the mod to use.
    /// e.g; `open_fortress`
    #[serde(rename = "sm_name")]
    pub sourcemod_name: String,
    /// two-letter abbreviation that is used in `versions.json` for the game.
    /// e.g; `of`
    pub short_name: String,
    /// stylized name of the sourcemod.
    /// e.g; `Open Fortress`
    pub name_stylized: String
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppVarRemote
{
    /// base URL for the versioning.
    /// e.g; `https://beans.adastral.net/`
    pub base_url: String,
    /// url where the version details are stored.
    /// e.g; `https://beans.adastral.net/versions.json`
    pub versions_url: String
}
