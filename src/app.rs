use eframe::{egui, App, CreationContext, Frame};
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel};
use std::path::PathBuf;
use std::sync::mpsc;

use crate::data_loader::{DataLoadResult, DataLoader};
use crate::font_setup::setup_custom_fonts;

pub struct MyApp {
    table: Vec<Vec<String>>,
    headers: Vec<String>,
    error: Option<String>,
    file_path: Option<PathBuf>,
    tx: mpsc::Sender<DataLoadResult>,
    rx: mpsc::Receiver<DataLoadResult>,
    egui_ctx: Context,
    loading: bool,
    rows_to_show: usize,
    headers_loaded: bool,
    sheet_names: Vec<String>,
    current_sheet: usize,
    is_excel_file: bool,
    end_of_file: bool,
    data_loader: DataLoader,
}

impl MyApp {
    pub fn new(cc: &CreationContext) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        
        let (tx, rx) = mpsc::channel();
        Self {
            table: Vec::new(),
            headers: Vec::new(),
            error: None,
            file_path: None,
            tx,
            rx,
            egui_ctx: cc.egui_ctx.clone(),
            loading: false,
            rows_to_show: 100,
            headers_loaded: false,
            sheet_names: Vec::new(),
            current_sheet: 0,
            is_excel_file: false,
            end_of_file: false,
            data_loader: DataLoader::new(),
        }
    }

    fn open_file(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Spreadsheet", &["csv", "xlsx", "xls", "ods"])
            .pick_file();

        if let Some(path) = file {
            self.reset_state();
            self.file_path = Some(path);
            self.load_more_data();
        }
    }

    fn reset_state(&mut self) {
        self.table.clear();
        self.headers.clear();
        self.headers_loaded = false;
        self.sheet_names.clear();
        self.current_sheet = 0;
        self.is_excel_file = false;
        self.error = None;
        self.end_of_file = false;
        self.rows_to_show = 100;
    }

    fn load_more_data(&mut self) {
        if self.loading || self.file_path.is_none() || self.end_of_file {
            return;
        }

        self.loading = true;
        let path = self.file_path.as_ref().unwrap().clone();
        let start_row = self.table.len();
        let num_rows = self.rows_to_show - self.table.len();
        let sheet_index = self.current_sheet;

        self.data_loader.load_data_async(
            path,
            start_row,
            num_rows,
            sheet_index,
            self.tx.clone(),
            self.egui_ctx.clone(),
        );
    }

    fn switch_sheet(&mut self, sheet_index: usize) {
        if sheet_index != self.current_sheet && sheet_index < self.sheet_names.len() {
            self.current_sheet = sheet_index;
            self.table.clear();
            self.headers.clear();
            self.headers_loaded = false;
            self.end_of_file = false;
            self.rows_to_show = 100;
            self.load_more_data();
        }
    }

    fn handle_data_response(&mut self) {
        if let Ok(result) = self.rx.try_recv() {
            self.loading = false;
            match result {
                Ok((headers, data, sheet_names, end_of_file)) => {
                    self.update_sheet_info(sheet_names);
                    self.update_headers(headers);
                    self.table.extend(data);
                    self.end_of_file = end_of_file;
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }
    }

    fn update_sheet_info(&mut self, sheet_names: Vec<String>) {
        if self.sheet_names.is_empty() && !sheet_names.is_empty() {
            self.sheet_names = sheet_names;
            self.is_excel_file = self.sheet_names.len() > 1 || self.sheet_names[0] != "CSV";
        }
    }

    fn update_headers(&mut self, headers: Vec<String>) {
        if !self.headers_loaded && !headers.is_empty() {
            self.headers = headers;
            self.headers_loaded = true;
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.handle_data_response();
        
        self.render_top_panel(ctx);
        self.render_main_content(ctx);
    }
}

impl MyApp {
    fn render_top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open File").clicked() {
                    self.open_file();
                }

                self.render_sheet_selector(ui);
                self.render_file_info(ui);
            });
        });
    }

    fn render_sheet_selector(&mut self, ui: &mut egui::Ui) {
        if self.is_excel_file && !self.sheet_names.is_empty() {
            ui.separator();
            ui.label("Sheet:");

            let mut selected_sheet = self.current_sheet;
            egui::ComboBox::from_label("")
                .selected_text(&self.sheet_names[self.current_sheet])
                .show_ui(ui, |ui| {
                    for (index, sheet_name) in self.sheet_names.iter().enumerate() {
                        ui.selectable_value(&mut selected_sheet, index, sheet_name);
                    }
                });

            if selected_sheet != self.current_sheet {
                self.switch_sheet(selected_sheet);
            }
        }
    }

    fn render_file_info(&self, ui: &mut egui::Ui) {
        if let Some(path) = &self.file_path {
            ui.separator();
            ui.label(format!(
                "File: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));

            if self.is_excel_file && !self.sheet_names.is_empty() {
                ui.label(format!("({} sheets)", self.sheet_names.len()));
            }
        }
    }

    fn render_main_content(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                self.render_data_table(ui);
            }
        });
    }

    fn render_data_table(&mut self, ui: &mut egui::Ui) {
        ScrollArea::both().show(ui, |ui| {
            let grid = self.render_table_grid(ui);
            self.handle_lazy_loading(ui, &grid);
            self.render_status_message(ui);
        });
    }

    fn render_table_grid(&self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        egui::Grid::new("table_grid")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                self.render_table_headers(ui);
                self.render_table_rows(ui);
            })
    }

    fn render_table_headers(&self, ui: &mut egui::Ui) {
        if !self.headers.is_empty() {
            ui.strong("Index");
            for header in &self.headers {
                ui.strong(header);
            }
            ui.end_row();
        }
    }

    fn render_table_rows(&self, ui: &mut egui::Ui) {
        for (row_index, row) in self.table.iter().enumerate() {
            ui.label((row_index + 1).to_string());
            for cell in row {
                ui.label(cell);
            }
            ui.end_row();
        }
    }

    fn handle_lazy_loading(&mut self, ui: &mut egui::Ui, grid: &egui::InnerResponse<()>) {
        if ui.clip_rect().bottom() >= grid.response.rect.bottom() - 10.0
            && !self.loading
            && !self.end_of_file
        {
            self.rows_to_show += 100;
            self.load_more_data();
        }
    }

    fn render_status_message(&self, ui: &mut egui::Ui) {
        if self.loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading more data...");
            });
        } else if self.end_of_file && !self.table.is_empty() {
            ui.horizontal(|ui| {
                ui.label("📄 End of file reached");
            });
        }
    }
}