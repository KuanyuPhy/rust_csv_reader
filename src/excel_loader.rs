use std::path::PathBuf;
use crate::data_loader::DataLoadResult;

pub async fn load_excel_data(
    path: PathBuf,
    start_row: usize,
    num_rows: usize,
    sheet_index: usize,
) -> DataLoadResult {
    use calamine::{open_workbook_auto, Reader};

    // Excel operations are CPU-bound, so we run them in a blocking task
    let result = tokio::task::spawn_blocking(move || {
        let mut workbook =
            open_workbook_auto(&path).map_err(|e| format!("Failed to open Excel file: {}", e))?;

        // Get the first worksheet
        let sheet_names = workbook.sheet_names().to_owned();
        if sheet_names.is_empty() {
            return Err("No worksheets found in the file".to_string());
        }

        // Use the specified sheet index, fallback to first sheet if index is invalid
        let sheet_index = if sheet_index < sheet_names.len() {
            sheet_index
        } else {
            0
        };
        let sheet_name = &sheet_names[sheet_index];
        let range = workbook.worksheet_range(sheet_name).map_err(|e| {
            format!("Failed to read worksheet '{}': {}", sheet_name, e)
        })?;

        let mut headers = Vec::new();
        let mut data = Vec::new();

        // Convert range to rows
        let rows: Vec<Vec<_>> = range.rows().map(|row| row.to_vec()).collect();

        if rows.is_empty() {
            return Ok((headers, data, sheet_names, true));
        }

        // Extract headers if this is the first load
        if start_row == 0 && !rows.is_empty() {
            headers = rows[0].iter().map(|cell| cell.to_string()).collect();
        }

        // Calculate the actual start row (skip header if start_row == 0)
        let data_start = if start_row == 0 { 1 } else { start_row + 1 };
        let data_end = std::cmp::min(data_start + num_rows, rows.len());

        // Extract the requested rows
        for row_idx in data_start..data_end {
            if row_idx < rows.len() {
                let row_data: Vec<String> = rows[row_idx]
                    .iter()
                    .map(|cell| cell.to_string())
                    .collect();
                data.push(row_data);
            }
        }

        // Check if we've reached the end of the sheet
        let end_of_file = data_end >= rows.len();

        Ok((headers, data, sheet_names, end_of_file))
    })
    .await;

    result.map_err(|e| format!("Task execution error: {}", e))?
}