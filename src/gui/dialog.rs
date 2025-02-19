use fltk::{image::PngImage,
           prelude::*,
           text::TextBuffer,
           *};
use fltk::window::Window;
use log::warn;

use crate::gui::{apply_app_scheme,
                 icon,
                 shared_ui::GenericDialog,
                 wait_for_quit,
                 GUIAppStatus};

pub struct DialogBuilder
{
    pub title: String,
    pub content: String,
    pub icon: Option<PngImage>
}

pub enum DialogIconKind
{
    Default,
    Warn,
    Error
}

impl Default for DialogBuilder
{
    fn default() -> Self
    {
        Self {
            title: format!("beans v{}", crate::VERSION),
            content: String::new(),
            icon: None
        }
    }
}

impl DialogBuilder
{
    pub fn new() -> Self
    {
        Self::default()
    }
    pub fn with_png_data(
        mut self,
        data: &[u8]
    ) -> Self
    {
        match PngImage::from_data(data)
        {
            Ok(img) => self.icon = Some(img),
            Err(e) =>
            {
                warn!("[DialogBuilder::with_png] Failed to set icon! {:#?}", e);
            }
        }
        self
    }
    pub fn with_icon(
        self,
        kind: DialogIconKind
    ) -> Self
    {
        let data: &Vec<u8> = match kind
        {
            DialogIconKind::Default => &icon::DEFAULT_RAW_X32,
            DialogIconKind::Warn => &icon::DEFAULT_WARN_RAW_X32,
            DialogIconKind::Error => &icon::DEFAULT_ERROR_RAW_X32
        };
        self.with_png_data(data)
    }
    pub fn with_title(
        mut self,
        content: String
    ) -> Self
    {
        self.title = content.clone();
        self
    }
    pub fn with_content(
        mut self,
        content: String
    ) -> Self
    {
        self.content = content.clone();
        self
    }
    pub fn run(&self)
    {
        if !crate::has_gui_support()
        {
            println!("============ {} ============", self.title);
            println!("{}", self.content);
            return;
        }

        let app = app::App::default().with_scheme(app::AppScheme::Gtk);
        apply_app_scheme();
        let (send_action, receive_action) = app::channel::<GUIAppStatus>();
        let mut ui = GenericDialog::make_window();
        let mut text_buffer = TextBuffer::default();
        text_buffer.append(&self.content);
        ui.txt_disp.set_buffer(text_buffer.clone());
        ui.win.set_icon(self.icon.clone());

        ui.win.set_label(&self.title);
        ui.btn_ok.emit(send_action, GUIAppStatus::Quit);
        window_centre_screen(&mut ui.win);
        ui.win.make_resizable(false);
        ui.win.show();
        wait_for_quit(&app, &receive_action);
    }
}


/// Make the `window` provided the in be the center of the current screen.
fn window_centre_screen(window: &mut Window)
{
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
        height
    );
}