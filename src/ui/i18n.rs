// Internationalization and localization (T476-T478)
//
// String resource loading with fallback to English

use std::collections::HashMap;

// Note: GetUserDefaultLocaleName not available in windows-rs 0.62
// Using stub implementation

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    EnglishUS,
    // Future: Add more locales
    // FrenchFR,
    // GermanDE,
    // JapaneseJP,
}

impl Locale {
    /// Get locale code string
    pub fn code(&self) -> &'static str {
        match self {
            Locale::EnglishUS => "en-US",
        }
    }

    /// Detect system locale (T478)
    /// Note: API not available in windows-rs 0.62 - defaulting to en-US
    pub fn detect_system_locale() -> Self {
        // TODO: Implement GetUserDefaultLocaleName when API becomes available
        // For now, default to English US
        Locale::EnglishUS
    }

    /// Parse locale from code string
    #[allow(dead_code)]
    fn from_code(code: &str) -> Self {
        match code {
            "en-US" | "en" => Locale::EnglishUS,
            _ => Locale::EnglishUS, // Fallback to English
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::EnglishUS
    }
}

/// String resource manager (T476, T477)
pub struct StringResources {
    locale: Locale,
    strings: HashMap<&'static str, &'static str>,
}

impl StringResources {
    /// Create string resources for locale (T477)
    pub fn new(locale: Locale) -> Self {
        let mut strings = HashMap::new();
        
        // Load strings for the specified locale
        match locale {
            Locale::EnglishUS => Self::load_english(&mut strings),
        }
        
        Self {
            locale,
            strings,
        }
    }

    /// Load English strings (T476)
    fn load_english(strings: &mut HashMap<&'static str, &'static str>) {
        // Window titles
        strings.insert("app.title", "Task Manager");
        strings.insert("app.admin", "Administrator");
        
        // Tabs
        strings.insert("tab.processes", "Processes");
        strings.insert("tab.performance", "Performance");
        strings.insert("tab.startup", "Startup");
        strings.insert("tab.services", "Services");
        strings.insert("tab.users", "Users");
        strings.insert("tab.details", "Details");
        strings.insert("tab.gpu", "GPU");
        
        // Columns
        strings.insert("column.name", "Name");
        strings.insert("column.pid", "PID");
        strings.insert("column.cpu", "CPU");
        strings.insert("column.memory", "Memory");
        strings.insert("column.status", "Status");
        strings.insert("column.description", "Description");
        
        // Actions
        strings.insert("action.end_task", "End Task");
        strings.insert("action.refresh", "Refresh");
        strings.insert("action.start", "Start");
        strings.insert("action.stop", "Stop");
        strings.insert("action.restart", "Restart");
        strings.insert("action.show", "Show");
        strings.insert("action.hide", "Hide");
        strings.insert("action.exit", "Exit");
        
        // Status
        strings.insert("status.running", "Running");
        strings.insert("status.stopped", "Stopped");
        strings.insert("status.enabled", "Enabled");
        strings.insert("status.disabled", "Disabled");
        
        // Dialogs
        strings.insert("dialog.confirm_end_task", "Do you want to end this process?");
        strings.insert("dialog.confirm_end_task_title", "Confirm End Task");
        strings.insert("dialog.error", "Error");
        strings.insert("dialog.warning", "Warning");
        strings.insert("dialog.information", "Information");
        
        // Status bar
        strings.insert("statusbar.processes", "Processes");
        strings.insert("statusbar.cpu", "CPU");
        strings.insert("statusbar.memory", "Memory");
        strings.insert("statusbar.updated", "Updated");
        strings.insert("statusbar.ago", "ago");
        
        // Settings
        strings.insert("settings.title", "Settings");
        strings.insert("settings.theme", "Theme");
        strings.insert("settings.refresh_rate", "Refresh Rate");
        strings.insert("settings.history_length", "History Length");
        strings.insert("settings.graph_type", "Graph Type");
        strings.insert("settings.startup", "Startup Options");
        strings.insert("settings.performance_mode", "Performance Mode");
        
        // Themes
        strings.insert("theme.light", "Light");
        strings.insert("theme.dark", "Dark");
        strings.insert("theme.system", "System");
        
        // Performance modes
        strings.insert("perf_mode.performance", "Performance");
        strings.insert("perf_mode.battery_saver", "Battery Saver");
        strings.insert("perf_mode.manual", "Manual");
        
        // Units
        strings.insert("unit.percent", "%");
        strings.insert("unit.mb", "MB");
        strings.insert("unit.gb", "GB");
        strings.insert("unit.seconds", "seconds");
        strings.insert("unit.second", "second");
    }

    /// Get localized string with fallback (T477)
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).copied().unwrap_or(key)
    }

    /// Get formatted string with parameters
    pub fn get_formatted(&self, key: &str, params: &[&str]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();
        
        for (i, param) in params.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, param);
        }
        
        result
    }

    /// Get current locale
    pub fn locale(&self) -> Locale {
        self.locale
    }
}

impl Default for StringResources {
    fn default() -> Self {
        Self::new(Locale::detect_system_locale())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_detection() {
        let locale = Locale::detect_system_locale();
        // Should not crash and return a valid locale
        assert_eq!(locale.code().len() > 0, true);
    }

    #[test]
    fn test_string_loading() {
        let strings = StringResources::new(Locale::EnglishUS);
        assert_eq!(strings.get("app.title"), "Task Manager");
        assert_eq!(strings.get("tab.processes"), "Processes");
        assert_eq!(strings.get("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_formatted_strings() {
        let strings = StringResources::new(Locale::EnglishUS);
        // Manually insert a template for testing
        let mut test_strings = strings;
        test_strings.strings.insert("test.format", "Hello {0}, you have {1} messages");
        
        let formatted = test_strings.get_formatted("test.format", &["Alice", "5"]);
        assert_eq!(formatted, "Hello Alice, you have 5 messages");
    }

    #[test]
    fn test_locale_code() {
        assert_eq!(Locale::EnglishUS.code(), "en-US");
    }
}
