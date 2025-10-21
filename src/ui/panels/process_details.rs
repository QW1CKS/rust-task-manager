//! Process Details Panel (T213-T219)
//!
//! Display detailed information about selected process:
//! - Basic info (name, PID, status, command line)
//! - Memory details (Working Set, Private Bytes, Commit Charge)
//! - Resource counts (threads, handles, GDI/USER objects)
//! - Security info (user, session ID, integrity level)
//! - Parent process with clickable link
//! - Copy to clipboard functionality

use std::fmt::Write;

/// T213-T219: Process details panel data
#[derive(Debug, Clone, Default)]
pub struct ProcessDetails {
    /// Basic Information
    /// Process name
    pub name: String,
    /// Process ID
    pub pid: u32,
    /// Process status
    pub status: ProcessStatus,
    /// Command line arguments
    pub command_line: Option<String>,
    
    /// Memory Information
    pub working_set: u64,
    /// Private bytes (process-exclusive memory)
    pub private_bytes: u64,
    /// Commit charge (virtual memory)
    pub commit_charge: u64,
    /// Peak working set
    pub peak_working_set: u64,
    
    /// Resource Counts
    pub thread_count: u32,
    /// Number of handles
    pub handle_count: u32,
    /// Number of GDI objects
    pub gdi_objects: u32,
    /// Number of USER objects
    pub user_objects: u32,
    
    /// Security Information
    pub user_name: Option<String>,
    /// Session ID
    pub session_id: u32,
    /// Process integrity level
    pub integrity_level: IntegrityLevel,
    
    /// Parent Information
    pub parent_pid: Option<u32>,
    /// Parent process name
    pub parent_name: Option<String>,
    
    /// Additional Info
    pub cpu_usage: f64,
    /// I/O read bytes
    pub io_read_bytes: u64,
    /// I/O write bytes
    pub io_write_bytes: u64,
}

/// T214: Process status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessStatus {
    #[default]
    /// Process is running normally
    Running,
    /// Process is suspended
    Suspended,
    /// Process is not responding
    NotResponding,
    /// Process is terminating
    Terminating,
}

impl ProcessStatus {
    /// Get string representation of status
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Suspended => "Suspended",
            Self::NotResponding => "Not Responding",
            Self::Terminating => "Terminating",
        }
    }
}

/// T217: Process integrity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum IntegrityLevel {
    /// Untrusted integrity level
    Untrusted,
    /// Low integrity level (sandboxed)
    Low,
    #[default]
    /// Medium integrity level (normal)
    Medium,
    /// High integrity level (elevated)
    High,
    /// System integrity level
    System,
}

impl IntegrityLevel {
    /// Get string representation of integrity level
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Untrusted => "Untrusted",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::System => "System",
        }
    }
}

/// T213: Process details panel UI component
pub struct ProcessDetailsPanel {
    details: Option<ProcessDetails>,
}

impl ProcessDetailsPanel {
    /// Create new process details panel
    pub fn new() -> Self {
        Self { details: None }
    }

    /// T214: Set selected process details
    pub fn set_details(&mut self, details: Option<ProcessDetails>) {
        self.details = details;
    }

    /// Get current details
    pub fn details(&self) -> Option<&ProcessDetails> {
        self.details.as_ref()
    }

    /// Clear selected process
    pub fn clear(&mut self) {
        self.details = None;
    }

    /// T214-T218: Format details as text sections
    pub fn format_sections(&self) -> Vec<(String, Vec<(String, String)>)> {
        let Some(details) = &self.details else {
            return vec![];
        };

        let mut sections = Vec::new();

        // T214: Basic Information
        let mut basic_info = vec![
            ("Name".to_string(), details.name.clone()),
            ("PID".to_string(), details.pid.to_string()),
            ("Status".to_string(), details.status.as_str().to_string()),
        ];
        
        if let Some(ref cmd) = details.command_line {
            basic_info.push(("Command Line".to_string(), cmd.clone()));
        }
        
        sections.push(("Basic Information".to_string(), basic_info));

        // T215: Memory Details
        sections.push((
            "Memory".to_string(),
            vec![
                ("Working Set".to_string(), format_bytes(details.working_set)),
                ("Peak Working Set".to_string(), format_bytes(details.peak_working_set)),
                ("Private Bytes".to_string(), format_bytes(details.private_bytes)),
                ("Commit Charge".to_string(), format_bytes(details.commit_charge)),
            ],
        ));

        // T216: Resource Counts
        sections.push((
            "Resources".to_string(),
            vec![
                ("Threads".to_string(), details.thread_count.to_string()),
                ("Handles".to_string(), details.handle_count.to_string()),
                ("GDI Objects".to_string(), details.gdi_objects.to_string()),
                ("USER Objects".to_string(), details.user_objects.to_string()),
            ],
        ));

        // T217: Security Information
        let mut security_info = vec![
            ("Session ID".to_string(), details.session_id.to_string()),
            ("Integrity Level".to_string(), details.integrity_level.as_str().to_string()),
        ];
        
        if let Some(ref user) = details.user_name {
            security_info.insert(0, ("User".to_string(), user.clone()));
        }
        
        sections.push(("Security".to_string(), security_info));

        // T218: Parent Process
        if let Some(parent_pid) = details.parent_pid {
            let mut parent_info = vec![
                ("Parent PID".to_string(), parent_pid.to_string()),
            ];
            
            if let Some(ref parent_name) = details.parent_name {
                parent_info.push(("Parent Name".to_string(), parent_name.clone()));
            }
            
            sections.push(("Parent Process".to_string(), parent_info));
        }

        // Performance
        sections.push((
            "Performance".to_string(),
            vec![
                ("CPU Usage".to_string(), format!("{:.1}%", details.cpu_usage)),
                ("I/O Read".to_string(), format_bytes(details.io_read_bytes)),
                ("I/O Write".to_string(), format_bytes(details.io_write_bytes)),
            ],
        ));

        sections
    }

    /// T219: Copy details to clipboard as formatted text
    pub fn copy_to_clipboard(&self) -> Option<String> {
        let _details = self.details.as_ref()?;
        let mut text = String::new();

        writeln!(&mut text, "Process Details").ok()?;
        writeln!(&mut text, "===============\n").ok()?;

        for (section_name, items) in self.format_sections() {
            writeln!(&mut text, "{}:", section_name).ok()?;
            for (key, value) in items {
                writeln!(&mut text, "  {}: {}", key, value).ok()?;
            }
            writeln!(&mut text).ok()?;
        }

        writeln!(&mut text, "Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).ok()?;

        Some(text)
    }

    /// T218: Check if parent process link should be clickable
    pub fn has_parent_link(&self) -> bool {
        self.details
            .as_ref()
            .and_then(|d| d.parent_pid)
            .is_some()
    }

    /// T218: Get parent PID for navigation
    pub fn parent_pid(&self) -> Option<u32> {
        self.details
            .as_ref()
            .and_then(|d| d.parent_pid)
    }
}

impl Default for ProcessDetailsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_details() -> ProcessDetails {
        ProcessDetails {
            name: "test.exe".to_string(),
            pid: 1234,
            status: ProcessStatus::Running,
            command_line: Some("test.exe --arg1 --arg2".to_string()),
            working_set: 100_000_000,
            private_bytes: 80_000_000,
            commit_charge: 90_000_000,
            peak_working_set: 120_000_000,
            thread_count: 4,
            handle_count: 256,
            gdi_objects: 12,
            user_objects: 8,
            user_name: Some("DOMAIN\\User".to_string()),
            session_id: 1,
            integrity_level: IntegrityLevel::Medium,
            parent_pid: Some(5678),
            parent_name: Some("explorer.exe".to_string()),
            cpu_usage: 12.5,
            io_read_bytes: 1_000_000,
            io_write_bytes: 500_000,
        }
    }

    #[test]
    fn test_panel_creation() {
        let panel = ProcessDetailsPanel::new();
        assert!(panel.details().is_none());
    }

    #[test]
    fn test_set_details() {
        let mut panel = ProcessDetailsPanel::new();
        let details = make_test_details();
        
        panel.set_details(Some(details));
        assert!(panel.details().is_some());
        assert_eq!(panel.details().unwrap().pid, 1234);
    }

    #[test]
    fn test_clear() {
        let mut panel = ProcessDetailsPanel::new();
        panel.set_details(Some(make_test_details()));
        
        panel.clear();
        assert!(panel.details().is_none());
    }

    #[test]
    fn test_format_sections() {
        let mut panel = ProcessDetailsPanel::new();
        panel.set_details(Some(make_test_details()));
        
        let sections = panel.format_sections();
        assert!(!sections.is_empty());
        
        // Check basic information section
        let basic = sections.iter().find(|(name, _)| name == "Basic Information");
        assert!(basic.is_some());
        
        let (_, items) = basic.unwrap();
        assert!(items.iter().any(|(k, _)| k == "Name"));
        assert!(items.iter().any(|(k, _)| k == "PID"));
    }

    #[test]
    fn test_copy_to_clipboard() {
        let mut panel = ProcessDetailsPanel::new();
        panel.set_details(Some(make_test_details()));
        
        let text = panel.copy_to_clipboard();
        assert!(text.is_some());
        
        let text = text.unwrap();
        assert!(text.contains("test.exe"));
        assert!(text.contains("1234"));
        assert!(text.contains("Running"));
    }

    #[test]
    fn test_parent_link() {
        let mut panel = ProcessDetailsPanel::new();
        assert!(!panel.has_parent_link());
        
        panel.set_details(Some(make_test_details()));
        assert!(panel.has_parent_link());
        assert_eq!(panel.parent_pid(), Some(5678));
    }

    #[test]
    fn test_process_status() {
        assert_eq!(ProcessStatus::Running.as_str(), "Running");
        assert_eq!(ProcessStatus::Suspended.as_str(), "Suspended");
        assert_eq!(ProcessStatus::NotResponding.as_str(), "Not Responding");
    }

    #[test]
    fn test_integrity_level() {
        assert_eq!(IntegrityLevel::Low.as_str(), "Low");
        assert_eq!(IntegrityLevel::Medium.as_str(), "Medium");
        assert_eq!(IntegrityLevel::High.as_str(), "High");
        assert_eq!(IntegrityLevel::System.as_str(), "System");
        
        // Test ordering
        assert!(IntegrityLevel::Low < IntegrityLevel::Medium);
        assert!(IntegrityLevel::Medium < IntegrityLevel::High);
        assert!(IntegrityLevel::High < IntegrityLevel::System);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(5_242_880), "5.00 MB");
        assert_eq!(format_bytes(2_147_483_648), "2.00 GB");
    }
}
