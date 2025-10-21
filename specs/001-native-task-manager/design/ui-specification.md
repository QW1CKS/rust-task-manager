# UI/UX Design Specification: Native Task Manager
# Part 1 - Layout, Visual Design & Typography

**Feature**: Native High-Performance Task Manager  
**Spec Reference**: `../spec.md`  
**Created**: 2025-10-21  
**Status**: Design Specification

---

## 1. LAYOUT ARCHITECTURE

### 1.1 Adaptive Grid System

**Base Grid Unit**: 8px (consistent with Windows 11 Fluent Design)

```
Grid Scale:
- 1x = 8px   (micro spacing, icon padding)
- 2x = 16px  (standard element spacing)
- 3x = 24px  (section padding)
- 4x = 32px  (panel margins)
- 6x = 48px  (major section separation)
```

**Responsive Breakpoints**:

| Breakpoint | Width Range | Layout Behavior | Column Count |
|------------|-------------|-----------------|--------------|
| **Compact** | 800px - 1024px | Single-column metrics, collapsed panels | 4 columns |
| **Standard** | 1025px - 1600px | Two-column metrics, standard panels | 8 columns |
| **Wide** | 1601px - 2560px | Three-column metrics, expanded panels | 12 columns |
| **Ultra-wide** | 2561px+ | Four-column metrics, multi-panel view | 16 columns |

**Minimum Window Dimensions**:
- Width: 800px (enforced by Win32 WM_GETMINMAXINFO)
- Height: 600px
- Below minimum: Show warning overlay with resize instruction

**Grid Alignment Rules**:
- All UI elements snap to 8px grid
- Text baselines align to 4px sub-grid for optical alignment
- Icons align to 4px boundaries for pixel-perfect rendering at all DPI scales

### 1.2 Collapsible Panel System

**Panel Types**:

1. **Primary Content Panel** (non-collapsible)
   - Contains main data view (process list, performance graphs)
   - Minimum width: 400px
   - Occupies remaining space after side panels

2. **Details Side Panel** (collapsible)
   - Default width: 320px (40 columns = 320px at 8px grid)
   - Collapsed width: 0px (hidden)
   - Collapse trigger: Arrow button in panel header
   - State persistence: Registry key `HKCU\Software\TaskManager\Layout\DetailsPanelCollapsed`
   - Animation: 200ms ease-out slide transition

3. **Metrics Summary Panel** (collapsible)
   - Top-docked horizontal panel
   - Default height: 120px (15 rows)
   - Collapsed height: 48px (title bar only)
   - Shows CPU/Memory/Disk/Network at-a-glance

4. **Filter/Search Panel** (expandable)
   - Default: Single-line search box (40px height)
   - Expanded: Advanced filter interface (160px height)
   - Expand trigger: "Advanced" button or Ctrl+Shift+F

**Panel Resize Behavior**:
- Resize handles: 4px wide, highlight on hover to 8px
- Live preview during resize (no lag tolerance)
- Minimum panel widths enforced (320px for details, 400px for primary)
- Snap zones at 25%, 33%, 50%, 66%, 75% of window width

**Persistence**:
```rust
// Registry structure for layout state
HKCU\Software\TaskManager\Layout\
    PrimaryPanelWidth: DWORD      // pixels
    DetailsPanelCollapsed: DWORD  // 0=visible, 1=collapsed
    MetricsPanelHeight: DWORD     // pixels
    FilterPanelExpanded: DWORD    // 0=compact, 1=expanded
```

### 1.3 Tab Organization & Hierarchy

**Primary Tab Bar** (always visible):

```
┌─────────────────────────────────────────────────┐
│ [Processes] [Performance] [Startup] [Services] │ More ▼
└─────────────────────────────────────────────────┘
```

**Tab Configuration**:
- Height: 48px (6 grid units)
- Active tab indicator: 3px accent color bottom border
- Hover state: Background 8% opacity overlay
- Font: Segoe UI, 12pt, Semibold (active), Regular (inactive)
- Icon size: 16x16px at 100% DPI
- Icon-text spacing: 8px
- Tab padding: 16px horizontal, 12px vertical

**Overflow Menu** ("More" dropdown):
- Shows additional tabs: Details, Users, GPU
- Visible when window width < 1024px
- Dropdown arrow: Segoe Fluent Icons, 12pt

**Tab Switching Performance**:
- Target: <16ms tab content swap
- Implementation: Pre-render hidden tabs in background
- Memory budget: 2MB per pre-rendered tab (5 tabs = 10MB budget)

**Keyboard Navigation**:
- Ctrl+Tab: Next tab (circular)
- Ctrl+Shift+Tab: Previous tab
- Ctrl+1 through Ctrl+6: Direct tab access
- Arrow keys when tab bar focused: Navigate between tabs

### 1.4 Responsive Window Resizing

**Resize Behavior Matrix**:

| Window Width | Behavior |
|--------------|----------|
| 800-1024px | Hide details panel, single-column metrics, compact toolbar |
| 1025-1600px | Show details panel, two-column metrics, full toolbar |
| 1601px+ | Show details panel, three-column metrics, expanded toolbar |

**Real-time Resize Strategy**:
- **During resize** (WM_SIZING): Throttle reflow to 60 FPS (16ms min)
- **After resize** (WM_SIZE): Full reflow with animation completion
- **No blocking**: UI remains interactive during resize
- **Graph handling**: Pause graph updates during drag, resume on release

**DPI Scaling Resize**:
- Monitor DPI change (WM_DPICHANGED): Immediate synchronous reflow
- Per-monitor DPI v2: Calculate layout in logical pixels, render at physical pixels
- Font scaling: Recalculate with new DPI scale factor
- Icon switching: Load appropriate icon size for DPI tier (16px, 24px, 32px)

**Resize Performance Budget**:
- Layout calculation: <4ms
- Direct2D resource recreation: <8ms
- First paint after resize: <16ms (one frame)

### 1.5 Compact vs. Detailed View Modes

**Compact Mode** (FR-049):

Activation: View menu → "Compact Mode" or Ctrl+Alt+C

Visual changes:
- Tab bar height: 48px → 32px
- Row height: 32px → 24px
- Padding reduction: 16px → 8px elements
- Font size: 12pt → 10pt for data cells
- Hide column headers (show on hover)
- Collapse metrics panel to single-row
- Remove all 24px+ spacing

**Size comparison**:
- Standard minimum: 800x600px
- Compact minimum: 640x400px
- Typical compact size: 800x480px (fits on small laptop screens)

**Detailed Mode** (Default):

Full information density:
- Row height: 32px (comfortable reading)
- All metric columns visible
- Expanded details panel
- Visible section headers
- Icon + text labels for all actions

**Toggle Persistence**:
- Registry: `HKCU\Software\TaskManager\ViewMode` (0=Standard, 1=Compact)
- Restore on launch
- Mode affects all tabs consistently

**Mode-Specific Layouts**:

```
Standard Mode Layout:
┌─────────────────────────────────────────────┐
│  Tab Bar (48px)                             │
├─────────────────────────────────────────────┤
│  Metrics Summary (120px)                    │
├─────────────────┬───────────────────────────┤
│                 │  Details Panel            │
│  Process List   │  (320px width)            │
│                 │                           │
│                 │                           │
└─────────────────┴───────────────────────────┘

Compact Mode Layout:
┌─────────────────────────────────────────────┐
│ Tabs(32px) │ CPU:45% Mem:8GB Disk:12MB/s   │
├─────────────────────────────────────────────┤
│                                             │
│  Process List (dense, no details panel)     │
│                                             │
│                                             │
└─────────────────────────────────────────────┘
```

---

## 2. VISUAL DESIGN SYSTEM

### 2.1 Color Palette

**Light Theme** (Windows 11 Light):

```css
/* Base Colors */
--background-primary:      #F3F3F3  /* Main background */
--background-secondary:    #FAFAFA  /* Panels, cards */
--background-tertiary:     #FFFFFF  /* Elevated surfaces */

/* Text Colors */
--text-primary:           #1F1F1F  /* Headlines, body */
--text-secondary:         #605E5C  /* Supporting text */
--text-tertiary:          #8A8886  /* Disabled, hints */
--text-inverse:           #FFFFFF  /* On accent/dark */

/* Accent Colors (from Windows accent color) */
--accent-primary:         System.AccentColor  /* User's Windows accent */
--accent-hover:           System.AccentColor @ 90% brightness
--accent-pressed:         System.AccentColor @ 80% brightness
--accent-disabled:        System.AccentColor @ 40% opacity

/* Semantic Colors */
--success:                #107C10  /* Good status, low usage */
--warning:                #FFA000  /* Medium usage, caution */
--error:                  #D13438  /* High usage, critical */
--info:                   #0078D4  /* Informational */

/* Border & Divider Colors */
--border-strong:          #E1DFDD  /* Panel borders */
--border-subtle:          #EDEBE9  /* Dividers */
--border-focus:           System.AccentColor

/* Elevation & Shadow */
--shadow-elevation-1:     0px 1.6px 3.6px rgba(0,0,0,0.13), 
                          0px 0.3px 0.9px rgba(0,0,0,0.11)
--shadow-elevation-2:     0px 3.2px 7.2px rgba(0,0,0,0.13),
                          0px 0.6px 1.8px rgba(0,0,0,0.11)

/* Fluent Materials */
--mica-tint:              #F3F3F3 @ 70% opacity
--acrylic-tint:           #FCFCFC @ 60% opacity
--acrylic-blur:           30px
```

**Dark Theme** (Windows 11 Dark):

```css
/* Base Colors */
--background-primary:      #202020  /* Main background */
--background-secondary:    #2C2C2C  /* Panels, cards */
--background-tertiary:     #282828  /* Elevated surfaces */

/* Text Colors */
--text-primary:           #FFFFFF  /* Headlines, body */
--text-secondary:         #C8C6C4  /* Supporting text */
--text-tertiary:          #8A8886  /* Disabled, hints */
--text-inverse:           #1F1F1F  /* On accent/light */

/* Accent Colors */
--accent-primary:         System.AccentColor (lightened 10% for dark mode)
--accent-hover:           System.AccentColor @ 110% brightness
--accent-pressed:         System.AccentColor @ 90% brightness
--accent-disabled:        System.AccentColor @ 40% opacity

/* Semantic Colors (adjusted for dark mode) */
--success:                #6CCB5F  /* Good status */
--warning:                #FCE100  /* Medium usage */
--error:                  #FF6B6B  /* High usage */
--info:                   #60CDFF  /* Informational */

/* Border & Divider Colors */
--border-strong:          #3D3D3D  /* Panel borders */
--border-subtle:          #333333  /* Dividers */
--border-focus:           System.AccentColor

/* Elevation & Shadow */
--shadow-elevation-1:     0px 1.6px 3.6px rgba(0,0,0,0.26),
                          0px 0.3px 0.9px rgba(0,0,0,0.22)
--shadow-elevation-2:     0px 3.2px 7.2px rgba(0,0,0,0.26),
                          0px 0.6px 1.8px rgba(0,0,0,0.22)

/* Fluent Materials */
--mica-tint:              #202020 @ 80% opacity
--acrylic-tint:           #2C2C2C @ 50% opacity
--acrylic-blur:           30px
```

**High Contrast Theme** (WCAG 2.1 AA Compliant):

```css
/* Automatically inherits from Windows High Contrast settings */
--background:             SystemColor.Window
--text:                   SystemColor.WindowText
--highlight:              SystemColor.Highlight
--highlight-text:         SystemColor.HighlightText
--button-face:            SystemColor.ButtonFace
--button-text:            SystemColor.ButtonText

/* Minimum contrast ratios enforced: */
/* Normal text (12pt+): 4.5:1 */
/* Large text (14pt+ bold, 18pt+): 3:1 */
/* UI components: 3:1 */
```

### 2.2 Typography System

**Font Family Hierarchy**:

```css
/* Primary Font Stack */
--font-family-primary: "Segoe UI Variable", "Segoe UI", sans-serif;

/* Monospace (for PID, memory values) */
--font-family-mono: "Cascadia Mono", "Consolas", monospace;

/* Icon Font */
--font-family-icons: "Segoe Fluent Icons", "Segoe MDL2 Assets";
```

**Type Scale** (adheres to Windows 11 type ramp):

| Style | Size | Weight | Line Height | Use Case |
|-------|------|--------|-------------|----------|
| **Display** | 28pt | Semibold (600) | 36pt | Page titles (rare) |
| **Title Large** | 20pt | Semibold (600) | 28pt | Tab headers, dialog titles |
| **Title** | 16pt | Semibold (600) | 22pt | Section headers |
| **Subtitle** | 14pt | Semibold (600) | 20pt | Panel titles, group headers |
| **Body Strong** | 12pt | Semibold (600) | 18pt | Emphasized body text |
| **Body** | 12pt | Regular (400) | 18pt | Default UI text, labels |
| **Caption** | 10pt | Regular (400) | 14pt | Supporting text, hints |
| **Caption Strong** | 10pt | Semibold (600) | 14pt | Emphasized small text |
| **Data Large** | 14pt | Regular (400) | 20pt | Primary metrics display |
| **Data Mono** | 11pt | Regular (400) | 16pt | PID, memory values |

**Font Rendering Settings**:
- DirectWrite rendering mode: Natural symmetric
- Antialiasing: ClearType (Windows setting)
- Hinting: Full
- Text contrast: 1.0 (default gamma)

**Variable Font Usage** (Windows 11 22H2+):
- Use Segoe UI Variable for dynamic weight adjustments
- Fallback to Segoe UI on older Windows 10 versions
- Variable axes: Weight (300-700 range used)

**Responsive Typography**:

```
At 100% DPI (96 dpi):
- Use defined pt sizes directly

At 125% DPI (120 dpi):
- Scale all sizes by 1.25x
- 12pt → 15pt effective

At 150% DPI (144 dpi):
- Scale all sizes by 1.5x
- 12pt → 18pt effective

At 200% DPI (192 dpi):
- Scale all sizes by 2.0x
- 12pt → 24pt effective
```

**OpenType Features**:
- Tabular figures: Enabled for numeric data (consistent digit width)
- Proportional figures: Disabled for data columns
- Kerning: Enabled for text labels
- Ligatures: Disabled (technical application context)

### 2.3 Iconography System

**Icon Library**: Segoe Fluent Icons (Windows 11) / Segoe MDL2 Assets (Windows 10 fallback)

**Icon Sizes & DPI Scaling**:

| Size Class | 100% DPI | 125% DPI | 150% DPI | 200% DPI | Usage |
|------------|----------|----------|----------|----------|-------|
| **Small** | 12x12px | 15x15px | 18x18px | 24x24px | Inline icons, status |
| **Regular** | 16x16px | 20x20px | 24x24px | 32x32px | Toolbar, menu, tabs |
| **Medium** | 20x20px | 25x25px | 30x30px | 40x40px | Large buttons |
| **Large** | 24x24px | 30x30px | 36x36px | 48x48px | Feature icons |

**Icon Color Mapping**:

```css
/* State-based coloring */
--icon-default:       --text-secondary
--icon-hover:         --text-primary
--icon-pressed:       --accent-primary
--icon-disabled:      --text-tertiary

/* Semantic icons */
--icon-success:       --success
--icon-warning:       --warning
--icon-error:         --error
--icon-info:          --info
```

**Process State Icons** (consistent visual language):

| Icon Glyph | Unicode | Meaning | Color |
|------------|---------|---------|-------|
| ● | U+E91F | Running normally | --text-secondary |
| ⏸ | U+E769 | Suspended | --warning |
| ⚡ | U+E945 | High priority | --accent-primary |
| 🔒 | U+E72E | System/protected | --text-tertiary |
| ⚠ | U+E7BA | Not responding | --error |
| 🔺 | U+E96D | Elevated (admin) | --warning |

**Resource Usage Icons**:

| Icon | Usage Range | Color Coding |
|------|-------------|--------------|
| ▁ | 0-20% | --success |
| ▃ | 21-50% | --success |
| ▅ | 51-75% | --warning |
| ▇ | 76-90% | --warning |
| █ | 91-100% | --error |

**Icon Rendering**:
- Method: Direct2D DrawText with icon font
- Alignment: Pixel-aligned at all DPI scales
- Optical adjustment: -1px vertical shift for visual centering
- Fallback: Unicode symbols if icon font unavailable

**Custom Icons** (raster, for app icon):

App icon sizes (Windows shell integration):
- 16x16, 20x20, 24x24, 32x32, 40x40, 48x48, 64x64, 96x96, 128x128, 256x256
- Format: ICO file with all sizes embedded
- Design: Consistent with Fluent Design principles

### 2.4 Data Visualization Style Guide

**Chart Color Palettes**:

**Sequential Palette** (single metric over time):
```css
/* Light mode gradient */
--chart-gradient-start:   rgba(0, 120, 212, 0.8)  /* Accent blue */
--chart-gradient-end:     rgba(0, 120, 212, 0.2)

/* Dark mode gradient */
--chart-gradient-start:   rgba(96, 205, 255, 0.8)
--chart-gradient-end:     rgba(96, 205, 255, 0.2)
```

**Categorical Palette** (multiple processes/metrics):
```css
Category colors (accessible contrast, distinct hues):
1:  #0078D4  /* Blue */
2:  #107C10  /* Green */
3:  #CA5010  /* Orange */
4:  #8764B8  /* Purple */
5:  #00B7C3  /* Teal */
6:  #C239B3  /* Magenta */
7:  #FFB900  /* Gold */
8:  #E74856  /* Red */
9:  #00CC6A  /* Mint */
10: #8E8CD8  /* Lavender */
```

**Chart Grid Styling**:
```css
--grid-major-stroke:      --border-subtle
--grid-major-width:       1px
--grid-minor-stroke:      --border-subtle @ 40% opacity
--grid-minor-width:       0.5px
--grid-dash-pattern:      [2, 4]  /* 2px dash, 4px gap */
```

**Chart Axes**:
```css
--axis-stroke:            --border-strong
--axis-width:             1.5px
--axis-label-font:        --font-family-primary, 10pt
--axis-label-color:       --text-secondary
--axis-tick-length:       4px
```

**Graph Line Styles**:
```css
/* Real-time graph (CPU, Memory) */
--line-stroke-width:      2px
--line-stroke-smooth:     true  /* Bezier interpolation */
--line-fill-opacity:      0.3   /* Area under curve */

/* Historical comparison */
--line-stroke-width:      1.5px
--line-stroke-dash:       none  /* Solid for primary */
                          [4, 2] /* Dashed for comparison */
```

**Heat Map Gradient** (for resource usage):
```css
/* 5-stop gradient (cool to hot) */
0%:    #0078D4  /* Blue - low usage */
25%:   #107C10  /* Green - moderate */
50%:   #FFB900  /* Yellow - medium-high */
75%:   #FF8C00  /* Orange - high */
100%:  #E81123  /* Red - critical */
```

**Chart Performance Requirements**:
- Rendering: <8ms per frame (120+ FPS capable)
- Data points: Max 3600 points per series (1 hour at 1Hz)
- Smoothing: Catmull-Rom spline interpolation
- Clipping: Hardware-accelerated clip regions (Direct2D layers)

### 2.5 Animation & Motion

**Animation Timing Functions** (Windows 11 Fluent motion):

```css
/* Standard easing curves */
--ease-standard:      cubic-bezier(0.8, 0, 0.2, 1)    /* Default */
--ease-accelerate:    cubic-bezier(0.9, 0.1, 1, 0.2)  /* Exit */
--ease-decelerate:    cubic-bezier(0.1, 0.9, 0.2, 1)  /* Enter */
--ease-bounce:        cubic-bezier(0.5, 1.5, 0.5, 1)  /* Playful */
```

**Duration Scale**:

| Duration | Use Case | Examples |
|----------|----------|----------|
| **100ms** | Micro-interactions | Hover state, focus ring |
| **150ms** | Fast transitions | Button press, menu open |
| **200ms** | Standard transitions | Panel collapse, tab switch |
| **300ms** | Moderate transitions | Page navigation, dialog appear |
| **400ms** | Slow transitions | Large content changes |
| **500ms** | Extra slow | Rarely used, special effects |

**Motion Specifications by Component**:

**Button Press**:
```
Duration: 100ms
Easing: ease-accelerate
Properties: 
  - Scale: 1.0 → 0.95 (pressed) → 1.0 (released)
  - Background opacity: 0% → 8% (hover) → 12% (pressed)
```

**Panel Collapse/Expand**:
```
Duration: 200ms
Easing: ease-standard
Properties:
  - Width: 320px → 0px (collapse) or 0px → 320px (expand)
  - Opacity: 1.0 → 0.0 (content fades during collapse)
  - Content hidden at 50% animation progress
```

**Tab Switch**:
```
Duration: 150ms
Easing: ease-decelerate
Properties:
  - Old tab content: Opacity 1.0 → 0.0, translateX 0 → -20px
  - New tab content: Opacity 0.0 → 1.0, translateX 20px → 0
  - Crossfade overlap: Yes (both transitions simultaneous)
```

**Graph Update** (real-time data):
```
Duration: 1000ms (matches 1Hz update rate)
Easing: linear (continuous motion)
Properties:
  - Data point position: Shift left by 1-second width
  - New point: Slide in from right edge
  - Smoothing: Bezier curve interpolation between points
```

**Tooltip Appear/Disappear**:
```
Appear:
  Duration: 150ms
  Delay: 500ms (hover dwell time)
  Easing: ease-decelerate
  Properties: Opacity 0 → 1, translateY -4px → 0

Disappear:
  Duration: 100ms
  Delay: 0ms (immediate)
  Easing: ease-accelerate
  Properties: Opacity 1 → 0
```

**Focus Ring**:
```
Duration: 100ms
Easing: ease-standard
Properties:
  - Border width: 0 → 2px
  - Border color: transparent → --border-focus
  - Offset: 2px outside element
```

**Reduced Motion Support** (respects Windows setting):

When `Settings > Accessibility > Visual effects > Animation effects` is OFF:
- All durations: → 0ms (instant)
- Preserve state changes (show final state)
- Maintain layout shifts (no jarring jumps)
- Exception: Real-time graphs (reduce to 30 FPS, disable interpolation)

```cpp
// Win32 detection
BOOL animations_enabled;
SystemParametersInfo(SPI_GETCLIENTAREAANIMATION, 0, &animations_enabled, 0);
if (!animations_enabled) {
    // Disable all UI animations
    animation_multiplier = 0.0;
}
```

**Performance Budget**:
- Animation frame budget: <16ms (60 FPS)
- GPU compositing: All animations use Direct2D layers (hardware-accelerated)
- CPU overhead: <1% during animations
- Memory: Pre-allocate animation intermediate buffers (no runtime allocation)

---

## 3. COMPONENT SPECIFICATIONS

### 3.1 Process List Table

**Layout**:
- Row height: 32px (standard), 24px (compact)
- Header height: 36px (fixed)
- Vertical padding: 8px per cell
- Horizontal padding: 12px per cell

**Column Specifications**:

| Column | Default Width | Min Width | Max Width | Alignment | Format |
|--------|---------------|-----------|-----------|-----------|--------|
| **Icon** | 32px | 32px | 32px | Center | Icon |
| **Name** | 240px | 120px | 600px | Left | Text |
| **PID** | 80px | 60px | 120px | Right | Monospace |
| **Status** | 100px | 80px | 150px | Left | Text + Icon |
| **CPU %** | 80px | 60px | 120px | Right | Percentage |
| **Memory** | 100px | 80px | 150px | Right | Bytes |
| **Disk** | 100px | 80px | 150px | Right | Bytes/s |
| **Network** | 100px | 80px | 150px | Right | Bytes/s |

**Sorting**:
- Visual indicator: Triangle icon in header (▲ asc, ▼ desc)
- Click header: Toggle sort direction
- Shift+click: Multi-column sort (secondary sort)
- Persist sort in registry
- Performance: <5ms re-sort for 2048 processes

**Row States**:

```css
/* Normal row */
background: transparent
border-bottom: 1px solid --border-subtle

/* Hover row */
background: --background-secondary
cursor: pointer

/* Selected row */
background: --accent-primary @ 15% opacity
border: 1px solid --accent-primary

/* Active (focused) row */
background: --accent-primary @ 20% opacity
border: 2px solid --accent-primary
```

**Virtualization** (performance optimization):
- Visible rows only: Render rows in viewport + 10 row buffer
- Row recycling: Reuse row objects for scrolling
- Scroll performance: <16ms per scroll event
- Handle: 2048 processes with <1MB memory for visible rows

---

*Continued in Part 2: Interaction Design, Information Architecture, and Unique Features*
