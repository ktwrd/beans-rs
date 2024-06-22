use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use fltk::app;
use fltk::prelude::{GroupExt, WidgetBase, WidgetExt};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use tokio::runtime::Runtime;
use crate::{BeansError, DownloadFailureReason, helper};
use crate::gui::download_ui::DownloadProgressInterface;
use crate::gui::GUIAppStatus;

#[derive(Debug, Clone)]
pub struct DownloadProgressArgs {
    pub unique_id: String,
    pub max: u64,
    pub current: u64,
    pub speed: String
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
    static ref APP_MAP: RwLock<HashMap::<String, app::App>> = RwLock::new(HashMap::new());
    static ref GUI_MAP_COMPLETE: RwLock<HashMap::<String, Option<bool>>> = RwLock::new(HashMap::new());

    static ref GUI_DL_STATE: RwLock<HashMap::<String, DownloadGuiState>> = RwLock::new(HashMap::new());
    static ref GUI_DL_ERR: RwLock<HashMap::<String, BeansError>> = RwLock::new(HashMap::new());
    static ref GUI_DL_MAP: RwLock<HashMap::<String, String>> = RwLock::new(HashMap::new());
}
/// Download with either a GUI or CLI progress bar. Will use a GUI progress bar if `crate::HEADLESS`
/// is `false`.
pub async fn with_progress(url: String, out_location: String, title: String) -> Result<(), BeansError>
{
    unsafe {
        if crate::HEADLESS {
            with_progress_cli(url, out_location).await
        } else {
            with_progress_gui(url, out_location, title).await
        }
    }
}
/// Download a file without displaying a progress bar.
pub async fn silent(url: String, out_location: String) -> Result<(), BeansError>
{
    with_progress_custom(url, out_location, |_| {}, |_| {}, |_| {}, |_| {}).await
}
#[derive(Debug)]
struct DownloadGuiState
{
    pub progress_percentage: f64,
    pub label_left: String,
    pub label_right: String
}
impl Default for DownloadGuiState {
    fn default() -> Self {
        Self {
            progress_percentage: 0f64,
            label_left: String::default(),
            label_right: String::default(),
        }
    }
}

/// Download a file while using a FLTK window.
async fn with_progress_gui(url: String, out_location: String, title: String) -> Result<(), BeansError>
{
    debug!("[download::with_progress_gui] called!");
    let custom_id = helper::generate_rand_str(32);
    let custom_id_clone = custom_id.clone();
    let tmp_runtime = Arc::new(Runtime::new().expect("[download::with_progress_gui] Failed to make tokio runtime"));
    tmp_runtime.spawn(async move {
        debug!("[download::with_progress_gui] thread spawned");
        let q = with_progress_custom(
            url.clone(),
            out_location.clone(),
            move |a: DownloadProgressArgs| {
                std::thread::spawn(move || {
                    let p = helper::calc_percentage(a.current, a.max);
                    let data = DownloadGuiState {
                        progress_percentage: p,
                        label_left: format!("{} remaining", helper::format_size(a.max - a.current)),
                        label_right: format!("{}", a.speed)
                    };
                    if let Ok(id_map) = GUI_DL_MAP.read() {
                        if let Some(id_ref) = id_map.get(&a.unique_id) {
                            let id = id_ref.clone();
                            match GUI_DL_STATE.write() {
                                Ok(mut map) => {
                                    map.insert(id, data);
                                },
                                Err(e) => {
                                    error!("Failed to set GUI_DL_STATE[{}] {:#?}", id, e);
                                }
                            }
                        }
                    }
                });
            },
            |i| {
                debug!("[download::with_progress_gui] thread:progress_init {:#?}", i);
                if let Ok(mut map) = GUI_DL_MAP.write() {
                    map.insert(i.unique_id, custom_id_clone.clone());
                }
            },
            move |args| {
                debug!("[download::with_progress_gui] thread:progress_complete {:#?}", args);
                if let Ok(id_map) =  GUI_DL_MAP.read() {
                    if let Some(id_ref) = id_map.get(&args.unique_id) {
                        let id = id_ref.clone();
                        if let Ok(mut x) = GUI_MAP_COMPLETE.write() {
                            x.insert(id.clone(), Some(true));
                        }
                        if let Ok(mut map) = APP_MAP.write() {
                            if let Some(app) = map.get_mut(&id) {
                                trace!("[download::with_progress_gui->progress_complete] calling app.quit");
                                app.quit();
                            } else {
                                trace!("[download::with_progress_gui->progress_complete] couldn't find {} in APP_MAP", &id);
                            }
                            map.remove(&id);
                        }
                    }
                }
            },
            move |_| {}).await;
        debug!("[download::progress_with_gui] thread done :3");
        if let Err(e) = q {
            debug!("encountered error {:#?}", e);
            if let Ok(mut map) = GUI_DL_ERR.write() {
                map.insert(custom_id_clone.clone(), e);
            }
        }
    });

    if let Ok(mut map) = GUI_MAP_COMPLETE.write() {
        map.insert(custom_id.clone(), Some(false));
    }
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    if let Ok(mut map) = APP_MAP.write() {
        map.insert(custom_id.clone(), app);
    }
    let (_, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = DownloadProgressInterface::make_window();
    crate::gui::window_centre_screen(&mut ui.win);
    ui.win.handle(move |w, ev| match ev {
        fltk::enums::Event::Resize => {
            if w.width() > 520 || w.height() > 100 {
                w.set_size(520, 100);
            }
            true
        },
        _ => false
    });
    ui.win.make_resizable(false);
    ui.win.show();
    ui.win.set_label(&format!("{}", title));
    ui.label_left.set_label("Preparing Download");
    ui.label_right.set_label("");
    ui.download_progress.set_minimum(0f64);
    ui.download_progress.set_value(0f64);
    ui.download_progress.set_maximum(100f64);
    let _ =&ui.win.set_callback(move |_| {
        if fltk::app::event() == fltk::enums::Event::Close {
            info!("[with_progress_gui->progress_init->ui.win.set_callback] Close event received, exiting application.");
            sentry::end_session();
            std::process::exit(0);
        }
    });

    let mut running: bool = false;
    if let Ok(map) = GUI_MAP_COMPLETE.read() {
        running = map.get(&custom_id).unwrap_or(&Some(false)).unwrap_or(false) == false;
    }
    while running {
        if let Ok(state_map) = GUI_DL_STATE.read() {
            if let Some(state) = state_map.get(&custom_id) {
                let _ = &ui.download_progress.set_value(state.progress_percentage);
                let _ = &ui.label_left.set_label(&state.label_left);
                let _ = &ui.label_right.set_label(&state.label_right);
                app.redraw();
            }
        }
        match app::wait_for(0f64) {
            Ok(_) => {
                if let Some(action) = receive_action.recv() {
                    match action {
                        GUIAppStatus::Quit => {
                            ui.win.platform_hide();
                            info!("[with_progress_gui->progress_init->receive_action] Told to quit, exiting process");
                            match APP_MAP.read() {
                                Ok(map) => {
                                    if let Some(x) = map.get(&custom_id) {
                                        x.quit();
                                    } else {
                                        info!("[with_progress_gui->progress_init->receive_action] Couldn't find {} in APP_MAP, oh well lol", &custom_id);
                                    }
                                },
                                Err(e) => {
                                    warn!("[with_progress_gui->progress_init->receive_action] Soft failed on APP_MAP.read() {:#?}", e);
                                }
                            }
                            // make sure that we unlock fltk after we're done :3
                            app::unlock();
                            std::process::exit(0);
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                error!("[download::with_progress_gui] failed to app::wait_for {:#?}", e);
            }
        }
        if let Ok(map) = GUI_MAP_COMPLETE.read() {
            running = map.get(&custom_id).unwrap_or(&Some(false)).unwrap_or(false) == false;
        }
    }

    if let Ok(mut map) = GUI_DL_ERR.write() {
        if map.contains_key(&custom_id) {
            if let Some(x) = map.remove(&custom_id) {
                return Err(x);
            }
        }
    }
    return Ok(());
}
/// Download a file with a CLI-based progress bar.
async fn with_progress_cli(url: String, out_location: String) -> Result<(), BeansError> {
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
                map.insert(b.unique_id.clone(), p);
            }
        }, move |c: DownloadCompleteArgs| {
            if let Ok(mut map) = PBAR_MAP.write() {
                if let Some(p) = map.get(&c.unique_id) {
                    p.finish();
                }
                map.remove(&c.unique_id);
            }
        }, move |_: String| {}).await
}
/// Download a file with a custom progress bar. Must be non-blocking or quickly to do stuff!
pub async fn with_progress_custom<FP, FI, FC, FW>(
    url: String,
    out_location: String,
    progress: FP,
    progress_init: FI,
    progress_complete: FC,
    progress_wake: FW) -> Result<(), BeansError>
where FP: Fn(DownloadProgressArgs),
      FI: Fn(DownloadProgressArgs),
      FC: Fn(DownloadCompleteArgs),
      FW: Fn(String)
{
    trace!("[download::with_progress_custom] fetching details from {url}");
    let res = match reqwest::Client::new()
        .get(&url)
        .send()
        .await {
        Ok(v) => v,
        Err(e) => {
            trace!("[download::with_progress_custom] failed on details {:#?}", e);
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
    trace!("[download::with_progress_custom] has length of {total_size}");

    let unique_id = helper::generate_rand_str(32);
    trace!("[download::with_progress_custom] using ID {unique_id}");
    progress_init(DownloadProgressArgs {
        unique_id: unique_id.clone(),
        max: total_size,
        current: 0,
        speed: String::from("0b/s")
    });


    // download chunks
    let mut file = match std::fs::File::create(out_location.clone()) {
        Ok(v) => v,
        Err(e) => {
            trace!("[download::with_progress_custom] failed on File::create() {:#?}", e);
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
    trace!("[download::with_progress_custom] created file thing at {out_location}");
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    let mut last_progress = Instant::now();
    let mut last_progress_size = 0u64;
    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(v) => v,
            Err(e) => {
                trace!("[download::with_progress_custom] failed on stream.next() {:#?}", e);
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
            trace!("[download::with_progress_custom] failed on file.write_all() {:#?}", e);
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

        let time_since_last = last_progress.elapsed();
        let time_since_last_ms = time_since_last.as_millis();
        if time_since_last_ms > 500 {
            let sx = (1000u64 as f64 / time_since_last_ms as f64) * (downloaded as f64 - last_progress_size as f64).round();
            let ssx = match sx > 0.0f64 {
                true => sx as usize,
                false => 0usize
            } as u64;
            // this is a blocking task when using `with_progress_gui` because i don't know how to
            // properly multi-thread with fltk-rs
            // - kate 2024/06/06
            progress(DownloadProgressArgs {
                unique_id: unique_id.clone(),
                max: total_size,
                current: new,
                speed: format!("{}/s", helper::format_size(ssx))
            });
            last_progress_size = downloaded;
            last_progress = Instant::now();
        }
        progress_wake(unique_id.clone());
    }

    trace!("[download::with_progress_custom] completed successfully");
    progress_complete(DownloadCompleteArgs {
        unique_id: unique_id.clone(),
        dl_size: total_size,
        error_content: None
    });
    Ok(())
}