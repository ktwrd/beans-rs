use std::backtrace::Backtrace;
use std::num::ParseIntError;
use thiserror::Error;
use crate::version::AdastralVersionFile;

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
    Reqwest {
        error: reqwest::Error,
        backtrace: Backtrace
    },
    #[error("Failed to serialize or deserialize data")]
    SerdeJson {
        error: serde_json::Error,
        backtrace: Backtrace
    },

    #[error("Latest version is already installed. (current: {current}, latest: {latest})")]
    LatestVersionAlreadyInstalled {
        current: usize,
        latest: usize
    },
    #[error("Failed to download file\n{reason:#?}")]
    DownloadFailure {
        reason: DownloadFailureReason
    },

    #[error("IO Error\n{error:#?}")]
    IO {
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Unable to perform action since the mod isn't installed since {missing_file} couldn't be found")]
    TargetSourcemodNotInstalled {
        missing_file: String,
        backtrace: Backtrace
    },

    #[error("Failed to run the verify command with butler.")]
    ButlerVerifyFailure {
        signature_url: String,
        gamedir: String,
        remote: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Failed to run the apply command with butler.")]
    ButlerPatchFailure {
        patchfile_location: String,
        gamedir: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Could not find file {location}")]
    FileNotFound {
        location: String,
        backtrace: Backtrace
    },

    #[error("Version {version:#?} could not be found on the server.")]
    RemoteVersionNotFound {
        version: Option<usize>
    },

    #[error("Could not find steam installation")]
    SteamNotFound,

    #[error("{msg}")]
    RegistryKeyFailure {
        msg: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Failed to migrate old version file to the new format at {location}")]
    VersionFileMigrationFailure {
        error: std::io::Error,
        location: String
    },
    #[error("Failed to delete old version file {location}")]
    VersionFileMigrationDeleteFailure {
        error: std::io::Error,
        location: String
    },
    #[error("Failed to convert version file to JSON format.")]
    VersionFileSerialize {
        error: serde_json::Error,
        instance: AdastralVersionFile
    },
    #[error("Failed to parse the version in {old_location}. It's content was {old_content}")]
    VersionFileParseFailure {
        error: ParseIntError,
        old_location: String,
        old_content: String
    },
    #[error("Failed to read version file at {location}. {error:}")]
    VersionFileReadFailure {
        error: std::io::Error,
        location: String
    }
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
        BeansError::IO {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}
impl From<reqwest::Error> for BeansError {
    fn from (e: reqwest::Error) -> Self {
        BeansError::Reqwest {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}
impl From<serde_json::Error> for BeansError {
    fn from(e: serde_json::Error) -> Self {
        BeansError::SerdeJson {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}