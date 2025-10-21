# UI/UX Design Specification: Native Task Manager
# Part 2 - Interaction Design, Information Architecture & Features

**Feature**: Native High-Performance Task Manager  
**Spec Reference**: `../spec.md`  
**Created**: 2025-10-21  
**Status**: Design Specification

---

## 4. INTERACTION DESIGN

### 4.1 Keyboard Shortcut System

**Global Application Shortcuts**:

| Shortcut | Action | Scope | Notes |
|----------|--------|-------|-------|
| **Ctrl+F** | Focus search/filter | Global | FR-052 |
| **Ctrl+Shift+F** | Advanced filter panel | Global | Extended functionality |
| **F5** | Refresh/force update | Global | FR-052 |
| **Ctrl+R** | Refresh (alternate) | Global | Windows convention |
| **Delete** | End selected process | Processes tab | FR-052, requires confirmation |
| **Ctrl+E** | Export data | Performance tab | FR-052 |
| **Ctrl+,** | Open settings | Global | Standard preferences shortcut |
| **Alt+F4** | Close application | Global | Windows standard |
| **Ctrl+W** | Close window | Global | Alternate close |
| **Ctrl+T** | New window instance | Global | Power user feature |

**Tab Navigation Shortcuts**:

| Shortcut | Action | Notes |
|----------|--------|-------|
| **Ctrl+Tab** | Next tab (circular) | Standard |
| **Ctrl+Shift+Tab** | Previous tab | Standard |
| **Ctrl+1** | Processes tab | Direct access |
| **Ctrl+2** | Performance tab | Direct access |
| **Ctrl+3** | Startup tab | Direct access |
| **Ctrl+4** | Services tab | Direct access |
| **Ctrl+5** | Details tab | Direct access |
| **Ctrl+6** | Users tab | Direct access |

**View Shortcuts**:

| Shortcut | Action | Context |
|----------|--------|---------|
| **Ctrl+Alt+C** | Toggle compact mode | Global |
| **Ctrl+** (plus) | Zoom in | Global (50%-200%) |
| **Ctrl+-** (minus) | Zoom out | Global |
| **Ctrl+0** | Reset zoom (100%) | Global |
| **Ctrl+Shift+L** | Toggle details panel | Global |
| **Ctrl+Shift+M** | Toggle metrics panel | Global |

**Process Management Shortcuts**:

| Shortcut | Action | Context | Security |
|----------|--------|---------|----------|
| **Delete** | End process | Process selected | Confirmation dialog |
| **Ctrl+Delete** | Force end process | Process selected | Skip graceful shutdown |
| **Shift+Delete** | End process tree | Process selected | Terminate with children |
| **Alt+Enter** | Process properties | Process selected | Show detailed dialog |
| **Ctrl+G** | Go to process | Process list | Jump to PID input |
| **Ctrl+O** | Open file location | Process selected | Explorer integration |

**List Navigation Shortcuts** (when list focused):

| Shortcut | Action |
|----------|--------|
| **↑/↓** | Move selection up/down |
| **Page Up/Down** | Scroll page |
| **Home/End** | First/last item |
| **Ctrl+A** | Select all (multi-select mode) |
| **Ctrl+Click** | Toggle individual selection |
| **Shift+Click** | Range selection |
| **Space** | Toggle selection/expand group |
| **Enter** | Default action (properties) |

**Accessibility Shortcuts**:

| Shortcut | Action | Notes |
|----------|--------|-------|
| **F6** | Cycle focus between panes | Standard Windows |
| **Shift+F10** | Context menu | FR-036, keyboard access |
| **Alt+underlined letter** | Activate menu/button | Mnemonics |
| **Tab** | Next focusable element | FR-051 |
| **Shift+Tab** | Previous focusable element | FR-051 |
| **Esc** | Cancel/close dialog | FR-051 |

**Mnemonic Key Assignments** (Alt+ letter):

```
File menu:        Alt+F
  Export:           E
  Settings:         S
  Exit:             X

View menu:        Alt+V
  Compact Mode:     C
  Details Panel:    D
  Metrics Panel:    M
  Refresh Rate:     R

Options menu:     Alt+O
  Always on Top:    A
  Start Minimized:  S
  Run at Startup:   U

Help menu:        Alt+H
  View Help:        H
  About:            A
```

**Shortcut Conflict Resolution**:
- Windows system shortcuts take precedence
- Custom shortcuts only when window has focus
- Disable conflicting shortcuts in text input fields
- Visual indicator in menus showing shortcuts

**Shortcut Customization** (Phase 2 feature):
- Allow user rebinding in Settings
- Conflict detection UI
- Export/import shortcut profiles
- Reset to defaults option

### 4.2 Mouse Interaction Patterns

**Single Click Behaviors**:

| Target | Action | Visual Feedback |
|--------|--------|-----------------|
| **Process row** | Select process | Row highlight, details panel update |
| **Column header** | Sort by column | Arrow icon, re-order rows |
| **Tab** | Switch tab | Tab activation, content swap |
| **Button** | Execute action | Press animation, state change |
| **Graph** | Show tooltip | Crosshair, value popup |
| **Panel resize handle** | (No action on click) | Cursor change only |

**Double Click Behaviors**:

| Target | Action | Expected Result |
|--------|--------|-----------------|
| **Process row** | Open properties | Modal dialog with full details |
| **Column header** | Auto-resize column | Fit content width |
| **Title bar** | Maximize/restore | Window state change |
| **Graph background** | Reset zoom | Return to default time range |

**Right Click (Context Menus)**:

**Process Row Context Menu**:
```
End Process                    Delete
End Process Tree               Shift+Delete
Set Priority            >      [Submenu: Real-time, High, Above Normal, Normal, Below Normal, Low]
Set Affinity...                [Opens CPU affinity dialog]
───────────────────────────
Go to Details                  Alt+Enter
Open File Location             Ctrl+O
Search Online                  [Opens browser with process name]
Properties                     [Full details dialog]
───────────────────────────
Create Dump File...            [Debug feature]
```

**Graph Context Menu**:
```
Copy Graph Image               Ctrl+C
Export Data...                 Ctrl+E
───────────────────────────
Time Range            >        [1 min, 5 min, 30 min, 1 hour, Custom...]
Show Grid Lines                [Toggle, checkmark when enabled]
Show Averages                  [Toggle]
───────────────────────────
Reset View                     [Zoom/pan reset]
```

**Column Header Context Menu**:
```
✓ Name                         [Checkmark = visible]
✓ PID
✓ CPU %
✓ Memory
  Disk
  Network
✓ Status
───────────────────────────
Select Columns...              [Full column chooser dialog]
Reset to Default               [Restore default column set]
Auto-resize All Columns        [Fit all content]
```

**Hover Behaviors**:

| Element | Hover Duration | Visual Change | Tooltip Delay |
|---------|----------------|---------------|---------------|
| **Button** | Immediate | Background +8% opacity | 500ms |
| **Row** | Immediate | Background highlight | None |
| **Graph point** | Immediate | Crosshair + tooltip | 150ms |
| **Tab** | Immediate | Background tint | 800ms |
| **Icon** | Immediate | Color shift to primary | 600ms |
| **Column resize** | Immediate | Cursor change (⇔) | None |

**Drag and Drop**:

**Draggable Elements**:
1. **Column Headers** (reorder columns)
   - Drag threshold: 4px movement
   - Visual: Ghost column header follows cursor
   - Drop zones: Between other headers (vertical separator highlight)
   - Drop feedback: Animated column swap
   - Persistence: Save order to registry

2. **Panel Resize Handles**
   - Drag: Live resize with immediate redraw
   - Constraint: Min/max panel widths enforced
   - Cursor: Resize arrows (⇔ horizontal, ⇕ vertical)
   - Performance: <16ms per drag update

3. **Window** (standard Windows behavior)
   - Drag title bar: Move window
   - Aero Snap: Edge/corner docking
   - Multi-monitor: Move across screens

**Mouse Wheel Behaviors**:

| Context | Action | Modifier | Result |
|---------|--------|----------|--------|
| **Process list** | Scroll | None | Vertical scroll (3 rows per tick) |
| **Graph area** | Scroll | None | Time range adjustment (zoom horizontal) |
| **Graph area** | Scroll | Ctrl | Zoom in/out (vertical scale) |
| **Anywhere** | Scroll | Ctrl | Global zoom (50%-200%) |

**Cursor States**:

| Cursor | When | Visual |
|--------|------|--------|
| **Default** | General UI | Arrow |
| **Pointer** | Hovering clickable | Hand |
| **Text** | Over text/input | I-beam |
| **Resize H** | Column separator | ⇔ |
| **Resize V** | Panel separator | ⇕ |
| **Wait** | Processing | Spinning circle |
| **Not Allowed** | Invalid drop target | 🚫 |

### 4.3 Touch-Friendly Design

**Touch Target Sizing** (Windows 11 Guidelines):

```
Minimum touch target: 44x44px (11mm at 96 DPI)
Recommended: 48x48px (12mm)
Spacing between targets: ≥8px
```

**Touch-Optimized Components**:

**Button Sizing**:
- Standard mode: 32px height (meets minimum)
- Touch mode: 48px height (activated automatically on tablet/touch)
- Width: Auto with min 80px

**List Row Sizing**:
- Standard: 32px height
- Touch mode: 48px height
- Selection: Entire row clickable (not just text)

**Scrollbar Width**:
- Standard: 12px width (Windows default)
- Touch mode: 20px width (easier to grab)
- Auto-hide: Fade out after 2s inactivity

**Touch Gestures**:

| Gesture | Action | Context |
|---------|--------|---------|
| **Tap** | Select/activate | Equivalent to click |
| **Double tap** | Properties/maximize | Same as double click |
| **Long press** | Context menu | Same as right click (800ms dwell) |
| **Swipe horizontal** | Switch tabs | Fluid tab navigation |
| **Swipe vertical** | Scroll list | Natural scrolling |
| **Pinch** | Zoom graph | Time range adjustment |
| **Two-finger drag** | Pan graph | Move viewport |

**Touch Mode Activation**:

```cpp
// Auto-detect touch capability
if (GetSystemMetrics(SM_DIGITIZER) & NID_INTEGRATED_TOUCH) {
    enable_touch_mode = true;
    increase_target_sizes();
}

// Manual toggle in View menu
View > Touch Mode [Checkbox]
```

**Touch Feedback**:
- Press: 100ms scale animation (1.0 → 0.95)
- Ripple effect: Circular expand from touch point (200ms duration)
- Haptic: Not available on Windows desktop (skip)

### 4.4 Focus System & Keyboard Navigation

**Focus Ring Specification**:

```css
/* Focus indicator (FR-051, accessibility) */
--focus-ring-width:      2px
--focus-ring-color:      --border-focus (System.AccentColor)
--focus-ring-offset:     2px (outside element)
--focus-ring-radius:     4px (rounded corners)
--focus-ring-style:      solid
```

**Focus Order** (logical tab sequence):

```
1. Menu bar (if visible)
2. Search/filter input
3. Tab bar tabs (left to right)
4. Toolbar buttons (left to right)
5. Primary content area (list/graph)
6. Details side panel (if visible)
   a. Section headers (collapsible)
   b. Content within sections (top to bottom)
7. Status bar (if interactive elements present)
```

**Focus Trap** (modal dialogs):
- Tab cycles within dialog only
- Shift+Tab reverse cycles
- Esc closes dialog, returns focus to trigger element
- First focusable element gets focus on open

**Focus Restoration**:
- Store focus path before context switch
- Restore focus after tab switch
- Restore focus after dialog close
- Persist focus across sessions (registry)

**Visual Focus Indicators**:

| Component | Focus Style |
|-----------|-------------|
| **Button** | 2px accent border + 2px offset |
| **List row** | Full row border, 2px accent |
| **Text input** | Border color change + blinking cursor |
| **Tab** | Underline + accent color text |
| **Graph** | Dashed border around graph area |
| **Checkbox** | Border + fill background |

**Keyboard Navigation Path Example**:

```
Processes Tab Focus Flow:
1. Search box [text input]
2. "Advanced" button [button]
3. Tab bar - Processes [tab]
4. Column header - Name [sortable header]
5. Column header - PID [sortable header]
6. ... (all column headers)
7. Process list row 1 [selectable row]
   - Arrow keys navigate within list
   - Tab escapes list to next component
8. Details panel header [collapsible]
9. Details panel - Process name [text]
10. Details panel - PID label [text]
... (all details panel elements)
11. Status bar - Refresh rate dropdown [combobox]
```

### 4.5 Contextual Help System

**Help Access Points**:

| Trigger | Action | Content Type |
|---------|--------|--------------|
| **F1** | Context-sensitive help | Opens help dialog for current focus |
| **?** icon in title bar | General help | Opens main help window |
| **Hover tooltips** | Quick hints | Inline tooltip (500ms delay) |
| **"Learn more" links** | Feature details | Opens help section or external URL |

**Tooltip System**:

**Tooltip Types**:

1. **Quick Hint** (most common):
```
Duration: 5s auto-dismiss
Delay: 500ms hover dwell
Content: 1-2 sentences max
Font: Caption (10pt)
Max width: 300px
Example: "End the selected process immediately"
```

2. **Extended Tooltip** (complex features):
```
Duration: 10s auto-dismiss (or manual dismiss ×)
Delay: 1000ms hover dwell
Content: Multiple paragraphs + optional image
Font: Body (12pt)
Max width: 400px
Example: CPU affinity dialog - explains core assignment
```

3. **Error Tooltip** (validation):
```
Duration: Until focus change
Delay: Immediate on error
Content: Error message + resolution steps
Color: --error background
Example: "PID must be a number between 1 and 65535"
```

**Tooltip Positioning**:
```
Preferred: Below element (8px gap)
Fallback if near bottom: Above element
Horizontal: Center-aligned to trigger
Arrow: Points to trigger element (8px triangle)
```

**Help Dialog Structure**:

```
┌─────────────────────────────────────────────────┐
│  Task Manager Help                          × │
├─────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────────┐   │
│ │  Contents       │  │  Getting Started    │   │
│ │  ───────────    │  │                     │   │
│ │  • Overview     │  │  Welcome to Task   │   │
│ │  • Processes    │  │  Manager! This app │   │
│ │  • Performance  │  │  helps you monitor │   │
│ │  • Keyboard     │  │  and manage...     │   │
│ │  • Shortcuts    │  │                     │   │
│ │  • FAQ          │  │  [Screenshot]      │   │
│ │                 │  │                     │   │
│ │                 │  │  Next: Processes → │   │
│ └─────────────────┘  └─────────────────────┘   │
│                                                 │
│ [Search help...]                    [Close]    │
└─────────────────────────────────────────────────┘
```

**Embedded Help** (in-app guidance):

**First Run Experience**:
```
On first launch (no registry settings):
1. Show welcome tooltip near main UI
   "Welcome! Press Ctrl+F to search for processes"
2. Highlight key features with pulsing indicators
3. Dismissible with "Don't show again" checkbox
```

**Empty States** (instructional):
```
When no processes match filter:
┌─────────────────────────────────────┐
│                                     │
│         🔍                          │
│                                     │
│    No processes found               │
│                                     │
│    Try clearing your filter or      │
│    search for a different term      │
│                                     │
│    [Clear Filter]                   │
└─────────────────────────────────────┘
```

**Inline Help Icons** (? button):
```
Placement: Next to complex settings
Action: Show extended tooltip on click/hover
Example: "Refresh Rate [?]" → Explains impact on CPU usage
```

---

## 5. INFORMATION ARCHITECTURE

### 5.1 Metric Organization by Importance

**Primary Metrics** (always visible, top-level):

**Priority 1 - Critical System Health**:
```
CPU Usage %           [Large, prominent]
Memory Usage          [Large, prominent]
Active Processes      [Count, medium emphasis]
```

**Priority 2 - Active Resource Consumption**:
```
Disk Activity         [Medium emphasis]
Network Activity      [Medium emphasis]
GPU Usage (if present) [Medium emphasis]
```

**Priority 3 - Contextual Details**:
```
Uptime                [Small, status bar]
Handles               [Small, status bar]
Threads               [Small, status bar]
```

**Metric Grouping by Relationship**:

**Resource Consumption Group**:
- CPU %
- Memory (Working Set, Commit, Private)
- Disk (Read, Write, Total)
- Network (Send, Receive, Total)
- GPU (Memory, Engine %, Encode, Decode)

**Process Characteristics Group**:
- PID
- Name
- Status
- User
- Security Context (Integrity Level)

**Performance Indicators Group**:
- Threads
- Handles
- CPU Time
- I/O Operations
- Page Faults

**Hierarchical Relationship Visualization**:

```
System
├── CPU
│   ├── Total Usage %
│   ├── Per-Core Usage %
│   ├── Core 0 [Graph]
│   ├── Core 1 [Graph]
│   └── ...
├── Memory
│   ├── Total Used / Total Available
│   ├── Committed / Commit Limit
│   ├── Cached
│   ├── Paged Pool
│   └── Non-paged Pool
├── Disk
│   ├── Disk 0 (C:)
│   │   ├── Active Time %
│   │   ├── Read Speed
│   │   └── Write Speed
│   └── Disk 1 (D:)
│       └── ...
└── Network
    ├── Adapter 1 (Ethernet)
    │   ├── Send
    │   ├── Receive
    │   └── Utilization %
    └── Adapter 2 (Wi-Fi)
        └── ...
```

### 5.2 Progressive Disclosure

**3-Tier Information Density**:

**Tier 1: Glance View** (default, compact mode):
- 5-10 key metrics visible
- Single-line summaries
- Color-coded status indicators
- Minimal text, max iconography

```
Example Compact View:
┌──────────────────────────────┐
│ CPU: 45% ▅  Mem: 8.2GB ▃    │
│ Disk: 12MB/s  Net: 1.2MB/s  │
└──────────────────────────────┘
```

**Tier 2: Standard View** (default state):
- 20-30 metrics visible
- Multi-line details
- Grouped by category
- Text labels + values

```
Example Standard View:
┌─────────────────────────────────────┐
│ CPU Usage:  45%  ▅▅▅▅▅▃▃▃▁▁         │
│   Processes:         142            │
│   Threads:          3,845           │
│                                     │
│ Memory:  8.2 GB / 16 GB  ▃▃▃▃▁▁▁▁   │
│   Cached:  2.1 GB                   │
│   Available: 7.8 GB                 │
└─────────────────────────────────────┘
```

**Tier 3: Detailed View** (details panel, properties dialog):
- 50+ metrics visible
- Full hierarchical data
- All available details
- Technical specifications

```
Example Detailed View:
┌─────────────────────────────────────┐
│ Process: chrome.exe (PID: 12456)    │
│                                     │
│ Performance                         │
│   CPU:             2.3%             │
│   CPU Time:        0:05:23          │
│   Threads:         28               │
│   Handles:         1,245            │
│                                     │
│ Memory (Private Working Set)        │
│   Working Set:     245 MB           │
│   Private:         198 MB           │
│   Commit:          312 MB           │
│   Paged Pool:      2.1 MB           │
│   NP Pool:         128 KB           │
│   Page Faults:     45,234           │
│                                     │
│ I/O                                 │
│   Read:            1.2 MB/s         │
│   Write:           34 KB/s          │
│   Other:           12 KB/s          │
│   Total Bytes:     2.4 GB           │
│                                     │
│ [More Details...]                   │
└─────────────────────────────────────┘
```

**Expansion Controls**:
- ▶ Collapsed section (click to expand)
- ▼ Expanded section (click to collapse)
- "Show more" links at section bottoms
- Ctrl+Click header: Expand all sections
- Remember expansion state per session

### 5.3 Notification & Alert Hierarchy

**Alert Severity Levels**:

| Level | Color | Icon | Use Case | Dismissible |
|-------|-------|------|----------|-------------|
| **Info** | --info (blue) | ℹ | Non-critical status updates | Auto (5s) |
| **Success** | --success (green) | ✓ | Confirmation of actions | Auto (3s) |
| **Warning** | --warning (yellow) | ⚠ | Resource approaching limits | Manual |
| **Error** | --error (red) | ✗ | Critical failures, access denied | Manual |
| **Critical** | --error (red, pulsing) | ⛔ | System stability risk | Manual + Sound |

**Alert Positioning**:

```
Top-right toast notifications:
┌──────────────────────────────────┐
│                     ┌──────────┐ │
│                     │  Toast   │ │
│                     │  Alert   │ │
│                     └──────────┘ │
│  [Main Content]                  │
│                                  │
└──────────────────────────────────┘

In-line warnings (near affected element):
┌──────────────────────────────────┐
│  ⚠ CPU usage above 90%           │
│  Consider closing programs  [×]  │
├──────────────────────────────────┤
│  [Affected graph/metric]         │
└──────────────────────────────────┘
```

**Alert Examples by Type**:

**Info Alert**:
```
ℹ  Monitoring refresh rate changed to 5 seconds
   [3 second auto-dismiss]
```

**Success Alert**:
```
✓  Process "notepad.exe" (PID: 5432) terminated successfully
   [3 second auto-dismiss]
```

**Warning Alert**:
```
⚠  Memory usage at 85% (13.6 GB / 16 GB)
   Close unused applications to free memory
   [×]  [Dismiss]
```

**Error Alert**:
```
✗  Access Denied: Cannot terminate system process
   Administrator privileges required. Click to elevate.
   [Elevate] [Cancel]
```

**Critical Alert**:
```
⛔  Critical: CPU temperature above 95°C
   Thermal throttling active. Check cooling system.
   [Dismiss]  [View Details]
   [Pulsing red border, system sound]
```

**Alert Batching**:
- Multiple similar alerts: Combine into single notification
- Example: "5 processes terminated" instead of 5 separate toasts
- Batch window: 2 seconds

**Alert History**:
- Access: Click status bar bell icon 🔔
- Shows last 50 alerts
- Grouped by session
- Filterable by severity

### 5.4 Error Message Format

**Error Message Structure**:

```
[Icon] [Error Title]
[Clear explanation in plain language]
[What went wrong specifically]
[What user can do to resolve]
[Technical details] (collapsible)
[Action Buttons]
```

**Example Error Messages**:

**Access Denied Error**:
```
┌────────────────────────────────────────────┐
│  ✗  Cannot Terminate Process               │
│                                            │
│  Access denied when trying to end         │
│  "svchost.exe" (PID: 1234).               │
│                                            │
│  This is a protected system process that  │
│  requires administrator privileges.       │
│                                            │
│  To terminate this process:               │
│  • Click "Run as Administrator" below     │
│  • Right-click app icon → Run as admin    │
│                                            │
│  ▶ Technical Details                      │
│    Error: ERROR_ACCESS_DENIED (5)         │
│    Process Integrity: System              │
│    Required Privilege: SeDebugPrivilege   │
│                                            │
│  [Run as Administrator]  [Cancel]         │
└────────────────────────────────────────────┘
```

**Data Export Error**:
```
┌────────────────────────────────────────────┐
│  ✗  Export Failed                          │
│                                            │
│  Unable to save performance data to:      │
│  C:\Users\...\performance_data.csv        │
│                                            │
│  The file may be open in another program  │
│  or you may not have write permissions.   │
│                                            │
│  To fix this:                             │
│  • Close the file if it's open            │
│  • Choose a different save location       │
│  • Check folder permissions               │
│                                            │
│  ▶ Technical Details                      │
│    Error: ERROR_SHARING_VIOLATION (32)    │
│    Path: C:\Users\...\performance...      │
│                                            │
│  [Choose Different Location]  [Retry]     │
│  [Cancel]                                  │
└────────────────────────────────────────────┘
```

**Network Connection Error**:
```
┌────────────────────────────────────────────┐
│  ⚠  Limited Network Data                   │
│                                            │
│  Network monitoring is showing incomplete │
│  data due to a system configuration issue.│
│                                            │
│  Some network connections may not appear  │
│  in the list. Core functionality still    │
│  works normally.                           │
│                                            │
│  This typically happens when:             │
│  • Windows Firewall is blocking access    │
│  • Network driver is outdated             │
│  • Running in restricted environment      │
│                                            │
│  ▶ Technical Details                      │
│    API: GetExtendedTcpTable failed        │
│    Error: ERROR_ACCESS_DENIED (5)         │
│                                            │
│  [Continue]  [Learn More]                 │
└────────────────────────────────────────────┘
```

**Error Message Guidelines**:
1. **Use plain language** - No jargon in main message
2. **Explain impact** - What functionality is affected
3. **Provide solution** - Actionable steps, not just problem
4. **Technical details collapsible** - For power users, support
5. **Appropriate tone** - Professional, helpful, not blaming user
6. **Specific, not generic** - Include affected entity (file path, PID)

### 5.5 Search & Filter Interaction Model

**Search/Filter Input**:

```
┌──────────────────────────────────────────┐
│  🔍  Search processes...  [×]  [≡]       │
└──────────────────────────────────────────┘
    │                         │     │
    │                         │     └─ Advanced filters (dropdown)
    │                         └─ Clear search (×)
    └─ Search icon (click to focus)
```

**Real-time Filtering** (FR-005, <100ms):
- Filter as user types (no "Search" button needed)
- Debounce: 50ms (balance responsiveness vs performance)
- Highlight matches in results
- Show match count: "Showing 5 of 142 processes"

**Search Syntax** (power user features):

```
Basic text:        "chrome"
  → Matches: Any process name containing "chrome"

Case-sensitive:    "Chrome" (capital = case-sensitive)
  → Matches: Exact case "Chrome" only

Wildcards:         "chr*"
  → Matches: chrome.exe, chromium.exe, etc.

Field-specific:    "cpu:>50"
  → Matches: Processes using >50% CPU

Multiple criteria: "chrome cpu:>10"
  → Matches: Chrome processes using >10% CPU

Exclusion:         "!system"
  → Matches: All except system processes

PID search:        "pid:1234"
  → Matches: Process with PID 1234
```

**Advanced Filter Panel** (Ctrl+Shift+F):

```
┌────────────────────────────────────────────────┐
│  Advanced Filters                              │
├────────────────────────────────────────────────┤
│  Process Name:     [chrome.exe      ]          │
│                                                │
│  Resource Usage:                               │
│    CPU:   [ 0% ] ═══════╸════ [ 100% ]        │
│    Memory:[ 0MB] ═══════╸════ [ 16GB ]        │
│                                                │
│  Status:           [All  ▼]                    │
│  User:             [All  ▼]                    │
│  Integrity Level:  [All  ▼]                    │
│                                                │
│  □ Show only elevated processes                │
│  □ Show only .NET/managed processes            │
│  ☑ Hide system processes                       │
│                                                │
│  [Reset]  [Apply]  [Save as Preset...]         │
└────────────────────────────────────────────────┘
```

**Filter Presets** (quick access):

```
Saved Filter Dropdown:
┌─────────────────────────┐
│  High CPU Usage         │
│  Memory Leaks           │
│  My Processes Only      │
│  Elevated Processes     │
│  ───────────────────    │
│  Manage Presets...      │
└─────────────────────────┘

Preset Definition Example:
"High CPU Usage" = cpu:>50 AND !system
```

**Active Filter Indicator**:

```
When filters active:
┌──────────────────────────────────────────┐
│  🔍  chrome  [×]                         │
│  ⓘ  Active: 1 filter | Showing 5 of 142 │
└──────────────────────────────────────────┘

Clear button:
  [×] next to search = Clear search text only
  [Clear All Filters] = Reset all criteria
```

---

## 6. UNIQUE FEATURES

### 6.1 Process Relationship Visualization

**Tree View Mode**:

```
Hierarchical Process Tree:
┌────────────────────────────────────────────┐
│  ▼ explorer.exe (PID: 1234)                │
│    ▼ chrome.exe (PID: 5678)                │
│      ├─ chrome.exe (PID: 5680) --renderer  │
│      ├─ chrome.exe (PID: 5681) --renderer  │
│      └─ chrome.exe (PID: 5682) --gpu       │
│    ▶ notepad.exe (PID: 9876)               │
│  ▶ System (PID: 4)                         │
│  ▶ services.exe (PID: 800)                 │
└────────────────────────────────────────────┘

Legend:
  ▼ = Expanded (showing children)
  ▶ = Collapsed (has children, hidden)
  ├─ = Child process (more siblings)
  └─ = Last child process
```

**Visual Encoding**:
- Indentation: 24px per hierarchy level
- Connector lines: 1px subtle border color
- Parent process: Bold text
- Child processes: Regular text
- Orphaned processes: Red text (parent terminated)

**Interaction**:
- Click ▶: Expand to show children
- Click ▼: Collapse to hide children
- Right-click process: "Expand All Children" option
- Ctrl+Click: Expand entire tree
- Color-code by CPU usage (inherit from parent aggregate)

**Graph Visualization** (Advanced mode):

```
Process Dependency Graph:
     ┌──────────┐
     │ explorer │
     └────┬─────┘
          │
     ┌────┴─────┬────────┐
     │          │        │
  ┌──▼───┐  ┌──▼──┐  ┌──▼────┐
  │chrome│  │Edge │  │Notepad│
  └──┬───┘  └─────┘  └───────┘
     │
  ┌──┴──┬──────┐
  │     │      │
┌─▼─┐ ┌─▼─┐ ┌─▼─┐
│GPU│ │R1 │ │R2 │
└───┘ └───┘ └───┘

Legend:
  Rectangle = Process
  Arrow = Parent → Child
  Size ∝ Memory usage
  Color = CPU usage (heat map)
```

**Layout Algorithm**:
- Top-down hierarchy (parent above children)
- Minimize edge crossings
- Equal spacing between siblings
- Cluster by process group

### 6.2 Resource Usage Heat Maps

**CPU Heat Map View**:

```
Time-based Heat Map (last hour):
┌────────────────────────────────────────────┐
│        00:00    00:15    00:30    00:45    │
│ PID    ↓       ↓        ↓        ↓         │
├────────────────────────────────────────────┤
│ 1234 │ ░░░░░░ ▓▓▓▓▓▓ ░░░░░░ ░░░░░░        │
│ 5678 │ ▓▓▓▓▓▓ ▓▓▓▓▓▓ ████████ ░░░░░░        │
│ 9012 │ ░░░░░░ ░░░░░░ ▓▓▓▓▓▓ ▓▓▓▓▓▓        │
│ 3456 │ ████████ ████████ ████████ ████████        │
└────────────────────────────────────────────┘

Color Scale (CPU %):
░░ = 0-25%   (Cool, low usage)
▓▓ = 26-50%  (Moderate)
▒▒ = 51-75%  (High)
██ = 76-100% (Critical, hot)
```

**Memory Heat Map** (per-process over time):

```
┌────────────────────────────────────────────┐
│        Process Memory Usage (Last Hour)    │
├────────────────────────────────────────────┤
│                                            │
│  High │                                    │
│   │   │         ███                        │
│  16GB │       ██░░░██                      │
│   │   │     ██░░░░░░░██                    │
│   8GB │   ██▓▓▓░░░░░▓▓▓██                  │
│   │   │ ██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██                │
│   0GB └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─            │
│       00:00   00:30   01:00  (time)        │
│                                            │
│  Select process: [chrome.exe ▼]           │
└────────────────────────────────────────────┘
```

**Heat Map Interactions**:
- Hover cell: Tooltip shows exact value + timestamp
- Click cell: Jump to that time in detail view
- Zoom: Scroll to adjust time granularity
- Export: Save heat map as image or CSV

### 6.3 Timeline Correlation Interface

**Multi-Metric Timeline**:

```
Synchronized Timeline View:
┌────────────────────────────────────────────────────┐
│  CPU  │ ████▓▓░░░░▓▓████░░░░░░████                 │
│  Mem  │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓                 │
│  Disk │ ░░░░████░░░░░░░░████░░░░░░                 │
│  Net  │ ░░░░░░░░░░░░░░████████░░░░                 │
│       └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─            │
│       00:00    00:15    00:30    00:45             │
│                                                    │
│  ┃← Playhead at 00:23:45                          │
│  ├─ CPU: 78% (spike detected)                     │
│  ├─ Mem: 12.4 GB (normal)                         │
│  ├─ Disk: 145 MB/s (write burst)                  │
│  └─ Net: 2.3 MB/s (normal)                        │
│                                                    │
│  Event Markers:                                   │
│  ⬇ 00:10 - Process "backup.exe" started           │
│  ⚠ 00:23 - CPU spike (threshold exceeded)         │
│  ⬇ 00:35 - High disk activity                     │
└────────────────────────────────────────────────────┘

Controls:
◄◄  ◄  ▌▌  ►  ►►  [Speed: 1x ▼]  [Export]
```

**Correlation Detection** (automatic):
- Identify simultaneous metric spikes
- Highlight causation candidates
- Example: "Disk spike coincided with process X start"

**Event Annotation**:
- User can add markers: Right-click timeline → "Add Marker"
- Marker types: Note, Issue, Milestone
- Persist markers across sessions

### 6.4 Custom Dashboard Creation

**Dashboard Builder** (drag-and-drop):

```
┌──────────────────────────────────────────────────┐
│  Dashboard: My Performance View     [Edit Mode]  │
├──────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────────────┐  │
│  │  CPU Graph     │  │  Memory Graph          │  │
│  │  [Configure]   │  │  [Configure]           │  │
│  │                │  │                        │  │
│  │  [Live data]   │  │  [Live data]           │  │
│  └────────────────┘  └────────────────────────┘  │
│                                                  │
│  ┌──────────────────────────────────────────┐    │
│  │  Top 10 Processes by CPU                 │    │
│  │  [Table widget]                          │    │
│  │  1. chrome.exe    45%                    │    │
│  │  2. System        12%                    │    │
│  │  ...                                     │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  [+ Add Widget]  [Save]  [Exit Edit Mode]       │
└──────────────────────────────────────────────────┘

Available Widgets (drag from palette):
- CPU Graph (single/multi-core)
- Memory Graph
- Disk Activity
- Network Activity
- Top Processes (configurable metric)
- Metric Gauge (single value, large display)
- Alert Feed (recent warnings/errors)
- Process Tree (live)
```

**Widget Configuration**:
- Double-click widget: Open settings
- Resize: Drag corners (snap to grid)
- Move: Drag widget to new position
- Delete: X button or Delete key

**Dashboard Presets**:
```
Built-in:
1. Overview (default)
2. Performance Analysis (graphs focused)
3. Security Monitor (elevation, integrity levels)
4. Developer (per-process details)

User-created:
5. My Gaming Setup
6. Server Monitoring
...
```

### 6.5 Monitoring Alert System

**Alert Rule Configuration**:

```
┌────────────────────────────────────────────────┐
│  Create Monitoring Alert                      │
├────────────────────────────────────────────────┤
│  Alert Name:  [High CPU Usage          ]      │
│                                                │
│  Condition:                                    │
│    Metric:     [CPU Usage        ▼]           │
│    Operator:   [Greater than     ▼]           │
│    Threshold:  [80         ] %                 │
│    Duration:   [30         ] seconds           │
│                                                │
│  Actions:                                      │
│    ☑ Show notification                         │
│    ☑ Play sound: [Windows Default ▼]          │
│    ☑ Log to event log                          │
│    □ Run command: [              ]  [Browse]   │
│                                                │
│  Advanced:                                     │
│    ▶ Schedule (when to monitor)                │
│    ▶ Filters (apply to specific processes)     │
│                                                │
│  [Test Alert]  [Save]  [Cancel]                │
└────────────────────────────────────────────────┘
```

**Alert Examples**:

1. **High CPU Alert**:
   - Trigger: CPU > 90% for 1 minute
   - Action: Toast notification + sound
   - Color: Warning (yellow)

2. **Memory Leak Detection**:
   - Trigger: Process memory increasing >100MB/min for 5 min
   - Action: Warning notification with process name
   - Suggested action: "Consider restarting process"

3. **Process Crash Alert**:
   - Trigger: Process exits with error code ≠ 0
   - Action: Error notification + Event Log entry
   - Show: Process name, PID, exit code

4. **Disk Space Alert**:
   - Trigger: Free space < 5 GB
   - Action: Critical notification
   - Suggested action: "Free up disk space"

**Alert Management Interface**:

```
┌────────────────────────────────────────────────┐
│  Monitoring Alerts                 [+ New]     │
├────────────────────────────────────────────────┤
│  ☑ High CPU Usage             Last: 2min ago  │
│  ☑ Memory Leak Detection      Last: Never     │
│  □ Process Crash Alert         (Disabled)      │
│  ☑ Low Disk Space              Last: 5hrs ago │
│                                                │
│  Alert History:                                │
│  ⚠ 14:23 - High CPU Usage (chrome.exe, 95%)   │
│  ⚠ 12:45 - Low Disk Space (C:, 3.2 GB)        │
│  ✗ 11:30 - Process Crash (app.exe, code -1)   │
│                                                │
│  [Configure]  [Disable All]  [Clear History]  │
└────────────────────────────────────────────────┘
```

**Alert Throttling**:
- Same alert: Don't repeat within 5 minutes
- Similar alerts: Batch into single notification
- Alert storm protection: Max 10 alerts per minute

---

## 7. ACCESSIBILITY SPECIFICATIONS

### 7.1 Screen Reader Support

**UI Automation Properties** (FR-053, FR-057):

```cpp
// Example: Process list row UIA properties
element.AutomationId = "ProcessRow_1234"
element.Name = "chrome.exe, PID 1234, CPU 23%, Memory 450 MB"
element.LocalizedControlType = "Data Item"
element.HelpText = "Double-click for details, Delete to end process"
element.AcceleratorKey = "Delete"
element.AccessKey = "" // None for list items
```

**Announcement Priorities**:

| Event | ARIA Live | Announcement |
|-------|-----------|--------------|
| Process terminated | Assertive | "Process chrome.exe terminated" |
| Filter applied | Polite | "Showing 12 of 150 processes" |
| Tab switched | Polite | "Performance tab selected" |
| Alert triggered | Assertive | "Warning: CPU usage at 95%" |
| Background update | Off | (Silent, no announcement) |

**Screen Reader Testing**:
- Primary: NVDA (free, most common)
- Secondary: JAWS (enterprise standard)
- Tertiary: Narrator (Windows built-in)

### 7.2 High Contrast Mode

**Automatic Theme Switching** (FR-054):

```cpp
// Detect high contrast mode
HIGHCONTRAST hc = {sizeof(HIGHCONTRAST)};
SystemParametersInfo(SPI_GETHIGHCONTRAST, sizeof(hc), &hc, 0);
if (hc.dwFlags & HCF_HIGHCONTRASTON) {
    apply_high_contrast_theme();
}

// Listen for theme changes
WM_THEMECHANGED → Reapply theme
WM_SETTINGCHANGE → Check if high contrast toggled
```

**High Contrast Overrides**:
- All custom colors: → System colors
- All shadows/gradients: → Removed
- Border widths: → 2px minimum (increased visibility)
- Icon outlines: → Added 1px contrasting border

### 7.3 Keyboard-Only Operation

**No Mouse Required** (FR-051):
- 100% feature parity via keyboard
- Every action has keyboard equivalent
- Visual focus indicators always visible
- Shortcut cheat sheet: F1 → "Keyboard Shortcuts"

### 7.4 Zoom & Scaling

**Independent Zoom** (FR-055, 50%-200%):

```
Zoom Levels:
50%  - Ultra compact (advanced users)
75%  - Reduced
100% - Default
125% - Comfortable
150% - Large
175% - Extra large
200% - Maximum

Zoom affects:
✓ Font sizes
✓ Icon sizes
✓ Control padding/spacing
✓ Graph line widths
✗ Window chrome (native, system-controlled)
```

**Zoom Persistence**:
- Registry: `HKCU\Software\TaskManager\ZoomLevel`
- Per-user setting (not machine-wide)
- Restore on launch

---

**END OF PART 2**

For implementation details, refer to:
- Part 1: Layout Architecture, Visual Design, Typography, Iconography, Data Visualization, Animations
- Spec: `../spec.md` (functional requirements)
- Plan: `../plan.md` (implementation phases)
- Checklists: `../checklists/ux.md` (quality validation)
