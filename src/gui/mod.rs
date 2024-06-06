use fltk::prelude::WidgetExt;
use fltk::window::Window;

pub mod download_ui;
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum GUIAppStatus {
    Update,
    Quit,
}
/// Make the `window` provided the in be the center of the current screen.
pub fn window_centre_screen(window: &mut Window) {
    let (sx, sy) = fltk::app::screen_coords();
    let width = window.width();
    let height = window.height();
    let (mut x, mut y) = fltk::app::screen_size().clone();
    x -= width.clone() as f64;
    y -= height.clone() as f64;
    window.resize(((x / 2.0) as i32) + sx, ((y / 2.0) as i32) + sy, width, height);
}