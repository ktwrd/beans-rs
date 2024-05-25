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
    DownloadFailure(String, reqwest::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error)
}

impl<E> From<E> for BeansError
    where
        E: Into<reqwest::Error>,
{
    fn from(err: E) -> Self {
        BeansError::Reqwest(err.into())
    }
}