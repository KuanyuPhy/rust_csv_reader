use crate::data_loader::DataLoadResult;
use std::path::PathBuf;
use tokio::task;
use regex::Regex;

pub async fn load_csv_data(path: PathBuf, start_row: usize, num_rows: usize) -> DataLoadResult {
    let result = task::spawn_blocking(move || {
        load_csv_data_sync(path, start_row, num_rows)
    }).await;

    result.map_err(|e| format!("任務執行錯誤: {}", e))?
}

fn load_csv_data_sync(path: PathBuf, start_row: usize, num_rows: usize) -> DataLoadResult {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path(&path)
        .map_err(|e| e.to_string())?;

    let mut headers = Vec::new();
    let mut data = Vec::new();
    let mut data_start_line = 0;

    if start_row == 0 {
        let file_structure = detect_file_structure(&mut rdr)?;

        match file_structure {
            FileStructure::Simple => {
                data_start_line = 0;
            },
            FileStructure::Mixed { header_line } => {
                data_start_line = header_line;
            }
        }

        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(&path)
            .map_err(|e| e.to_string())?;

        // Fix 1: Add underscore to `i`
        for _i in 0..data_start_line {
            rdr.records().next();
        }

        if let Some(record) = rdr.records().next() {
            let record = record.map_err(|e| e.to_string())?;
            headers = record.iter().map(|s| s.to_string()).collect();
        }
    }

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path(&path)
        .map_err(|e| e.to_string())?;

    let skip_to_data = if start_row == 0 {
        data_start_line + 1
    } else {
        data_start_line + 1 + start_row
    };

    // Fix 2: Add underscore to `i`
    for _i in 0..skip_to_data {
        rdr.records().next();
    }

    let mut end_of_file = false;
    // Fix 3: Add underscore to `i`
    for _i in 0..num_rows {
        if let Some(record) = rdr.records().next() {
            let record = record.map_err(|e| e.to_string())?;
            data.push(record.iter().map(|s| s.to_string()).collect());
        } else {
            end_of_file = true;
            break;
        }
    }

    Ok((headers, data, vec!["CSV".to_string()], end_of_file))
}

#[derive(Debug)]
enum FileStructure {
    Simple,
    Mixed { header_line: usize },
}

fn detect_file_structure(rdr: &mut csv::Reader<std::fs::File>) -> Result<FileStructure, String> {
    let mut line_count = 0;
    let mut sample_lines = Vec::new();
    let max_sample_lines = 50;

    for record in rdr.records() {
        if line_count >= max_sample_lines {
            break;
        }

        let record = record.map_err(|e| e.to_string())?;
        let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        sample_lines.push((line_count, fields));

        line_count += 1;
    }

    if sample_lines.is_empty() {
        return Err("檔案為空".to_string());
    }

    let is_mixed = is_mixed_structure(&sample_lines);

    if is_mixed {
        if let Some(header_line) = find_header_with_regex(&sample_lines) {
            return Ok(FileStructure::Mixed { header_line });
        }
    }

    Ok(FileStructure::Simple)
}

fn is_mixed_structure(sample_lines: &[(usize, Vec<String>)]) -> bool {
    if sample_lines.len() < 3 {
        return false;
    }

    let first_line_fields = sample_lines[0].1.len();
    let second_line_fields = sample_lines[1].1.len();
    let third_line_fields = sample_lines[2].1.len();

    let max_fields = first_line_fields.max(second_line_fields).max(third_line_fields);
    let min_fields = first_line_fields.min(second_line_fields).min(third_line_fields);

    if max_fields > min_fields * 2 {
        return true;
    }

    for (_, fields) in sample_lines[..3].iter() {
        if fields.len() <= 2 {
            if fields.len() == 2 {
                let first_field = fields[0].to_lowercase();

                if first_field.contains("user") ||
                    first_field.contains("supplier") ||
                    first_field.contains("wafer") ||
                    first_field.contains("led") ||
                    first_field.contains("date") ||
                    first_field.contains("time") ||
                    first_field.contains("version") ||
                    first_field.contains("id") {
                    return true;
                }
            }
        }
    }

    for (_, fields) in sample_lines[..3].iter() {
        if fields.len() == 1 {
            let field = fields[0].to_lowercase();
            if field.len() == 1 && field.chars().next().unwrap().is_alphabetic() {
                return true;
            }
        }
    }

    false
}

fn find_header_with_regex(sample_lines: &[(usize, Vec<String>)]) -> Option<usize> {
    let index_pattern = Regex::new(r"INDEX").unwrap();
    let underscore_pattern = Regex::new(r".*_.*").unwrap();

    for (line_num, fields) in sample_lines {
        let mut pattern_matches = 0;
        let mut keyword_matches = 0;
        let mut underscore_count = 0;

        if fields.len() < 5 {
            continue;
        }

        for field in fields {
            let field_upper = field.to_uppercase();
            let field_lower = field.to_lowercase();

            if index_pattern.is_match(&field_upper) {
                pattern_matches += 3;
            }

            if underscore_pattern.is_match(field) {
                pattern_matches += 1;
                underscore_count += 1;
            }

            let data_keywords = ["index", "upl", "epi", "aoi", "chip", "wp", "wd", "fwhm"];
            for keyword in &data_keywords {
                if field_lower.contains(keyword) {
                    keyword_matches += 1;
                }
            }
        }

        let is_likely_header =
            fields.len() >= 10 &&
            underscore_count >= 5 &&
            (pattern_matches >= 5 || keyword_matches >= 5);

        if is_likely_header {
            return Some(*line_num);
        }
    }

    None
}