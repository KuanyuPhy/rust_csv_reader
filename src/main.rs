mod app;
mod csv_loader;
mod data_loader;
mod excel_loader;
mod font_setup;

use app::MyApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "CSV/Excel Reader",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
