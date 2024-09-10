use std::{backtrace::Backtrace,
          fs::File,
          io::Read};

use indicatif::{ProgressBar,
                ProgressStyle};
use log::{debug, error, info};
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::BeansError;

pub fn unpack_tarball(
    tarball_location: String,
    output_directory: String,
    show_progress: bool
) -> Result<(), BeansError>
{
    let mut tarball = open_tarball_file(tarball_location.clone(), output_directory.clone())?;
    if show_progress
    {
        let mut archive_entries_instance = tar::Archive::new(&tarball);
        let archive_entries = match archive_entries_instance.entries()
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
        let archive_entry_count = (&archive_entries.count()).clone() as u64;
        info!("Extracting {} files", archive_entry_count);

        let pb = ProgressBar::new(archive_entry_count);
        pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        pb.set_message("Extracting files");

        // re-open the file, since tar::Archive::new will not work with a re-used file.
        tarball = open_tarball_file(tarball_location.clone(), output_directory.clone())?;
        let mut archive_inner = tar::Archive::new(&tarball);
        archive_inner.set_preserve_permissions(false);
        let mut idx: u64 = 0;
        match archive_inner.entries()
        {
            Ok(etrs) =>
            {
                for (size, entry) in etrs.enumerate()
                {
                    idx += 1;
                    match entry
                    {
                        Ok(mut x) =>
                        {
                            let ln = x.link_name();
                            pb.set_message("Extracting files");
                            let mut filename = String::new();
                            if let Ok(n) = ln
                            {
                                if let Some(p) = n
                                {
                                    if let Some(s) = p.to_str()
                                    {
                                        pb.set_message(format!("{:}", s));
                                        filename = String::from(s);
                                    }
                                }
                            }

                            if let Err(e) = x.unpack_in(&output_directory)
                            {
                                pb.finish_and_clear();
                                debug!("[{idx:}] error={:#?}", e);
                                debug!("[{idx:}] entry.path={:#?}", x.path());
                                debug!("[{idx:}] entry.link_name={:#?}", x.link_name());
                                debug!("[{idx:}] entry.size={:#?}", x.size());
                                debug!("[{idx:}] size={size:}");
                                error!("[extract::unpack_tarball] Failed to unpack file {filename} ({e:})");
                                let error = BeansError::TarUnpackItemFailure {
                                    src_file: tarball_location,
                                    target_dir: output_directory,
                                    link_name: filename,
                                    error: e,
                                    backtrace: Backtrace::capture(),
                                };
                                debug!("[{idx:}] {:#?}", error);
                                return Err(error);
                            }
                            pb.inc(1);
                        }
                        Err(e) =>
                        {
                            pb.finish_and_clear();
                            debug!("[{idx:}] error={:#?}", e);
                            debug!("[extract::unpack_tarball] idx: {idx:}, size={size:}");
                            error!("[extract::unpack_tarball] Failed to unpack entry ({e:})");
                            return Err(BeansError::TarExtractFailure {
                                src_file: tarball_location,
                                target_dir: output_directory,
                                error: e,
                                backtrace: Backtrace::capture()
                            });
                        }
                    };
                }
            }
            Err(e) =>
            {
                pb.finish_and_clear();
                debug!("{:#?}", e);
                error!("[extract::unpack_tarball] Failed to extract tarball entries (src: {tarball_location:}, dest: {output_directory:}, error: {e:})");
                return Err(BeansError::TarExtractFailure {
                    src_file: tarball_location,
                    target_dir: output_directory,
                    error: e,
                    backtrace: Backtrace::capture()
                });
            }
        };
        pb.finish();
        debug!("[extract::unpack_tarball] Total entries extracted: {idx:}");
    }
    else
    {
        if let Err(e) = tar::Archive::new(&tarball).unpack(&output_directory)
        {
            debug!("{:#?}", e);
            error!("[extract::unpack_tarball] Failed to unpack {tarball_location} to directory {output_directory} ({e:}");
            return Err(BeansError::TarExtractFailure {
                src_file: tarball_location,
                target_dir: output_directory,
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    }
    Ok(())
}
fn open_tarball_file(tarball_location: String, output_directory: String) -> Result<File, BeansError>
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
        let pb_decompress = ProgressBar::new((zstd_file_length.clone() * 2) as u64);
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
