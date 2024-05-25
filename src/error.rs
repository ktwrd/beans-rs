#[derive(Debug)]
pub enum BeansError
{
    /// Failed to check if there is free space. Value is the location
    FreeSpaceCheckFailure(String),
    /// Failed to find the sourcemod mod folder.
    SourceModLocationNotFound,
    FileOpenFailure(String, std::io::Error),
    FileWriteFailure(String, std::io::Error),
    TarExtractFailure(String, String, std::io::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),

    LatestVersionAlreadyInstalled,
    DownloadFailure(DownloadFailureReason)
}
#[derive(Debug)]
pub enum DownloadFailureReason
{
    /// url, reqwest::Error
    Reqwest(String, reqwest::Error),
    /// The downloaded file could not be found, perhaps it failed?
    FileNotFound(String)
}

impl<E> From<E> for BeansError
    where
        E: Into<reqwest::Error>,
{
    fn from(err: E) -> Self {
        BeansError::Reqwest(err.into())
    }
}