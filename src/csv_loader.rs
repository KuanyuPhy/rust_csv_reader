use std::path::PathBuf;
use futures::stream::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use crate::data_loader::DataLoadResult;

pub async fn load_csv_data(
    path: PathBuf,
    start_row: usize,
    num_rows: usize,
) -> DataLoadResult {
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| e.to_string())?;
    let compat_file = file.compat();
    let mut rdr = csv_async::AsyncReader::from_reader(compat_file);

    let mut headers = Vec::new();
    let mut data = Vec::new();
    let mut current_row = 0;

    // Read headers if this is the first load (start_row == 0)
    if start_row == 0 {
        if let Some(record) = rdr.records().next().await {
            let record = record.map_err(|e| e.to_string())?;
            headers = record.iter().map(|s| s.to_string()).collect();
            current_row = 1;
        }
    }

    let mut records = rdr.records();

    // Skip to the start row (accounting for header)
    let skip_rows = if start_row == 0 { 0 } else { start_row };
    for _ in current_row..skip_rows {
        if records.next().await.is_none() {
            return Ok((headers, data, vec!["CSV".to_string()], true)); // Reached end of file
        }
    }

    // Read the requested number of rows
    let mut end_of_file = false;
    for _ in 0..num_rows {
        if let Some(record) = records.next().await {
            let record = record.map_err(|e| e.to_string())?;
            data.push(record.iter().map(|s| s.to_string()).collect());
        } else {
            end_of_file = true;
            break; // Reached end of file
        }
    }

    Ok((headers, data, vec!["CSV".to_string()], end_of_file))
}