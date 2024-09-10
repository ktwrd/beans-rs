use std::{backtrace::Backtrace,
          fmt::{Display,
                Formatter},
          num::ParseIntError};

use thiserror::Error;

use crate::{appvar::AppVarData,
            version::AdastralVersionFile};

#[derive(Debug, Error)]
pub enum BeansError
{
    /// Failed to check if there is free space. Value is the location
    #[error("Not enough free space in {location}")]
    FreeSpaceCheckFailure
    {
        location: String
    },
    /// Failed to find the sourcemod mod folder.
    #[error("Failed to detect sourcemod folder. Please provide it via the --location argument.")]
    SourceModLocationNotFound,
    #[error("Failed to open file at {location} ({error:})")]
    FileOpenFailure
    {
        location: String,
        error: std::io::Error
    },
    #[error("Failed to write file at {location} ({error:})")]
    FileWriteFailure
    {
        location: String,
        error: std::io::Error
    },
    #[error("Failed to create directory {location} ({error:})")]
    DirectoryCreateFailure
    {
        location: String,
        error: std::io::Error,
        backtrace: Backtrace
    },
    #[error("Failed to delete directory {location} ({error:})")]
    DirectoryDeleteFailure
    {
        location: String,
        error: std::io::Error
    },
    #[error("Failed to extract {src_file} to directory {target_dir} ({error:})")]
    TarExtractFailure
    {
        src_file: String,
        target_dir: String,
        error: std::io::Error,
        backtrace: Backtrace
    },
    #[error("Failed to extract item {link_name} to directory {target_dir} ({error:})")]
    TarUnpackItemFailure
    {
        src_file: String,
        target_dir: String,
        link_name: String,
        error: std::io::Error,
        backtrace: Backtrace
    },
    #[error("Failed to send request ({error:})")]
    Reqwest
    {
        error: reqwest::Error,
        backtrace: Backtrace
    },
    #[error("Failed to serialize or deserialize data ({error:})")]
    SerdeJson
    {
        error: serde_json::Error,
        backtrace: Backtrace
    },

    #[error("Latest version is already installed. (current: {current}, latest: {latest})")]
    LatestVersionAlreadyInstalled
    {
        current: usize, latest: usize
    },
    #[error("Failed to download file\n{reason:#?}")]
    DownloadFailure
    {
        reason: DownloadFailureReason
    },

    #[error("General IO Error\n{error:#?}")]
    IO
    {
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Unable to perform action since the mod isn't installed since {missing_file} couldn't be found")]
    TargetSourcemodNotInstalled
    {
        missing_file: String,
        backtrace: Backtrace
    },

    #[error("Failed to run the verify command with butler. ({error:})")]
    ButlerVerifyFailure
    {
        signature_url: String,
        gamedir: String,
        remote: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Failed to run the apply command with butler. {error:}")]
    ButlerPatchFailure
    {
        patchfile_location: String,
        gamedir: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Could not find file {location}")]
    FileNotFound
    {
        location: String,
        backtrace: Backtrace
    },

    #[error("Version {version:#?} could not be found on the server.")]
    RemoteVersionNotFound
    {
        version: Option<usize>
    },

    #[error("Could not find steam installation, which means we can't find the sourcemods folder. Please provide the sourcemods folder with the --location parameter.")]
    SteamNotFound,

    #[error("{msg}")]
    RegistryKeyFailure
    {
        msg: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("Failed to migrate old version file to the new format at {location} ({error:})")]
    VersionFileMigrationFailure
    {
        error: std::io::Error,
        location: String
    },
    #[error("Failed to delete old version file {location} ({error:})")]
    VersionFileMigrationDeleteFailure
    {
        error: std::io::Error,
        location: String
    },
    #[error("Failed to convert version file to JSON format. ({error:})")]
    VersionFileSerialize
    {
        error: serde_json::Error,
        instance: AdastralVersionFile
    },
    #[error(
        "Failed to parse the version in {old_location}. It's content was {old_content} ({error:})"
    )]
    VersionFileParseFailure
    {
        error: ParseIntError,
        old_location: String,
        old_content: String
    },
    #[error("Failed to read version file at {location}. ({error:})")]
    VersionFileReadFailure
    {
        error: std::io::Error,
        location: String
    },

    #[error("Failed to serialize provided AppVarData to JSON. ({error:})")]
    AppVarDataSerializeFailure
    {
        error: serde_json::Error,
        data: AppVarData
    },

    #[error("Failed to read gameinfo.txt at {location} ({error:})")]
    GameInfoFileReadFail
    {
        error: std::io::Error,
        location: String,
        backtrace: Backtrace
    },

    #[error("Failed to set permissions on gameinfo.txt at {location} ({error:})")]
    GameInfoPermissionSetFail
    {
        error: std::io::Error,
        permissions: std::fs::Permissions,
        location: String
    },

    #[error("Failed to backup gameinfo.txt, {reason:}")]
    GameinfoBackupFailure
    {
        reason: GameinfoBackupFailureReason
    },

    #[error("Failed to remove files in {location} ({error:})")]
    CleanTempFailure
    {
        location: String,
        error: std::io::Error,
        backtrace: Backtrace
    },

    #[error("{name:} ({pid:}) is still running. Please close it and restart beans.")]
    GameStillRunning
    {
        name: String, pid: String
    }
}
#[derive(Debug)]
pub enum DownloadFailureReason
{
    Reqwest
    {
        url: String, error: reqwest::Error
    },
    /// The downloaded file could not be found, perhaps it failed?
    FileNotFound
    {
        location: String
    }
}
#[derive(Debug)]
pub enum GameinfoBackupFailureReason
{
    ReadContentFail(GameinfoBackupReadContentFail),
    BackupDirectoryCreateFailure(GameinfoBackupCreateDirectoryFail),
    WriteFail(GameinfoBackupWriteFail)
}
#[derive(Debug)]
pub struct GameinfoBackupReadContentFail
{
    pub error: std::io::Error,
    pub proposed_location: String,
    pub current_location: String
}
#[derive(Debug)]
pub struct GameinfoBackupCreateDirectoryFail
{
    pub error: std::io::Error,
    pub location: String
}
#[derive(Debug)]
pub struct GameinfoBackupWriteFail
{
    pub error: std::io::Error,
    pub location: String
}
impl Display for GameinfoBackupFailureReason
{
    fn fmt(
        &self,
        f: &mut Formatter<'_>
    ) -> std::fmt::Result
    {
        match self
        {
            GameinfoBackupFailureReason::ReadContentFail(v) => write!(
                f,
                "Couldn't read the content at {} ({:})",
                v.current_location, v.error
            ),
            GameinfoBackupFailureReason::BackupDirectoryCreateFailure(v) => write!(
                f,
                "Couldn't create backups directory {} ({:})",
                v.location, v.error
            ),
            GameinfoBackupFailureReason::WriteFail(v) => write!(
                f,
                "Failed to write content to {} ({:})",
                v.location, v.error
            )
        }
    }
}
#[derive(Debug)]
pub struct TarExtractFailureDetails
{
    pub source: String,
    pub target: String,
    pub error: std::io::Error
}

impl From<std::io::Error> for BeansError
{
    fn from(e: std::io::Error) -> Self
    {
        BeansError::IO {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}
impl From<reqwest::Error> for BeansError
{
    fn from(e: reqwest::Error) -> Self
    {
        BeansError::Reqwest {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}
impl From<serde_json::Error> for BeansError
{
    fn from(e: serde_json::Error) -> Self
    {
        BeansError::SerdeJson {
            error: e,
            backtrace: Backtrace::capture()
        }
    }
}
