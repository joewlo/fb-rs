use anyhow::Result;
use clap::ValueEnum;

/// Output format for command results.
#[derive(Copy, Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}

/// Trait for types that can render themselves as table, JSON, or CSV.
pub trait OutputFormatter {
    fn format_table(&self) -> Result<String>;
    fn format_json(&self) -> Result<String>;
    fn format_csv(&self) -> Result<String>;

    fn format(&self, fmt: &OutputFormat) -> Result<String> {
        match fmt {
            OutputFormat::Table => self.format_table(),
            OutputFormat::Json => self.format_json(),
            OutputFormat::Csv => self.format_csv(),
        }
    }
}

/// Collect all unique keys from a slice of JSON objects.
fn collect_headers(items: &[serde_json::Value]) -> Vec<String> {
    let mut headers: Vec<String> = Vec::new();
    for item in items {
        if let serde_json::Value::Object(map) = item {
            for key in map.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }
    headers
}

/// Extract a value from a JSON object as a string.
fn cell_value(item: &serde_json::Value, key: &str) -> String {
    if let serde_json::Value::Object(map) = item {
        if let Some(val) = map.get(key) {
            return match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "-".to_string(),
                other => other.to_string(),
            };
        }
    }
    String::new()
}

impl OutputFormatter for Vec<serde_json::Value> {
    fn format_table(&self) -> Result<String> {
        if self.is_empty() {
            return Ok("No results.".to_string());
        }

        let headers = collect_headers(self);
        if headers.is_empty() {
            return Ok("No columns.".to_string());
        }

        // Compute column widths.
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for item in self {
            for (i, h) in headers.iter().enumerate() {
                let val_len = cell_value(item, h).len();
                if val_len > widths[i] {
                    widths[i] = val_len;
                }
            }
        }

        let mut out = String::new();

        // Top border
        out.push('+');
        for w in &widths {
            out.push_str(&"-".repeat(w + 2));
            out.push('+');
        }
        out.push('\n');

        // Header row
        out.push('|');
        for (i, h) in headers.iter().enumerate() {
            out.push_str(&format!(" {:<width$} |", h, width = widths[i]));
        }
        out.push('\n');

        // Header separator
        out.push('+');
        for w in &widths {
            out.push_str(&"=".repeat(w + 2));
            out.push('+');
        }
        out.push('\n');

        // Data rows
        for item in self {
            out.push('|');
            for (i, h) in headers.iter().enumerate() {
                out.push_str(&format!(" {:<width$} |", cell_value(item, h), width = widths[i]));
            }
            out.push('\n');
        }

        // Bottom border
        out.push('+');
        for w in &widths {
            out.push_str(&"-".repeat(w + 2));
            out.push('+');
        }
        out.push('\n');

        Ok(out)
    }

    fn format_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    fn format_csv(&self) -> Result<String> {
        if self.is_empty() {
            return Ok(String::new());
        }

        let headers = collect_headers(self);
        let mut buf = Vec::new();
        {
            let mut wtr = csv::Writer::from_writer(&mut buf);
            wtr.write_record(&headers)?;
            for item in self {
                let row: Vec<String> =
                    headers.iter().map(|h| cell_value(item, h)).collect();
                wtr.write_record(&row)?;
            }
            wtr.flush()?;
        }
        Ok(String::from_utf8(buf)?)
    }
}

/// Print a value that implements `OutputFormatter` to stdout.
pub fn print_output<T: OutputFormatter>(data: &T, fmt: &OutputFormat) -> Result<()> {
    let output = data.format(fmt)?;
    print!("{}", output);
    Ok(())
}
