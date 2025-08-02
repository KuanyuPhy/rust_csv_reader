mod app;
mod csv_loader;
mod data_loader;
mod excel_loader;
mod font_setup;

use app::MyApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])  // 初始視窗大小
            .with_min_inner_size([800.0, 600.0])  // 最小視窗大小
            .with_resizable(true),  // 可調整大小
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "CSV/Excel Reader",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
