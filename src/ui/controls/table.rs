//! Process Table with Virtualized Scrolling (T185-T192)
//!
//! High-performance table control for displaying 1000+ processes with:
//! - Virtualized scrolling (only render visible rows)
//! - Column headers with click-to-sort
//! - Row selection (mouse + keyboard)
//! - Multi-selection (Ctrl+Click, Shift+Click)
//! - Alternating row colors

use crate::core::filter::{SortColumn, SortDirection};
use std::collections::HashSet;

/// Simple process info for table display
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub parent_pid: u32,
    /// Process name/executable
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Private memory in bytes
    pub memory_private: u64,
    /// Working set memory in bytes
    pub memory_working_set: u64,
    /// Number of handles
    pub handle_count: u32,
}

/// T185: Table column definition
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumn {
    /// Column identifier for sorting
    pub id: SortColumn,
    /// Column width in pixels
    pub width: f32,
    /// Column header label
    pub label: &'static str,
}

impl TableColumn {
    /// Name column definition
    pub const NAME: Self = Self {
        id: SortColumn::Name,
        width: 200.0,
        label: "Name",
    };

    /// PID column definition
    pub const PID: Self = Self {
        id: SortColumn::Pid,
        width: 80.0,
        label: "PID",
    };

    /// CPU column definition
    pub const CPU: Self = Self {
        id: SortColumn::Cpu,
        width: 80.0,
        label: "CPU %",
    };

    /// Memory column definition
    pub const MEMORY: Self = Self {
        id: SortColumn::Memory,
        width: 120.0,
        label: "Memory",
    };

    /// Handles column definition
    pub const HANDLES: Self = Self {
        id: SortColumn::Handles,
        width: 80.0,
        label: "Handles",
    };
}

/// T185: Default table columns
pub const DEFAULT_COLUMNS: &[TableColumn] = &[
    TableColumn::NAME,
    TableColumn::PID,
    TableColumn::CPU,
    TableColumn::MEMORY,
    TableColumn::HANDLES,
];

/// T191: Table selection state
#[derive(Debug, Default)]
pub struct TableSelection {
    /// Currently selected PIDs
    selected: HashSet<u32>,
    /// Last clicked PID (for Shift+Click range selection)
    last_clicked: Option<u32>,
}

impl TableSelection {
    /// Create a new empty table selection
    pub fn new() -> Self {
        Self::default()
    }

    /// T191: Single selection (click)
    pub fn select_single(&mut self, pid: u32) {
        self.selected.clear();
        self.selected.insert(pid);
        self.last_clicked = Some(pid);
    }

    /// T191: Toggle selection (Ctrl+Click)
    pub fn toggle(&mut self, pid: u32) {
        if self.selected.contains(&pid) {
            self.selected.remove(&pid);
        } else {
            self.selected.insert(pid);
        }
        self.last_clicked = Some(pid);
    }

    /// T192: Range selection (Shift+Click)
    pub fn select_range(&mut self, pid: u32, all_processes: &[ProcessInfo]) {
        if let Some(last) = self.last_clicked {
            self.selected.clear();
            
            // Find indices
            let start_idx = all_processes.iter().position(|p| p.pid == last);
            let end_idx = all_processes.iter().position(|p| p.pid == pid);

            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };

                for process in &all_processes[start..=end] {
                    self.selected.insert(process.pid);
                }
            }
        } else {
            self.select_single(pid);
        }
    }

    /// Check if PID is selected
    pub fn is_selected(&self, pid: u32) -> bool {
        self.selected.contains(&pid)
    }

    /// Get all selected PIDs
    pub fn selected_pids(&self) -> Vec<u32> {
        self.selected.iter().copied().collect()
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.selected.clear();
        self.last_clicked = None;
    }
}

/// T186-T190: Virtualized process table
pub struct ProcessTable {
    /// Table columns
    columns: Vec<TableColumn>,
    /// Current sort state
    sort_column: SortColumn,
    sort_direction: SortDirection,
    /// Selection state
    selection: TableSelection,
    /// Scroll position (row index)
    scroll_offset: usize,
    /// Row height in pixels
    row_height: f32,
    /// Header height in pixels
    header_height: f32,
}

impl ProcessTable {
    /// Create new table
    pub fn new() -> Self {
        Self {
            columns: DEFAULT_COLUMNS.to_vec(),
            sort_column: SortColumn::Cpu,
            sort_direction: SortDirection::Descending,
            selection: TableSelection::new(),
            scroll_offset: 0,
            row_height: 24.0,
            header_height: 30.0,
        }
    }

    /// T189: Handle column header click (toggle sort)
    pub fn on_header_click(&mut self, column: SortColumn) {
        if self.sort_column == column {
            // Toggle direction
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            // New column, default to descending
            self.sort_column = column;
            self.sort_direction = SortDirection::Descending;
        }
    }

    /// Get current sort state
    pub fn sort_state(&self) -> (SortColumn, SortDirection) {
        (self.sort_column, self.sort_direction)
    }

    /// T190: Handle row click
    pub fn on_row_click(&mut self, pid: u32, ctrl: bool, shift: bool, all_processes: &[ProcessInfo]) {
        if shift {
            self.selection.select_range(pid, all_processes);
        } else if ctrl {
            self.selection.toggle(pid);
        } else {
            self.selection.select_single(pid);
        }
    }

    /// T191: Handle keyboard navigation
    pub fn on_key_down(&mut self, key: u32, ctrl: bool, all_processes: &[ProcessInfo]) {
        // Arrow keys, Enter, etc.
        // Simplified: just navigate selection
        match key {
            0x26 => {
                // Up arrow
                if let Some(current) = self.selection.selected_pids().first() {
                    if let Some(idx) = all_processes.iter().position(|p| p.pid == *current) {
                        if idx > 0 {
                            let new_pid = all_processes[idx - 1].pid;
                            if ctrl {
                                self.selection.toggle(new_pid);
                            } else {
                                self.selection.select_single(new_pid);
                            }
                        }
                    }
                }
            }
            0x28 => {
                // Down arrow
                if let Some(current) = self.selection.selected_pids().first() {
                    if let Some(idx) = all_processes.iter().position(|p| p.pid == *current) {
                        if idx < all_processes.len() - 1 {
                            let new_pid = all_processes[idx + 1].pid;
                            if ctrl {
                                self.selection.toggle(new_pid);
                            } else {
                                self.selection.select_single(new_pid);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Get selected PIDs
    pub fn selected_pids(&self) -> Vec<u32> {
        self.selection.selected_pids()
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// T186-T188: Calculate visible rows (virtualization logic)
    /// Returns (start_row, end_row, visible_count)
    pub fn calculate_visible_rows(
        &self,
        processes: &[ProcessInfo],
        viewport_height: f32,
    ) -> (usize, usize, usize) {
        let table_height = viewport_height - self.header_height;
        let visible_rows = (table_height / self.row_height).ceil() as usize + 1;
        let start_row = self.scroll_offset;
        let end_row = (start_row + visible_rows).min(processes.len());
        (start_row, end_row, visible_rows)
    }

    /// Format cell text for display
    pub fn format_cell_text(&self, column: SortColumn, process: &ProcessInfo) -> String {
        match column {
            SortColumn::Name => process.name.clone(),
            SortColumn::Pid => process.pid.to_string(),
            SortColumn::Cpu => format!("{:.1}", process.cpu_usage),
            SortColumn::Memory => format_bytes(process.memory_private),
            SortColumn::Handles => process.handle_count.to_string(),
        }
    }

    /// Get header text with sort indicator
    pub fn format_header_text(&self, column: &TableColumn) -> String {
        if self.sort_column == column.id {
            match self.sort_direction {
                SortDirection::Ascending => format!("{} ▲", column.label),
                SortDirection::Descending => format!("{} ▼", column.label),
            }
        } else {
            column.label.to_string()
        }
    }

    /// Calculate which row is at the given Y coordinate
    pub fn row_at_point(&self, y: f32) -> Option<usize> {
        if y < self.header_height {
            None
        } else {
            let row_y = y - self.header_height;
            let row = (row_y / self.row_height) as usize;
            Some(self.scroll_offset + row)
        }
    }

    /// Calculate which column is at the given X coordinate
    pub fn column_at_point(&self, x: f32) -> Option<SortColumn> {
        let mut col_x = 0.0;
        for column in &self.columns {
            if x >= col_x && x < col_x + column.width {
                return Some(column.id);
            }
            col_x += column.width;
        }
        None
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_process(name: &str, pid: u32, cpu: f64, memory: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            parent_pid: 0,
            name: name.to_string(),
            cpu_usage: cpu,
            memory_private: memory,
            memory_working_set: memory,
            handle_count: 100,
        }
    }

    #[test]
    fn test_selection_single() {
        let mut selection = TableSelection::new();
        selection.select_single(1000);

        assert!(selection.is_selected(1000));
        assert_eq!(selection.selected_pids(), vec![1000]);
    }

    #[test]
    fn test_selection_toggle() {
        let mut selection = TableSelection::new();
        selection.select_single(1000);
        selection.toggle(2000);

        assert!(selection.is_selected(1000));
        assert!(selection.is_selected(2000));

        selection.toggle(1000);
        assert!(!selection.is_selected(1000));
        assert!(selection.is_selected(2000));
    }

    #[test]
    fn test_selection_range() {
        let processes = vec![
            make_test_process("p1", 1000, 0.0, 0),
            make_test_process("p2", 2000, 0.0, 0),
            make_test_process("p3", 3000, 0.0, 0),
            make_test_process("p4", 4000, 0.0, 0),
        ];

        let mut selection = TableSelection::new();
        selection.select_single(1000);
        selection.select_range(3000, &processes);

        assert!(selection.is_selected(1000));
        assert!(selection.is_selected(2000));
        assert!(selection.is_selected(3000));
        assert!(!selection.is_selected(4000));
    }

    #[test]
    fn test_sort_toggle() {
        let mut table = ProcessTable::new();

        // Initial sort: CPU descending
        assert_eq!(table.sort_state(), (SortColumn::Cpu, SortDirection::Descending));

        // Click same column - should toggle direction
        table.on_header_click(SortColumn::Cpu);
        assert_eq!(table.sort_state(), (SortColumn::Cpu, SortDirection::Ascending));

        // Click again - toggle back
        table.on_header_click(SortColumn::Cpu);
        assert_eq!(table.sort_state(), (SortColumn::Cpu, SortDirection::Descending));

        // Click different column - should switch and default to descending
        table.on_header_click(SortColumn::Memory);
        assert_eq!(table.sort_state(), (SortColumn::Memory, SortDirection::Descending));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.0 GB");
    }

    #[test]
    fn test_row_at_point() {
        let table = ProcessTable::new();

        // Point in header
        assert_eq!(table.row_at_point(15.0), None);

        // First row (header_height + row_height/2)
        assert_eq!(table.row_at_point(42.0), Some(0));

        // Second row
        assert_eq!(table.row_at_point(66.0), Some(1));
    }
}
