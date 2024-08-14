use log::info;
use std::{backtrace::Backtrace, fs::File, io::Read};
use indicatif::{ProgressBar, ProgressStyle};
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::BeansError;

pub fn unpack_tarball(tarball_location: String, output_directory: String, show_progress: bool) -> Result<(), BeansError>
{
    let tarball = match File::open(&tarball_location) {
        Ok(x) => x,
        Err(e) => {
            return Err(BeansError::TarExtractFailure {
                src_file: tarball_location,
                target_dir: output_directory,
                error: e,
                backtrace: Backtrace::capture()
            });
        }
    };
    let mut archive = tar::Archive::new(&tarball);
    if show_progress {
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
        let archive_entry_count = (&archive_entries.count()).clone() as u64;
        info!("Extracting {} files", archive_entry_count);
    
        let pb = ProgressBar::new(archive_entry_count);
        pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        pb.set_message("Extracting files");
    
        archive = tar::Archive::new(&tarball);
        match archive.entries() {
            Ok(etrs) =>
            {
                for entry in etrs {
                    match entry {
                        Ok(mut x) => {
                            let ln = x.link_name();
                            pb.set_message("Extracting files");
                            let mut filename = String::new();
                            if let Ok(n) = ln {
                                if let Some(p) = n {
                                    if let Some(s) = p.to_str() {
                                        pb.set_message(format!("{:}", s));
                                        filename = String::from(s);
                                    }
                                }
                            }
                            if let Err(e) = x.unpack_in(&output_directory) {
                                return Err(BeansError::TarUnpackItemFailure {
                                    src_file: tarball_location,
                                    target_dir: output_directory,
                                    link_name: filename,
                                    error: e,
                                    backtrace: Backtrace::capture()
                                });
                            }
                            pb.inc(1);
                        },
                        Err(e) => {
                            return Err(BeansError::TarExtractFailure {
                                src_file: tarball_location,
                                target_dir: output_directory,
                                error: e,
                                backtrace: Backtrace::capture()
                            });
                        }
                    };
                }
            },
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
        pb.finish();
    } else {
        archive.unpack(output_directory);
    }
    return Ok(());
}
pub fn decompress_zstd(zstd_location: String, output_file: String, show_progress: bool) -> Result<(), BeansError>
{
    let zstd_file = File::open(&zstd_location)?;
    let zstd_file_length = &zstd_file.metadata()?.len();
    let mut tar_tmp_file = File::create_new(&output_file)?;
    if show_progress {
        let decoder = ZstdDecoder::new(zstd_file)?;
        // estimate extracted size as x2 since idk how to get the decompressed size with zstd
        let pb_decompress = ProgressBar::new((zstd_file_length.clone() * 2) as u64);
        pb_decompress
            .set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        
        std::io::copy(&mut pb_decompress.wrap_read(decoder), &mut tar_tmp_file).expect("Failed to decompress file");
        pb_decompress.finish();
    } else {
        zstd::stream::copy_decode(zstd_file, &tar_tmp_file)?;
    }

    Ok(())
}