#[cfg(feature = "gui")]
use fltk::{app::Receiver,
           prelude::*,
           window::Window,
           *};
#[cfg(feature = "gui")]
use fltk_theme::{color_themes,
                 ColorTheme};
#[cfg(feature = "gui")]
use log::debug;

#[cfg(feature = "gui")]
mod dialog;
#[cfg(not(feature = "gui"))]
mod dialog_headless;
#[cfg(feature = "gui")]
pub(crate) mod shared_ui;

#[cfg(feature = "gui")]
pub use dialog::*;
#[cfg(not(feature = "gui"))]
pub use dialog_headless::*;

pub mod icon;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum GUIAppStatus
{
    Update,
    Quit,

    UnknownStatus,

    BtnOk,
    BtnCancel,
    BtnAbort,
    BtnRetry,
    BtnIgnore,
    BtnYes,
    BtnNo,
    BtnTryAgain,
    BtnContinue
}

#[cfg(not(feature = "gui"))]
pub fn get_center_screen() -> (i32, i32)
{(0, 0)}

/// Get the X and Y position of the center of the current screen.
#[cfg(feature = "gui")]
pub fn get_center_screen() -> (i32, i32)
{
    let (px, py) = app::screen_coords();
    let (sw, sh) = app::screen_size();
    (((sw / 2.0) as i32) + px, ((sh / 2.0) as i32) + py)
}

#[cfg(not(feature = "gui"))]
pub fn apply_app_scheme()
{}
#[cfg(feature = "gui")]
pub fn apply_app_scheme()
{
    let theme_content = match dark_light::detect()
    {
        Ok(c) => match c
        {
            dark_light::Mode::Light => color_themes::GRAY_THEME,
            _ => color_themes::DARK_THEME
        },
        Err(e) =>
        {
            log::warn!(
                "[gui::apply_app_scheme] Failed to detect light/dark mode\n{:#?}",
                e
            );
            color_themes::DARK_THEME
        }
    };
    debug!(
        "[apply_app_scheme] using color theme: {:#?}",
        dark_light::detect()
    );
    let theme = ColorTheme::new(theme_content);
    theme.apply();
}

#[cfg(feature = "gui")]
pub(crate) fn wait_for_quit(
    app: &app::App,
    receive_action: &Receiver<GUIAppStatus>
)
{
    while app.wait()
    {
        if let Some(GUIAppStatus::Quit) = receive_action.recv()
        {
            unsafe {
                crate::PAUSE_ONCE_DONE = false;
            }
            app.quit();
        }
    }
}
