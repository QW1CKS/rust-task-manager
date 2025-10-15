/**
 * User theme preference
 * Matches Rust Theme enum
 */
export type Theme = 'dark' | 'light';

/**
 * Window position and size preferences
 * Matches Rust WindowPreferences struct
 */
export interface WindowPreferences {
  width: number;
  height: number;
  x: number;
  y: number;
  maximized: boolean;
}

/**
 * Persisted user settings
 * Matches Rust UserPreferences struct
 */
export interface UserPreferences {
  theme: Theme;
  window: WindowPreferences;
  sortColumn: string;
  sortOrder: string;
}
