//! Data export functionality for performance metrics
//!
//! Supports CSV, JSON, and SQLite export formats.

use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use crate::core::metrics::SystemMetrics;

/// Export file format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Comma-separated values format
    Csv,
    /// JavaScript Object Notation format
    Json,
    /// SQLite database format
    Sqlite,
}

impl ExportFormat {
    /// Returns the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Sqlite => "db",
        }
    }

    /// Returns the human-readable name for this format
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV (Comma Separated Values)",
            ExportFormat::Json => "JSON (JavaScript Object Notation)",
            ExportFormat::Sqlite => "SQLite Database",
        }
    }
}

/// Metric data point for export
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: u64,
    pub metric_name: String,
    pub value: f32,
}

/// Data exporter
pub struct DataExporter {
    format: ExportFormat,
    data_points: Vec<DataPoint>,
}

impl DataExporter {
    /// Creates a new data exporter with the specified format
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            data_points: Vec::new(),
        }
    }

    /// Adds a single data point to the export queue
    pub fn add_data_point(&mut self, timestamp: u64, metric_name: impl Into<String>, value: f32) {
        self.data_points.push(DataPoint {
            timestamp,
            metric_name: metric_name.into(),
            value,
        });
    }

    /// Adds system metrics snapshot to the export queue
    pub fn add_system_metrics(&mut self, timestamp: u64, metrics: &SystemMetrics) {
        self.add_data_point(timestamp, "cpu_total", metrics.cpu_total);
        let memory_used_mb = (metrics.memory_total - metrics.memory_available) as f32 / 1024.0 / 1024.0;
        let memory_total_mb = metrics.memory_total as f32 / 1024.0 / 1024.0;
        self.add_data_point(timestamp, "memory_used_mb", memory_used_mb);
        self.add_data_point(timestamp, "memory_total_mb", memory_total_mb);
        self.add_data_point(timestamp, "memory_load_percent", metrics.memory_load_percent as f32);
    }

    pub fn clear(&mut self) {
        self.data_points.clear();
    }

    pub fn export_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        match self.format {
            ExportFormat::Csv => self.export_csv(path),
            ExportFormat::Json => self.export_json(path),
            ExportFormat::Sqlite => self.export_sqlite(path),
        }
    }

    fn export_csv(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write BOM for Excel compatibility
        writer.write_all(&[0xEF, 0xBB, 0xBF])?;

        // Write header
        writeln!(writer, "timestamp,metric_name,value")?;

        // Write data points
        for point in &self.data_points {
            writeln!(
                writer,
                "{},{},{}",
                point.timestamp, point.metric_name, point.value
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    fn export_json(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Group data points by metric name
        let mut metrics: std::collections::HashMap<String, Vec<(u64, f32)>> =
            std::collections::HashMap::new();

        for point in &self.data_points {
            metrics
                .entry(point.metric_name.clone())
                .or_default()
                .push((point.timestamp, point.value));
        }

        // Write JSON
        writeln!(writer, "{{")?;
        writeln!(writer, "  \"export_format\": \"task_manager_metrics\",")?;
        writeln!(writer, "  \"version\": \"1.0\",")?;
        writeln!(writer, "  \"metrics\": {{")?;

        let mut first_metric = true;
        for (name, series) in metrics.iter() {
            if !first_metric {
                writeln!(writer, ",")?;
            }
            first_metric = false;

            writeln!(writer, "    \"{}\": [", name)?;
            for (i, (timestamp, value)) in series.iter().enumerate() {
                let comma = if i < series.len() - 1 { "," } else { "" };
                writeln!(
                    writer,
                    "      {{\"timestamp\": {}, \"value\": {}}}{}",
                    timestamp, value, comma
                )?;
            }
            write!(writer, "    ]")?;
        }

        writeln!(writer)?;
        writeln!(writer, "  }}")?;
        writeln!(writer, "}}")?;

        writer.flush()?;
        Ok(())
    }

    fn export_sqlite(&self, _path: impl AsRef<Path>) -> std::io::Result<()> {
        // SQLite export requires rusqlite crate
        // For now, return an error as it's not implemented
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "SQLite export requires rusqlite dependency (not yet added)",
        ))
    }
}

/// Save file dialog result
pub struct SaveDialogResult {
    pub path: String,
    pub format: ExportFormat,
}

/// Show save file dialog
pub fn show_save_dialog(default_format: ExportFormat) -> Option<SaveDialogResult> {
    // TODO: Implement IFileSaveDialog COM interface
    // This requires Windows COM interop which is complex
    // For now, return None

    // Placeholder implementation
    let _ = default_format;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Sqlite.extension(), "db");
    }

    #[test]
    fn test_data_exporter() {
        let mut exporter = DataExporter::new(ExportFormat::Csv);
        exporter.add_data_point(1000, "cpu", 50.0);
        assert_eq!(exporter.data_points.len(), 1);
    }

    #[test]
    fn test_csv_export() -> std::io::Result<()> {
        let mut exporter = DataExporter::new(ExportFormat::Csv);
        exporter.add_data_point(1000, "cpu", 50.0);
        exporter.add_data_point(2000, "memory", 75.0);

        let temp_file = std::env::temp_dir().join("test_export.csv");
        exporter.export_to_file(&temp_file)?;

        let content = std::fs::read_to_string(&temp_file)?;
        assert!(content.contains("timestamp,metric_name,value"));
        assert!(content.contains("1000,cpu,50"));

        std::fs::remove_file(temp_file)?;
        Ok(())
    }
}
