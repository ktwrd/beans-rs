pub const JSON_DATA: &str = include_str!("appvar.json");
pub fn parse() -> AppVarData
{
    AppVarData::parse()
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
impl AppVarData {
    /// Parse `JSON_DATA` to Self. Uses `expect` btw!
    pub fn parse() -> Self {
        serde_json::from_str(JSON_DATA).expect("Failed to deserialize appvar.json")
    }

    /// Substitute values in the `source` string for what is defined in here.
    pub fn sub(&self, source: String) -> String
    {
        source.clone()
            .replace("$MOD_NAME_STYLIZED", &self.mod_info.name_stylized)
            .replace("$MOD_NAME_SHORT", &self.mod_info.short_name)
            .replace("$MOD_NAME", &self.mod_info.sourcemod_name)
            .replace("$URL_BASE", &self.remote_info.base_url)
            .replace("$URL_VERSIONS", &self.remote_info.versions_url)
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