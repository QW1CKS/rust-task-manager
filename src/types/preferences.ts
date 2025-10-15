/**
 * Theme mode type matching Rust ThemeMode enum
 */
export type ThemeMode = 'dark' | 'light';

/**
 * Window state interface matching Rust WindowState struct
 */
export interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
  maximized: boolean;
}

/**
 * User preferences interface matching Rust UserPreferences struct
 *
 * Persisted application settings stored in %APPDATA%\rust-task-manager\config.json
 */
export interface UserPreferences {
  theme: ThemeMode;
  window: WindowState;
  sortColumn?: string;
  sortOrder?: string;
}
