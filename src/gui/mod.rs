use fltk::{*, prelude::*};
use fltk::app::Receiver;
use fltk::window::Window;
use fltk_theme::{color_themes, ColorTheme};

pub mod download_ui;
pub mod wizard_ui;
pub mod wizard;
pub mod install_confirm;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum GUIAppStatus {
    Update,
    Quit,

    BtnContinue,
    BtnCancel,
    BtnYes,
    BtnNo,
    BtnOk,
    BtnBack,

    WizardBtnInstall,
    WizardBtnUpdate
}
/// Make the `window` provided the in be the center of the current screen.
pub fn window_centre_screen(window: &mut Window) {
    let (sx, sy) = app::screen_coords();
    let width = window.width();
    let height = window.height();
    let (mut x, mut y) = app::screen_size().clone();
    x -= width.clone() as f64;
    y -= height.clone() as f64;
    window.resize(((x / 2.0) as i32) + sx, ((y / 2.0) as i32) + sy, width, height);
}

/// Ensure that a window has a fixed width & height, and that it will appear in the centre of the
/// current screen.
pub fn window_ensure(win: &mut Window, width: i32, height: i32) {
    crate::gui::window_centre_screen(win);
    win.handle(move |w, ev| match ev {
        fltk::enums::Event::Resize => {
            if w.width() > width || w.height() > height {
                w.set_size(width, height);
            }
            true
        },
        _ => false
    });
    win.make_resizable(false);
    win.show();
}
pub fn apply_app_scheme() {
    let theme = ColorTheme::new(color_themes::DARK_THEME);
    theme.apply();
}
pub fn wait_for_quit(app: &app::App, receive_action: &Receiver<GUIAppStatus>) {
    while app.wait() {
        if let Some(action) = receive_action.recv() {
            match action {
                GUIAppStatus::Quit => {
                    app.quit();
                },
                _ => {}
            }
        }
    }
}