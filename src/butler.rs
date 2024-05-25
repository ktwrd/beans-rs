use crate::{BeansError, depends, DownloadFailureReason, helper};

pub fn verify(
    signature_url: String,
    gamedir: String,
    remote: String
) -> Result<(), BeansError> {
    let (_, butler_path) = depends::binary_locations();
    match std::process::Command::new(&butler_path)
        .args([
            "verify",
            &signature_url,
            &gamedir,
            format!("--heal=archive,{}", remote).as_str()
        ])
        .output() {
        Err(e) => {
            Err(BeansError::ButlerVerifyFailure {
                signature_url,
                gamedir,
                remote,
                error: e
            })
        },
        Ok(_) => Ok(())
    }
}
pub async fn patch_dl(
    dl_url: String,
    staging_dir: String,
    patch_filename: String,
    gamedir: String
) -> Result<(), BeansError> {
    if helper::file_exists(staging_dir.clone()) {
        std::fs::remove_dir_all(&staging_dir)?;
    }
    let tmp_file = helper::get_tmp_file(patch_filename);
    println!("[butler::patch_dl] downloading {} to {}", dl_url, tmp_file);
    helper::download_with_progress(dl_url, tmp_file.clone()).await?;

    if helper::file_exists(tmp_file.clone()) == false {
        return Err(BeansError::DownloadFailure {
            reason: DownloadFailureReason::FileNotFound {
                location: tmp_file
            }
        });
    }

    patch(tmp_file, staging_dir, gamedir)
}

pub fn patch(
    patchfile_location: String,
    staging_dir: String,
    gamedir: String
) -> Result<(), BeansError> {
    println!("[butler::patch] patching directory {} with {}", gamedir, patchfile_location);
    let (_, butler_path) = depends::binary_locations();
    match std::process::Command::new(&butler_path)
        .args([
            "apply",
            &format!("--staging-dir={}", staging_dir),
            &patchfile_location,
            &gamedir
        ])
        .output() {
        Err(e) => {
            Err(BeansError::ButlerPatchFailure {
                patchfile_location,
                gamedir,
                error: e
            })
        },
        Ok(_) => Ok(())
    }
}