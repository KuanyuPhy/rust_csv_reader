use std::sync::mpsc;
use eframe::{egui, App, Frame, CreationContext};
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use futures::stream::{StreamExt};
use once_cell::sync::Lazy;
use tokio_util::compat::TokioAsyncReadCompatExt;

static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Unable to create Runtime"));

fn main() -> Result<(), eframe::Error> {
    let _enter = RT.enter();
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "CSV/Excel Reader",
        options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}

struct MyApp {
    table: Vec<Vec<String>>,
    error: Option<String>,
    file_path: Option<PathBuf>,
    tx: mpsc::Sender<Result<Vec<Vec<String>>, String>>,
    rx: mpsc::Receiver<Result<Vec<Vec<String>>, String>>,
    egui_ctx: Context,
    loading: bool,
    rows_to_show: usize,
}

impl MyApp {
    fn new(cc: &CreationContext) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            table: Vec::new(),
            error: None,
            file_path: None,
            tx,
            rx,
            egui_ctx: cc.egui_ctx.clone(),
            loading: false,
            rows_to_show: 100, // Initially show 100 rows
        }
    }

    fn open_file(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Spreadsheet", &["csv", "xlsx", "xls", "ods"])
            .pick_file();

        if let Some(path) = file {
            self.table.clear();
            self.file_path = Some(path);
            self.rows_to_show = 100;
            self.load_more_data();
        }
    }

    fn load_more_data(&mut self) {
        if self.loading || self.file_path.is_none() {
            return;
        }

        self.loading = true;
        let path = self.file_path.as_ref().unwrap().clone();
        let tx = self.tx.clone();
        let egui_ctx = self.egui_ctx.clone();
        let start_row = self.table.len();
        let num_rows = self.rows_to_show - self.table.len();

        RT.spawn(async move {
            let result = load_data_async(path, start_row, num_rows).await;
            tx.send(result).unwrap();
            egui_ctx.request_repaint();
        });
    }
}

async fn load_data_async(path: PathBuf, start_row: usize, num_rows: usize) -> Result<Vec<Vec<String>>, String> {
    let extension = path.extension().and_then(std::ffi::OsStr::to_str);
    if extension == Some("csv") {
        let file = tokio::fs::File::open(path).await.map_err(|e| e.to_string())?;
        let compat_file = file.compat();
        let mut rdr = csv_async::AsyncReader::from_reader(compat_file);
        let mut records = rdr.records();
        let mut data = Vec::new();
        for _ in 0..start_row {
            if records.next().await.is_none() {
                return Ok(data); // Reached end of file
            }
        }
        for _ in 0..num_rows {
            if let Some(record) = records.next().await {
                let record = record.map_err(|e| e.to_string())?;
                data.push(record.iter().map(|s| s.to_string()).collect());
            } else {
                break; // Reached end of file
            }
        }
        Ok(data)
    } else {
        Err("Only CSV files are supported for async loading in this example.".to_string())
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if let Ok(result) = self.rx.try_recv() {
            self.loading = false;
            match result {
                Ok(data) => {
                    self.table.extend(data);
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open File").clicked() {
                    self.open_file();
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    let grid = egui::Grid::new("table_grid").striped(true).show(ui, |ui| {
                        for row in &self.table {
                            for cell in row {
                                ui.label(cell);
                            }
                            ui.end_row();
                        }
                    });

                    // Simple lazy loading trigger
                    if ui.clip_rect().bottom() >= grid.response.rect.bottom() - 10.0 && !self.loading {
                        self.rows_to_show += 100;
                        self.load_more_data();
                    }
                });
            }
        });
    }
}