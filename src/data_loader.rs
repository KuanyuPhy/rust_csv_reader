use egui::Context;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::mpsc;
use tokio::runtime::Runtime;

static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Unable to create Runtime"));

pub type DataLoadResult = Result<(Vec<String>, Vec<Vec<String>>, Vec<String>, bool), String>;

pub struct DataLoader;

impl DataLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load_data_async(
        &self,
        path: PathBuf,
        start_row: usize,
        num_rows: usize,
        sheet_index: usize,
        tx: mpsc::Sender<DataLoadResult>,
        egui_ctx: Context,
    ) {
        RT.spawn(async move {
            let result = load_data_async(path, start_row, num_rows, sheet_index).await;
            tx.send(result).unwrap();
            egui_ctx.request_repaint();
        });
    }
}

async fn load_data_async(
    path: PathBuf,
    start_row: usize,
    num_rows: usize,
    sheet_index: usize,
) -> DataLoadResult {
    let extension = path.extension().and_then(std::ffi::OsStr::to_str);

    match extension {
        Some("csv") => crate::csv_loader::load_csv_data(path, start_row, num_rows).await,
        Some("xlsx") | Some("xls") | Some("ods") => {
            crate::excel_loader::load_excel_data(path, start_row, num_rows, sheet_index).await
        }
        _ => Err("Unsupported file format. Please select a CSV, Excel, or ODS file.".to_string()),
    }
}
