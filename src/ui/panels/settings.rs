//! Settings panel for user preferences
//!
//! Implements T415-T422:
//! - Theme selector (Light/Dark/System)
//! - Refresh rate selector (0.1s - 10s)
//! - History length selector (1min - 24hr)
//! - Graph type selector (Line/Area/Both)
//! - Startup options (run at login, start minimized)
//! - Column visibility toggles
//! - Performance mode toggle

use windows::Win32::Foundation::RECT;
use std::collections::HashMap;

use crate::app::theme::Theme;
use crate::app::config::ConfigManager;

/// Settings panel sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Monitoring,
    Columns,
    Startup,
    Performance,
}

/// Theme selector options (T416)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeOption {
    System,
    Light,
    Dark,
}

impl ThemeOption {
    pub fn to_theme(self) -> Theme {
        match self {
            ThemeOption::System => Theme::System,
            ThemeOption::Light => Theme::Light,
            ThemeOption::Dark => Theme::Dark,
        }
    }

    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::System => ThemeOption::System,
            Theme::Light => ThemeOption::Light,
            Theme::Dark => ThemeOption::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeOption::System => "System default",
            ThemeOption::Light => "Light",
            ThemeOption::Dark => "Dark",
        }
    }

    pub fn all() -> &'static [ThemeOption] {
        &[ThemeOption::System, ThemeOption::Light, ThemeOption::Dark]
    }
}

/// Refresh rate options (T417)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshRate {
    Fast100ms,
    Fast500ms,
    Normal1s,
    Slow2s,
    VerySlow5s,
    Paused10s,
}

impl RefreshRate {
    pub fn to_millis(self) -> u32 {
        match self {
            RefreshRate::Fast100ms => 100,
            RefreshRate::Fast500ms => 500,
            RefreshRate::Normal1s => 1000,
            RefreshRate::Slow2s => 2000,
            RefreshRate::VerySlow5s => 5000,
            RefreshRate::Paused10s => 10000,
        }
    }

    pub fn from_millis(ms: u32) -> Self {
        match ms {
            0..=100 => RefreshRate::Fast100ms,
            101..=500 => RefreshRate::Fast500ms,
            501..=1000 => RefreshRate::Normal1s,
            1001..=2000 => RefreshRate::Slow2s,
            2001..=5000 => RefreshRate::VerySlow5s,
            _ => RefreshRate::Paused10s,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RefreshRate::Fast100ms => "High (0.1 seconds)",
            RefreshRate::Fast500ms => "Normal-high (0.5 seconds)",
            RefreshRate::Normal1s => "Normal (1 second)",
            RefreshRate::Slow2s => "Low (2 seconds)",
            RefreshRate::VerySlow5s => "Very low (5 seconds)",
            RefreshRate::Paused10s => "Paused (10 seconds)",
        }
    }

    pub fn all() -> &'static [RefreshRate] {
        &[
            RefreshRate::Fast100ms,
            RefreshRate::Fast500ms,
            RefreshRate::Normal1s,
            RefreshRate::Slow2s,
            RefreshRate::VerySlow5s,
            RefreshRate::Paused10s,
        ]
    }
}

/// History length options (T418)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryLength {
    OneMinute,
    FiveMinutes,
    OneHour,
    TwentyFourHours,
}

impl HistoryLength {
    pub fn to_seconds(self) -> u32 {
        match self {
            HistoryLength::OneMinute => 60,
            HistoryLength::FiveMinutes => 300,
            HistoryLength::OneHour => 3600,
            HistoryLength::TwentyFourHours => 86400,
        }
    }

    pub fn from_seconds(secs: u32) -> Self {
        match secs {
            0..=60 => HistoryLength::OneMinute,
            61..=300 => HistoryLength::FiveMinutes,
            301..=3600 => HistoryLength::OneHour,
            _ => HistoryLength::TwentyFourHours,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            HistoryLength::OneMinute => "1 minute",
            HistoryLength::FiveMinutes => "5 minutes",
            HistoryLength::OneHour => "1 hour",
            HistoryLength::TwentyFourHours => "24 hours",
        }
    }

    pub fn all() -> &'static [HistoryLength] {
        &[
            HistoryLength::OneMinute,
            HistoryLength::FiveMinutes,
            HistoryLength::OneHour,
            HistoryLength::TwentyFourHours,
        ]
    }
}

/// Graph type options (T419)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphType {
    Line,
    Area,
    Both,
}

impl GraphType {
    pub fn to_u32(self) -> u32 {
        match self {
            GraphType::Line => 0,
            GraphType::Area => 1,
            GraphType::Both => 2,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => GraphType::Area,
            2 => GraphType::Both,
            _ => GraphType::Line,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GraphType::Line => "Line graph",
            GraphType::Area => "Area graph",
            GraphType::Both => "Line + Area",
        }
    }

    pub fn all() -> &'static [GraphType] {
        &[GraphType::Line, GraphType::Area, GraphType::Both]
    }
}

/// Settings panel state
pub struct SettingsPanel {
    /// Current section being viewed
    current_section: SettingsSection,
    /// Settings bounds
    bounds: RECT,
    
    // Appearance settings (T416)
    pub theme: ThemeOption,
    
    // Monitoring settings (T417-T419)
    pub refresh_rate: RefreshRate,
    pub history_length: HistoryLength,
    pub graph_type: GraphType,
    
    // Startup options (T420)
    pub run_at_login: bool,
    pub start_minimized: bool,
    
    // Column visibility (T421)
    pub column_visibility: HashMap<String, bool>,
    
    // Performance options (T422)
    pub performance_mode: bool,
}

impl SettingsPanel {
    /// Create new settings panel
    pub fn new() -> Self {
        Self {
            current_section: SettingsSection::Appearance,
            bounds: RECT::default(),
            theme: ThemeOption::System,
            refresh_rate: RefreshRate::Normal1s,
            history_length: HistoryLength::OneMinute,
            graph_type: GraphType::Line,
            run_at_login: false,
            start_minimized: false,
            column_visibility: Self::default_column_visibility(),
            performance_mode: false,
        }
    }

    /// Load settings from config manager
    pub fn load_from_config(&mut self, config: &ConfigManager) {
        let app_config = config.get();
        
        // Load appearance
        self.theme = ThemeOption::from_theme(app_config.theme.preference);
        
        // Load monitoring
        self.refresh_rate = RefreshRate::from_millis(app_config.monitoring.refresh_rate_ms);
        self.history_length = HistoryLength::from_seconds(app_config.monitoring.history_length_sec);
        self.graph_type = GraphType::from_u32(app_config.monitoring.graph_type);
        
        // Load startup
        self.run_at_login = app_config.startup.run_at_login;
        self.start_minimized = app_config.startup.start_minimized;
        self.performance_mode = app_config.startup.performance_mode;
        
        // Load column visibility
        for (name, visible) in &app_config.columns.visibility {
            self.column_visibility.insert(name.clone(), *visible);
        }
    }

    /// Save settings to config manager
    pub fn save_to_config(&self, config: &ConfigManager) {
        // Save theme
        config.set_theme(self.theme.to_theme());
        
        // Save monitoring
        config.set_monitoring(
            self.refresh_rate.to_millis(),
            self.history_length.to_seconds(),
            self.graph_type.to_u32(),
        );
        
        // Save startup
        config.set_startup_options(
            self.run_at_login,
            self.start_minimized,
            self.performance_mode,
        );
        
        // Save column visibility
        for (name, visible) in &self.column_visibility {
            config.set_column_visibility(name, *visible);
        }
    }

    /// Set current section
    pub fn set_section(&mut self, section: SettingsSection) {
        self.current_section = section;
    }

    /// Get current section
    pub fn current_section(&self) -> SettingsSection {
        self.current_section
    }

    /// Set panel bounds
    pub fn set_bounds(&mut self, bounds: RECT) {
        self.bounds = bounds;
    }

    /// Get panel bounds
    pub fn bounds(&self) -> RECT {
        self.bounds
    }

    /// Toggle column visibility (T421)
    pub fn toggle_column(&mut self, column_name: &str) {
        let current = self.column_visibility.get(column_name).copied().unwrap_or(true);
        self.column_visibility.insert(column_name.to_string(), !current);
    }

    /// Get column visibility
    pub fn is_column_visible(&self, column_name: &str) -> bool {
        self.column_visibility.get(column_name).copied().unwrap_or(true)
    }

    /// Get all available sections
    pub fn all_sections() -> &'static [SettingsSection] {
        &[
            SettingsSection::Appearance,
            SettingsSection::Monitoring,
            SettingsSection::Columns,
            SettingsSection::Startup,
            SettingsSection::Performance,
        ]
    }

    /// Get section label
    pub fn section_label(section: SettingsSection) -> &'static str {
        match section {
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Monitoring => "Monitoring",
            SettingsSection::Columns => "Columns",
            SettingsSection::Startup => "Startup",
            SettingsSection::Performance => "Performance",
        }
    }

    /// Default column visibility
    fn default_column_visibility() -> HashMap<String, bool> {
        let mut visibility = HashMap::new();
        
        // Default visible columns
        visibility.insert("Name".to_string(), true);
        visibility.insert("PID".to_string(), true);
        visibility.insert("CPU".to_string(), true);
        visibility.insert("Memory".to_string(), true);
        visibility.insert("Status".to_string(), true);
        
        // Default hidden columns
        visibility.insert("Threads".to_string(), false);
        visibility.insert("Handles".to_string(), false);
        visibility.insert("Disk".to_string(), false);
        visibility.insert("Network".to_string(), false);
        visibility.insert("GPU".to_string(), false);
        
        visibility
    }

    /// Get settings for a specific section
    pub fn get_section_settings(&self, section: SettingsSection) -> Vec<SettingItem> {
        match section {
            SettingsSection::Appearance => vec![
                SettingItem::Choice {
                    label: "Theme",
                    options: ThemeOption::all().iter().map(|t| t.label()).collect(),
                    selected: self.theme as usize,
                },
            ],
            SettingsSection::Monitoring => vec![
                SettingItem::Choice {
                    label: "Refresh rate",
                    options: RefreshRate::all().iter().map(|r| r.label()).collect(),
                    selected: Self::refresh_rate_index(self.refresh_rate),
                },
                SettingItem::Choice {
                    label: "History length",
                    options: HistoryLength::all().iter().map(|h| h.label()).collect(),
                    selected: Self::history_length_index(self.history_length),
                },
                SettingItem::Choice {
                    label: "Graph type",
                    options: GraphType::all().iter().map(|g| g.label()).collect(),
                    selected: self.graph_type as usize,
                },
            ],
            SettingsSection::Columns => {
                let mut items = Vec::new();
                let mut columns: Vec<_> = self.column_visibility.keys().cloned().collect();
                columns.sort();
                
                for column in columns {
                    items.push(SettingItem::Toggle {
                        label: column.clone(),
                        enabled: self.column_visibility[&column],
                    });
                }
                items
            },
            SettingsSection::Startup => vec![
                SettingItem::Toggle {
                    label: "Run at Windows startup".to_string(),
                    enabled: self.run_at_login,
                },
                SettingItem::Toggle {
                    label: "Start minimized".to_string(),
                    enabled: self.start_minimized,
                },
            ],
            SettingsSection::Performance => vec![
                SettingItem::Toggle {
                    label: "Performance mode (disable animations)".to_string(),
                    enabled: self.performance_mode,
                },
            ],
        }
    }

    fn refresh_rate_index(rate: RefreshRate) -> usize {
        RefreshRate::all().iter().position(|&r| r == rate).unwrap_or(2)
    }

    fn history_length_index(length: HistoryLength) -> usize {
        HistoryLength::all().iter().position(|&l| l == length).unwrap_or(0)
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Setting item types for rendering
#[derive(Debug, Clone)]
pub enum SettingItem {
    /// Choice selector (dropdown/radio)
    Choice {
        label: &'static str,
        options: Vec<&'static str>,
        selected: usize,
    },
    /// Toggle checkbox
    Toggle {
        label: String,
        enabled: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_panel_creation() {
        let panel = SettingsPanel::new();
        assert_eq!(panel.current_section, SettingsSection::Appearance);
        assert_eq!(panel.theme, ThemeOption::System);
        assert_eq!(panel.refresh_rate, RefreshRate::Normal1s);
    }

    #[test]
    fn test_theme_options() {
        assert_eq!(ThemeOption::all().len(), 3);
        assert_eq!(ThemeOption::Light.label(), "Light");
        assert_eq!(ThemeOption::Dark.label(), "Dark");
    }

    #[test]
    fn test_refresh_rate_conversion() {
        assert_eq!(RefreshRate::Normal1s.to_millis(), 1000);
        assert_eq!(RefreshRate::Fast100ms.to_millis(), 100);
        assert_eq!(RefreshRate::from_millis(500), RefreshRate::Fast500ms);
        assert_eq!(RefreshRate::from_millis(1200), RefreshRate::Slow2s);
    }

    #[test]
    fn test_history_length_conversion() {
        assert_eq!(HistoryLength::OneMinute.to_seconds(), 60);
        assert_eq!(HistoryLength::OneHour.to_seconds(), 3600);
        assert_eq!(HistoryLength::from_seconds(300), HistoryLength::FiveMinutes);
    }

    #[test]
    fn test_graph_type() {
        assert_eq!(GraphType::Line.to_u32(), 0);
        assert_eq!(GraphType::Area.to_u32(), 1);
        assert_eq!(GraphType::Both.to_u32(), 2);
        assert_eq!(GraphType::from_u32(1), GraphType::Area);
    }

    #[test]
    fn test_column_visibility() {
        let mut panel = SettingsPanel::new();
        
        // Default visibility
        assert!(panel.is_column_visible("Name"));
        assert!(panel.is_column_visible("CPU"));
        assert!(!panel.is_column_visible("Threads"));
        
        // Toggle
        panel.toggle_column("Threads");
        assert!(panel.is_column_visible("Threads"));
        
        panel.toggle_column("CPU");
        assert!(!panel.is_column_visible("CPU"));
    }

    #[test]
    fn test_section_navigation() {
        let mut panel = SettingsPanel::new();
        
        assert_eq!(panel.current_section(), SettingsSection::Appearance);
        
        panel.set_section(SettingsSection::Monitoring);
        assert_eq!(panel.current_section(), SettingsSection::Monitoring);
    }

    #[test]
    fn test_config_integration() {
        let config = ConfigManager::new();
        let mut panel = SettingsPanel::new();
        
        // Modify settings
        panel.theme = ThemeOption::Dark;
        panel.refresh_rate = RefreshRate::Fast500ms;
        panel.run_at_login = true;
        
        // Save to config
        panel.save_to_config(&config);
        
        // Load into new panel
        let mut panel2 = SettingsPanel::new();
        panel2.load_from_config(&config);
        
        assert_eq!(panel2.theme, ThemeOption::Dark);
        assert_eq!(panel2.refresh_rate, RefreshRate::Fast500ms);
        assert!(panel2.run_at_login);
    }

    #[test]
    fn test_get_section_settings() {
        let panel = SettingsPanel::new();
        
        let appearance = panel.get_section_settings(SettingsSection::Appearance);
        assert_eq!(appearance.len(), 1); // Theme selector
        
        let monitoring = panel.get_section_settings(SettingsSection::Monitoring);
        assert_eq!(monitoring.len(), 3); // Refresh, history, graph type
        
        let startup = panel.get_section_settings(SettingsSection::Startup);
        assert_eq!(startup.len(), 2); // Run at login, start minimized
    }

    #[test]
    fn test_all_sections() {
        let sections = SettingsPanel::all_sections();
        assert_eq!(sections.len(), 5);
        assert_eq!(SettingsPanel::section_label(SettingsSection::Appearance), "Appearance");
    }
}
