#![windows_subsystem = "windows"]

mod app;
mod hex;
mod logging;
mod serial;
mod ui;
mod virtual_com;

use eframe::egui;
use app::ComAnalyzerApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_title("RustCOM"),
        ..Default::default()
    };

    eframe::run_native(
        "RustCOM",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ComAnalyzerApp::default()))
        }),
    )
}
