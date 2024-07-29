use fltk::app::Receiver;
use fltk::window::Window;
use fltk::{prelude::*, *};
use fltk_theme::{color_themes, ColorTheme};
use log::debug;

mod dialog;
pub(crate) mod shared_ui;

pub use dialog::*;

pub mod icon;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum GUIAppStatus {
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
    BtnContinue,
}

/// Make the `window` provided the in be the center of the current screen.
pub fn window_centre_screen(window: &mut Window) {
    let (sx, sy) = app::screen_coords();
    let width = window.width();
    let height = window.height();
    let (mut x, mut y) = app::screen_size();
    x -= width as f64;
    y -= height as f64;
    window.resize(
        ((x / 2.0) as i32) + sx,
        ((y / 2.0) as i32) + sy,
        width,
        height,
    );
}

/// Get the X and Y position of the center of the current screen.
pub fn get_center_screen() -> (i32, i32) {
    let (px, py) = app::screen_coords();
    let (sw, sh) = app::screen_size();
    (((sw / 2.0) as i32) + px, ((sh / 2.0) as i32) + py)
}

/// Ensure that a window has a fixed width & height, and that it will appear in the centre of the
/// current screen.
pub fn window_ensure(win: &mut Window, width: i32, height: i32) {
    window_centre_screen(win);
    win.handle(move |w, ev| match ev
    {
        fltk::enums::Event::Resize =>
        {
            if w.width() > width || w.height() > height
            {
                w.set_size(width, height);
            }
            true
        }
        _ => false,
    });
    win.make_resizable(false);
    win.show();
}

pub fn apply_app_scheme() {
    let theme_content = match dark_light::detect()
    {
        dark_light::Mode::Light => color_themes::GRAY_THEME,
        _ => color_themes::DARK_THEME,
    };
    debug!(
        "[apply_app_scheme] using color theme: {:#?}",
        dark_light::detect()
    );
    let theme = ColorTheme::new(theme_content);
    theme.apply();
}

pub fn wait_for_quit(app: &app::App, receive_action: &Receiver<GUIAppStatus>) {
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
