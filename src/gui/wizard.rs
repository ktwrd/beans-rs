use fltk::{*, prelude::*};
use crate::{RunnerContext, BeansError};
use crate::gui::GUIAppStatus;
use crate::gui::wizard_ui::WizardInterface;

pub fn run(ctx: &RunnerContext) {
    let app = app::App::default().with_scheme(app::AppScheme::Gtk);
    let (_, receive_action) = app::channel::<GUIAppStatus>();
    let mut ui = WizardInterface::make_window();
    crate::window_ensure!(ui, 640, 250);

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