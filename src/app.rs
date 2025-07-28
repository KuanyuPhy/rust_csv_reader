use eframe::{egui, App, CreationContext, Frame};
use egui::{
    CentralPanel, Color32, Context, Margin, Rounding, ScrollArea, Stroke, TopBottomPanel,
    Vec2,
};
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
        // Set up modern dark theme
        self.setup_theme(ctx);
        self.handle_data_response();

        self.render_top_panel(ctx);
        self.render_main_content(ctx);
        self.render_footer(ctx);
    }
}

impl MyApp {
    fn setup_theme(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        // Modern dark theme colors
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(Color32::from_rgb(220, 220, 220));
        style.visuals.panel_fill = Color32::from_rgb(32, 33, 36);
        style.visuals.window_fill = Color32::from_rgb(40, 42, 46);
        style.visuals.extreme_bg_color = Color32::from_rgb(24, 25, 28);
        style.visuals.code_bg_color = Color32::from_rgb(48, 50, 54);

        // Button styling
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(66, 133, 244);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(85, 145, 255);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(51, 103, 214);

        // Rounded corners
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        style.visuals.window_rounding = Rounding::same(12.0);

        // Spacing
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(16.0, 8.0);
        style.spacing.menu_margin = Margin::same(8.0);

        ctx.set_style(style);
    }

    fn render_top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel")
            .exact_height(50.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    // Smaller, more compact open file button
                    let open_btn = egui::Button::new("📁 Open File")
                        .fill(Color32::from_rgb(76, 175, 80))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(56, 142, 60)))
                        .rounding(Rounding::same(8.0));

                    if ui.add_sized([100.0, 32.0], open_btn).clicked() {
                        self.open_file();
                    }

                    ui.add_space(20.0);
                    
                    // Sheet selector
                    self.render_sheet_selector(ui);
                });
                ui.add_space(10.0);
            });
    }

    fn render_sheet_selector(&mut self, ui: &mut egui::Ui) {
        if self.is_excel_file && !self.sheet_names.is_empty() {
            // Simple sheet selector without complex wrappers
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📊 Sheet:").color(Color32::from_rgb(156, 163, 175)));
                ui.add_space(8.0);

                let mut selected_sheet = self.current_sheet;
                let combo = egui::ComboBox::from_id_source("sheet_selector")
                    .selected_text(format!("📄 {}", &self.sheet_names[self.current_sheet]))
                    .width(180.0);

                combo.show_ui(ui, |ui| {
                    for (index, sheet_name) in self.sheet_names.iter().enumerate() {
                        let is_selected = index == self.current_sheet;
                        let text = if is_selected {
                            egui::RichText::new(format!("📄 {}", sheet_name))
                                .color(Color32::from_rgb(66, 133, 244))
                        } else {
                            egui::RichText::new(format!("📄 {}", sheet_name))
                        };

                        ui.selectable_value(&mut selected_sheet, index, text);
                    }
                });

                if selected_sheet != self.current_sheet {
                    self.switch_sheet(selected_sheet);
                }
            });
        }
    }



    fn render_main_content(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                // Modern error display
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    egui::Frame::none()
                        .fill(Color32::from_rgb(220, 53, 69))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠️").size(20.0));
                                ui.label(
                                    egui::RichText::new(error).color(Color32::WHITE).size(14.0),
                                );
                            });
                        });
                });
            } else if self.file_path.is_none() {
                self.render_welcome_screen(ui);
            } else {
                self.render_data_table(ui);
            }
        });
    }

    fn render_welcome_screen(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            // Welcome card
            egui::Frame::none()
                .fill(Color32::from_rgb(48, 50, 54))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(40.0))

                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("📊").size(64.0));

                        ui.add_space(16.0);

                        ui.label(
                            egui::RichText::new("Excel & CSV Reader")
                                .size(24.0)
                                .color(Color32::from_rgb(66, 133, 244)),
                        );

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("Click 'Open File' to load your spreadsheet")
                                .size(14.0)
                                .color(Color32::from_rgb(156, 163, 175)),
                        );

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Supports:")
                                    .size(12.0)
                                    .color(Color32::from_rgb(120, 120, 120)),
                            );
                            ui.label(
                                egui::RichText::new("CSV • XLSX • XLS • ODS")
                                    .size(12.0)
                                    .color(Color32::from_rgb(76, 175, 80)),
                            );
                        });
                    });
                });
        });
    }

    fn render_data_table(&mut self, ui: &mut egui::Ui) {
        // Add padding around the table
        ui.add_space(16.0);

        // Modern table container
        egui::Frame::none()
            .fill(Color32::from_rgb(40, 42, 46))
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::same(16.0))
            .show(ui, |ui| {
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let grid = self.render_table_grid(ui);
                        self.handle_lazy_loading(ui, &grid);
                    });
            });

        ui.add_space(16.0);
        self.render_status_message(ui);
    }

    fn render_table_grid(&self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        egui::Grid::new("table_grid")
            .striped(false) // We'll handle striping manually for better control
            .min_col_width(80.0)
            .max_col_width(250.0)
            .spacing([1.0, 1.0])
            .show(ui, |ui| {
                self.render_table_headers(ui);
                self.render_table_rows(ui);
            })
    }

    fn render_table_headers(&self, ui: &mut egui::Ui) {
        if !self.headers.is_empty() {
            // Index header with special styling - more compact
            egui::Frame::none()
                .fill(Color32::from_rgb(66, 133, 244))
                .rounding(Rounding::same(4.0))
                .inner_margin(Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("#")
                            .color(Color32::WHITE)
                            .strong()
                            .size(12.0),
                    );
                });

            // Column headers - more compact
            for header in &self.headers {
                egui::Frame::none()
                    .fill(Color32::from_rgb(76, 175, 80))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(header)
                                .color(Color32::WHITE)
                                .strong()
                                .size(12.0),
                        );
                    });
            }
            ui.end_row();
        }
    }

    fn render_table_rows(&self, ui: &mut egui::Ui) {
        for (row_index, row) in self.table.iter().enumerate() {
            let is_even = row_index % 2 == 0;
            let row_bg = if is_even {
                Color32::from_rgb(48, 50, 54)
            } else {
                Color32::from_rgb(44, 46, 50)
            };

            // Row index cell - more compact
            egui::Frame::none()
                .fill(row_bg)
                .rounding(Rounding::same(3.0))
                .inner_margin(Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new((row_index + 1).to_string())
                            .color(Color32::from_rgb(156, 163, 175))
                            .size(11.0)
                            .monospace(),
                    );
                });

            // Data cells - more compact
            for (col_index, cell) in row.iter().enumerate() {
                let cell_bg = if col_index % 2 == 0 {
                    row_bg
                } else {
                    Color32::from_rgb(
                        row_bg.r().saturating_add(4),
                        row_bg.g().saturating_add(4),
                        row_bg.b().saturating_add(4),
                    )
                };

                egui::Frame::none()
                    .fill(cell_bg)
                    .rounding(Rounding::same(3.0))
                    .inner_margin(Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        // Truncate long text and add tooltip (Unicode-safe)
                        let display_text = if cell.chars().count() > 50 {
                            let truncated: String = cell.chars().take(47).collect();
                            format!("{}...", truncated)
                        } else {
                            cell.clone()
                        };

                        let label = ui.label(
                            egui::RichText::new(&display_text)
                                .color(Color32::from_rgb(220, 220, 220))
                                .size(12.0),
                        );

                        // Show full text on hover if truncated
                        if cell.len() > 50 {
                            label.on_hover_text(cell);
                        }
                    });
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
        ui.horizontal(|ui| {
            ui.add_space(20.0);

            if self.loading {
                // Modern loading indicator
                egui::Frame::none()
                    .fill(Color32::from_rgb(66, 133, 244))
                    .rounding(Rounding::same(20.0))
                    .inner_margin(Margin::symmetric(16.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new("Loading more data...")
                                    .color(Color32::WHITE)
                                    .size(13.0),
                            );
                        });
                    });
            } else if self.end_of_file && !self.table.is_empty() {
                // Modern end-of-file indicator
                egui::Frame::none()
                    .fill(Color32::from_rgb(76, 175, 80))
                    .rounding(Rounding::same(20.0))
                    .inner_margin(Margin::symmetric(16.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("✅").size(14.0));
                            ui.label(
                                egui::RichText::new("All data loaded")
                                    .color(Color32::WHITE)
                                    .size(13.0),
                            );
                        });
                    });
            }
        });
    }

    fn render_footer(&self, ctx: &Context) {
        if self.file_path.is_some() {
            TopBottomPanel::bottom("footer_panel")
                .exact_height(35.0)
                .show(ctx, |ui| {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        
                        // File information in footer
                        if let Some(path) = &self.file_path {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            let extension = path
                                .extension()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_uppercase();

                            let file_icon = match extension.as_str() {
                                "CSV" => "📊",
                                "XLSX" | "XLS" => "📈",
                                "ODS" => "📋",
                                _ => "📄",
                            };

                            // File name and icon
                            ui.label(
                                egui::RichText::new(format!("{} {}", file_icon, file_name))
                                    .color(Color32::from_rgb(200, 200, 200))
                                    .size(12.0),
                            );

                            ui.separator();

                            // Sheet information
                            if self.is_excel_file && !self.sheet_names.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("Sheet: {}", &self.sheet_names[self.current_sheet]))
                                        .color(Color32::from_rgb(156, 163, 175))
                                        .size(12.0),
                                );
                                ui.separator();
                                
                                ui.label(
                                    egui::RichText::new(format!("{} sheets total", self.sheet_names.len()))
                                        .color(Color32::from_rgb(120, 120, 120))
                                        .size(12.0),
                                );
                                ui.separator();
                            }

                            // Row count
                            if !self.table.is_empty() {
                                let status_text = if self.end_of_file {
                                    format!("{} rows (complete)", self.table.len())
                                } else {
                                    format!("{} rows (loading...)", self.table.len())
                                };
                                
                                ui.label(
                                    egui::RichText::new(status_text)
                                        .color(Color32::from_rgb(120, 120, 120))
                                        .size(12.0),
                                );
                            }
                        }

                        // Push loading status to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(16.0);
                            
                            if self.loading {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        egui::RichText::new("Loading...")
                                            .color(Color32::from_rgb(66, 133, 244))
                                            .size(11.0),
                                    );
                                });
                            } else if self.end_of_file && !self.table.is_empty() {
                                ui.label(
                                    egui::RichText::new("✅ Complete")
                                        .color(Color32::from_rgb(76, 175, 80))
                                        .size(11.0),
                                );
                            }
                        });
                    });
                    ui.add_space(6.0);
                });
        }
    }
}
