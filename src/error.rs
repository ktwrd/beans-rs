use std::backtrace::Backtrace;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum BeansError
{
    /// Failed to check if there is free space. Value is the location
    #[error("Not enough free space in {0}")]
    FreeSpaceCheckFailure(String),
    /// Failed to find the sourcemod mod folder.
    #[error("Failed to detect sourcemod folder")]
    SourceModLocationNotFound,
    #[error("Failed to open file at {0}")]
    FileOpenFailure(String, std::io::Error),
    #[error("Failed to write file at {0}")]
    FileWriteFailure(String, std::io::Error),
    #[error("Failed to extract {src_file} to directory {target_dir}")]
    TarExtractFailure {
        src_file: String,
        target_dir: String,
        error: std::io::Error,
        backtrace: Backtrace
    },
    #[error("Failed to send request")]
    Reqwest(reqwest::Error),
    #[error("Failed to serialize or deserialize data")]
    SerdeJson(serde_json::Error),

    #[error("Latest version is already installed. (current: {current}, latest: {latest}")]
    LatestVersionAlreadyInstalled {
        current: usize,
        latest: usize
    },
    #[error("Failed to download file\n{reason:#?}")]
    DownloadFailure {
        reason: DownloadFailureReason
    },

    #[error("IO Error\n{0:#?}")]
    IO(std::io::Error)
}
#[derive(Debug)]
pub enum DownloadFailureReason
{
    Reqwest {
        url: String,
        error: reqwest::Error
    },
    /// The downloaded file could not be found, perhaps it failed?
    FileNotFound {
        location: String
    }
}
#[derive(Debug)]
pub struct TarExtractFailureDetails {
    pub source: String,
    pub target: String,
    pub error: std::io::Error
}

impl From<std::io::Error> for BeansError {
    fn from(e: std::io::Error) -> Self {
        BeansError::IO(e)
    }
}
impl From<reqwest::Error> for BeansError {
    fn from (e: reqwest::Error) -> Self {
        BeansError::Reqwest(e)
    }
}