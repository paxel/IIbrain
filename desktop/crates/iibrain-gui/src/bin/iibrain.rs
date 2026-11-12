//! The IIbrain desktop app.

use std::time::Duration;

use iibrain_gui::{APP_ID, App, l10n};

/// How often the window takes in what other tools changed in the
/// Brain's folder.
const REFRESH: Duration = Duration::from_secs(2);

struct Window {
    app: App,
}

impl eframe::App for Window {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.refresh_if_due(REFRESH);
        ui.ctx().request_repaint_after(REFRESH);
        self.app.show(ui);
    }
}

fn main() -> eframe::Result {
    let locale = l10n::locale_for_system(sys_locale::get_locale().as_deref());
    let device_dir = directories::ProjectDirs::from("io.github", "paxel", "iibrain")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(".iibrain-device"));
    let app = App::new(&device_dir, locale);
    let viewport = egui::ViewportBuilder::default()
        .with_title(app.t().app_title())
        // The app id a launcher entry is named after.
        .with_app_id(APP_ID)
        .with_inner_size([1100.0, 720.0])
        .with_min_inner_size([640.0, 400.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "IIbrain",
        options,
        Box::new(move |_cc| Ok(Box::new(Window { app }))),
    )
}
