use std::collections::HashMap;
use std::io::Write;
use std::sync::RwLock;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use crate::{BeansError, DownloadFailureReason, helper};

#[derive(Debug, Clone)]
pub struct DownloadProgressArgs {
    pub unique_id: String,
    pub max: u64,
    pub current: u64
}
#[derive(Debug, Clone)]
pub struct DownloadCompleteArgs {
    pub unique_id: String,
    pub dl_size: u64,
    pub error_content: Option<String>
}
fn cli_download_style(pb: &ProgressBar) {
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
}
fn create_progress_bar(max: u64, url: &String) -> ProgressBar {
    let m = ProgressBar::new(max);
    cli_download_style(&m);
    m.set_message(format!("Downloading {}", &url));
    m
}
lazy_static!{
    static ref PBAR_MAP: RwLock<HashMap::<String, ProgressBar>> = RwLock::new(HashMap::new());
}
pub async fn with_progress_cli(url: String, out_location: String) -> Result<(), BeansError> {
    with_progress_custom(url.clone(), out_location.clone(),
        move |a: DownloadProgressArgs| {
            if let Ok(map) = PBAR_MAP.write() {
                if let Some(p) = map.get(&a.unique_id) {
                    p.set_position(a.current);
                }
            }
        }, move |b: DownloadProgressArgs| {
            if let Ok(mut map) = PBAR_MAP.write() {
                let p = create_progress_bar(b.max, &url);
                cli_download_style(&p);
                p.set_message(format!("Downloading {}", &url));
                map.insert(b.unique_id, p);
            }
        }, move |c: DownloadCompleteArgs| {
            if let Ok(mut map) = PBAR_MAP.write() {
                if let Some(p) = map.get(&c.unique_id) {
                    p.finish();
                }
                map.remove(&c.unique_id);
            }
        }).await
}
pub async fn with_progress_custom<FP, FI, FC>(
    url: String,
    out_location: String,
    progress: FP,
    progress_init: FI,
    progress_complete: FC) -> Result<(), BeansError>
where FP: Fn(DownloadProgressArgs),
      FI: Fn(DownloadProgressArgs),
      FC: Fn(DownloadCompleteArgs)
{
    let res = match reqwest::Client::new()
        .get(&url)
        .send()
        .await {
        Ok(v) => v,
        Err(e) => {
            sentry::capture_error(&e);
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::Reqwest {
                    url: url.clone(),
                    error: e
                }
            });
        }
    };

    let total_size = res
        .content_length()
        .expect("Failed to get length of data to download");

    let unique_id = helper::generate_rand_str(32);
    progress_init(DownloadProgressArgs {
        unique_id: unique_id.clone(),
        max: total_size,
        current: 0
    });


    // download chunks
    let mut file = match std::fs::File::create(out_location.clone()) {
        Ok(v) => v,
        Err(e) => {
            sentry::capture_error(&e);
            progress_complete(DownloadCompleteArgs {
                unique_id: unique_id.clone(),
                dl_size: 0,
                error_content: Some(format!("{:#?}", e))
            });
            return Err(BeansError::FileOpenFailure {
                location: out_location,
                error: e
            });
        }
    };
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(v) => v,
            Err(e) => {
                progress_complete(DownloadCompleteArgs {
                    unique_id: unique_id.clone(),
                    dl_size: downloaded,
                    error_content: Some(format!("{:#?}", e))
                });
                return Err(BeansError::DownloadFailure {
                    reason: DownloadFailureReason::ReqwestDownloadIncomplete {
                        url: url.clone(),
                        error: e,
                        current_size: downloaded
                    }
                });
            }
        };
        if let Err(e) = file.write_all(&chunk) {
            progress_complete(DownloadCompleteArgs {
                unique_id: unique_id.clone(),
                dl_size: downloaded,
                error_content: Some(format!("{:#?}", e))
            });
            return Err(BeansError::DownloadFailure {
                reason: DownloadFailureReason::DownloadIncomplete {
                    url: url.clone(),
                    error: e,
                    current_size: downloaded
                }
            });
        }
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        progress(DownloadProgressArgs {
            unique_id: unique_id.clone(),
            max: total_size,
            current: new
        });
    }

    progress_complete(DownloadCompleteArgs {
        unique_id: unique_id.clone(),
        dl_size: total_size,
        error_content: None
    });
    Ok(())
}