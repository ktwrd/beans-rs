use fltk::{*, prelude::*};
use fltk::image::PngImage;
use log::warn;
use crate::gui::{apply_app_scheme, window_centre_screen, wait_for_quit, GUIAppStatus, icon};
use crate::gui::shared_ui::GenericDialog;

pub struct DialogBuilder {
    pub title: String,
    pub content: String,
    pub icon: Option<PngImage>
}
pub enum DialogIconKind {
    Default,
    Warn,
    Error
}
impl DialogBuilder {
    pub fn new() -> Self {
        Self {
            title: format!("beans v{}", crate::VERSION),
            content: String::new(),
            icon: None
        }
    }
    pub fn with_png_data(mut self, data: &Vec<u8>) -> Self {
        match PngImage::from_data(data) {
            Ok(img) => {
                self.icon = Some(img)
            },
            Err(e) => {
                warn!("[DialogBuilder::with_png] Failed to set icon! {:#?}", e);
            }
        }
        return self;
    }
    pub fn with_icon(self, kind: DialogIconKind) -> Self {
        let data: &Vec<u8> = match kind {
            DialogIconKind::Default => &icon::DEFAULT_RAW_X32,
            DialogIconKind::Warn => &icon::DEFAULT_WARN_RAW_X32,
            DialogIconKind::Error => &icon::DEFAULT_ERROR_RAW_X32
        };
        self.with_png_data(data)
    }
    pub fn with_title(mut self, content: String) -> Self {
        self.title = content.clone();
        return self;
    }
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content.clone();
        self
    }
    pub fn run(&self) {
        if crate::has_gui_support() == false {
            println!("============ {} ============", self.title);
            println!("{}", self.content);
            return;
        }

        let app = app::App::default().with_scheme(app::AppScheme::Gtk);
        apply_app_scheme();
        let (send_action, receive_action) = app::channel::<GUIAppStatus>();
        let mut ui = GenericDialog::make_window();
        let initial_width = ui.win.width();
        ui.win.set_icon(self.icon.clone());

        ui.win.set_label(&self.title);
        ui.label.set_label(&self.content);
        ui.btn_ok.set_size(70, 24);
        ui.btn_ok.emit(send_action, GUIAppStatus::Quit);

        let (label_w, label_h) = ui.label.measure_label();
        ui.win.set_size(
            25 + label_w + 25,
            10 + label_h + 5 + ui.btn_ok.height() + 5
        );

        ui.btn_ok.set_pos(25, ui.win.height() - 24 - 5);
        window_centre_screen(&mut ui.win);
        ui.win.handle(move |w, ev|
            match ev {
                fltk::enums::Event::Resize => {
                    let height = w.height();
                    ui.btn_ok.set_pos(25, height - 24 - 5);
                    ui.btn_ok.set_size(70, 24);
                    let (lw, lh) = ui.label.measure_label();
                    let cw = w.width();
                    if cw != initial_width {
                        if cw > lw+50 {
                            w.set_size(lw+50, 10+lh+5+ ui.btn_ok.height() + 5);
                        }
                    }
                    false
                },
                _ => false
            });
        ui.win.make_resizable(false);
        ui.win.show();
        wait_for_quit(&app, &receive_action);
    }
}