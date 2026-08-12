/// Version file that is used as `.adastral` in the sourcemod mod folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdastralVersionFile
{
    pub version: String
}
