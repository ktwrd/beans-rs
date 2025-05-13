use std::{backtrace::Backtrace,
          fs::File};

use indicatif::{ProgressBar,
                ProgressStyle};
use log::{debug,
          error,
          info};
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::BeansError;

fn unpack_tarball_getfile(
    tarball_location: String,
    output_directory: String
) -> Result<File, BeansError>
{
    match File::open(&tarball_location)
    {
        Ok(x) => Ok(x),
        Err(e) => Err(BeansError::TarExtractFailure {
            src_file: tarball_location,
            target_dir: output_directory,
            error: e,
            backtrace: Backtrace::capture()
        })
    }
}

pub fn unpack_tarball(
    tarball_location: String,
    output_directory: String,
    show_progress: bool
) -> Result<(), BeansError>
{
    let mut tarball = unpack_tarball_getfile(tarball_location.clone(), output_directory.clone())?;
    let mut archive = tar::Archive::new(&tarball);

    if !show_progress
    {
        if let Err(e) = archive.unpack(&output_directory)
        {
            return Err(BeansError::TarExtractFailure {
                src_file: tarball_location,
                target_dir: output_directory,
                error: e,
                backtrace: Backtrace::capture()
            });
        }
        return Ok(());
    };

    let archive_entries = match archive.entries()
    {
        Ok(v) => v,
        Err(e) =>
        {
            return Err(BeansError::TarExtractFailure {
                src_file: tarball_location,
                target_dir: output_directory,
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let archive_entry_count = archive_entries.count() as u64;
    info!("Extracting {} files", archive_entry_count);

    tarball = unpack_tarball_getfile(tarball_location.clone(), output_directory.clone())?;
    archive = tar::Archive::new(&tarball);
    archive.set_preserve_permissions(false);

    let pb = ProgressBar::new(archive_entry_count);
    pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
    pb.set_message("Extracting files");

    let entries = match archive.entries()
    {
        Ok(a) => a,
        Err(error) =>
        {
            pb.finish_and_clear();
            return Err(BeansError::TarExtractFailure {
                src_file: tarball_location,
                target_dir: output_directory,
                error,
                backtrace: Backtrace::capture()
            });
        }
    };

    for (size, entry) in entries.enumerate()
    {
        match entry
        {
            Ok(mut x) =>
            {
                pb.set_message("Extracting files");
                let mut filename = String::new();

                if let Ok(Some(p)) = x.link_name()
                {
                    if let Some(s) = p.to_str()
                    {
                        pb.set_message(s.to_string());
                        filename = String::from(s);
                    }
                }

                if filename.len() == 0
                {
                    if let Ok(entry_path) = x.path()
                    {
                        if let Some(ep_str) = entry_path.to_str()
                        {
                            let ep = ep_str.to_string();
                            filename = ep;
                        }
                    }
                }

                if let Err(error) = x.unpack_in(&output_directory)
                {
                    debug!("error={:#?}", error);
                    debug!("entry.path={:#?}", x.path());
                    debug!("entry.link_name={:#?}", x.link_name());
                    debug!("entry.size={:#?}", x.size());
                    debug!("size={size:}");

                    let error_str = format!("{:#?}", error);
                    if error_str.contains("io: Custom {")
                        && error_str.contains("error: TarError {")
                        && error_str.contains("kind: PermissionDenied")
                        && error_str.contains("io: Os {")
                        && error_str.contains("code: 5")
                    {
                        warn!("Failed to unpack file {filename} (Permission Denied, might be read-only)")
                    }
                    else
                    {
                        pb.finish_and_clear();
                        error!(
                            "[extract::unpack_tarball] Failed to unpack file {filename} ({error:})"
                        );
                        return Err(BeansError::TarUnpackItemFailure {
                            src_file: tarball_location,
                            target_dir: output_directory,
                            link_name: filename,
                            error,
                            backtrace: Backtrace::capture()
                        });
                    }
                }

                if let Ok(entry_path) = x.path()
                {
                    if let Some(ep_str) = entry_path.to_str()
                    {
                        let ep = ep_str.to_string();
                        let target_path = join_path(output_directory.clone(), ep);
                        if crate::helper::file_exists(target_path.clone())
                        {
                            if let Err(e) = crate::helper::unmark_readonly(target_path.clone())
                            {
                                debug!("Failed to unmark read-only on file: {target_path:} {e:#?}");
                            }
                        }
                    }
                }
                pb.inc(1);
            }
            Err(error) =>
            {
                pb.finish_and_clear();
                debug!("[extract::unpack_tarball] size={size:}, error={:#?}", error);
                error!("[extract::unpack_tarball] Failed to unpack entry ({error:})");
                return Err(BeansError::TarExtractFailure {
                    src_file: tarball_location,
                    target_dir: output_directory,
                    error,
                    backtrace: Backtrace::capture()
                });
            }
        }
    }
    pb.finish();
    Ok(())
}

pub fn decompress_zstd(
    zstd_location: String,
    output_file: String,
    show_progress: bool
) -> Result<(), BeansError>
{
    let zstd_file = File::open(&zstd_location)?;
    let zstd_file_length = &zstd_file.metadata()?.len();
    let mut tar_tmp_file = File::create_new(&output_file)?;
    if show_progress
    {
        let decoder = ZstdDecoder::new(zstd_file)?;
        // estimate extracted size as x2 since idk how to get the decompressed size with
        // zstd
        let pb_decompress = ProgressBar::new(*zstd_file_length * 2);
        pb_decompress
            .set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

        std::io::copy(&mut pb_decompress.wrap_read(decoder), &mut tar_tmp_file)
            .expect("Failed to decompress file");
        pb_decompress.finish();
    }
    else
    {
        zstd::stream::copy_decode(zstd_file, &tar_tmp_file)?;
    }

    Ok(())
}
