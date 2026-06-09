# EGUI Orbit Downloader GUI Layer — Design Spec

## Overview

Add an egui-based desktop GUI layer to the existing download manager. The GUI recreates the classic Orbit Downloader layout while keeping the existing CLI/TUI fully intact. This is a pure additive layer — no backend changes, no refactoring.

## Launch Integration

A `--gui` flag added to `main.rs`:
- `cargo run -- --gui ...args` → launches eframe window
- `cargo run ...args` → existing TUI behavior, unchanged

## Architecture

### Module structure

```
Cargo.toml         +eframe, +egui, +egui_extras, +epi

src/
├── main.rs        + --gui flag routes to gui::run_gui() or TUI
├── download.rs     unchanged
├── state.rs        unchanged
├── tui.rs          unchanged

├── models/
│   ├── mod.rs
│   └── download.rs     DownloadItem, DownloadStatus, DownloadProvider trait

├── gui/
│   ├── mod.rs          run_gui() entry point, re-exports
│   ├── app.rs          EframeApp: eframe::App, layout tree, UiSettings
│   ├── menu.rs         File/Edit/View/Tools/Help dropdown menus
│   ├── toolbar.rs      New/Start/Pause/Delete/Scheduler/Report/Preferences
│   ├── sidebar.rs      ~220px resizable panel, categories + info box
│   ├── downloads.rs    egui_extras::TableBuilder download list
│   ├── bottom_panel.rs draggable splitter, Progress/Properties/Log tabs
│   ├── statusbar.rs    status left + hyperlinks right
│   └── theme.rs        compact WinXP/7 visual overrides

└── widgets/
    └── segmented_progress.rs   Orbit-style block grid widget
```

### Key abstractions

```rust
// models/download.rs
pub enum DownloadStatus { Downloading, Completed, Paused, Error, Queued }

pub struct DownloadItem {
    pub id: u64,
    pub file_name: String,
    pub status: DownloadStatus,
    pub size: u64,
    pub downloaded: u64,
    pub speed: u64,        // bytes/sec
    pub progress: f32,     // 0.0–1.0
    pub remaining: String,
    pub added_date: String,
}

pub trait DownloadProvider {
    fn downloads(&self) -> Vec<DownloadItem>;
    fn categories(&self) -> Vec<(String, usize)>;  // label, count
}
```

```rust
// gui/app.rs
pub struct UiSettings {
    pub rtl: bool,
}
```

## Component details

### Menu Bar (gui/menu.rs)
- egui `menu::bar()` with `menu::menu()` for each dropdown
- File: New Download, Open File, Exit
- Edit: Copy, Paste, Select All
- View: Categories, Details, Status Bar (checkable toggles)
- Tools: Scheduler, Preferences
- Help: About, Website, Documentation

### Toolbar (gui/toolbar.rs)
- Fixed 32px height, horizontal layout
- 7 buttons: each `Button::new("Label")` with icon text
- States: enabled/disabled via `UiSettings` or selection state
- Compact spacing (2px between buttons)

### Sidebar (gui/sidebar.rs)
- `SidePanel::resizable()`, default 220px
- Category items: `Frame` + `SelectableLabel` with counter badge
- Info box below categories — empty placeholder frame

### Download Table (gui/downloads.rs)
- `TableBuilder::new()` with striped rows
- Columns: File Name (250px), Status (100px), Size (90px), Downloaded (90px), Speed (80px), Remaining (80px), Progress (120px), Added Date (120px)
- Sortable: click column header to sort asc/desc
- Row selection via `selectable_label`
- `MockDownloadProvider` generates 100 items

### Bottom Panel (gui/bottom_panel.rs)
- `TopBottomPanel::bottom()` resizable, ~180px default
- Tab bar: Progress | Properties | Log
- **Progress tab**: `SegmentedProgress` widget
- **Properties tab**: `Grid` layout showing selected item metadata
- **Log tab**: `TextEdit` or scroll area with mock entries

### Status Bar (gui/statusbar.rs)
- `TopBottomPanel::bottom()` fixed ~22px
- Left label: "Ready" / "Downloading: N files"
- Right labels: `Hyperlink` for Website, Documentation, Support

### Segmented Progress (widgets/segmented_progress.rs)
- `struct SegmentedProgress { total: usize, downloaded: usize, cols: usize }`
- Custom `Widget` impl using `Ui::allocate_ui` + `paint_rect`
- Renders N×M grid of 4×4px rectangles
- Filled blocks = downloaded, outlined = pending
- Skips blocks outside visible rect for performance

### Theme (gui/theme.rs)
- Compact `Style`: reduced spacing, small font sizes (~12px), thin margins
- Color palette: light gray `#F0F0F0` backgrounds, `#0078D7` blue selection, `#E8E8E8` borders
- Windows classic visual style: sunken frames, 3D borders

## RTL support

`UiSettings.rtl` controls layout mirroring:
- Menu bar: reversed dropdown alignment
- Sidebar: right side instead of left
- Toolbar: right-to-left button order
- Table: right-aligned text, reversed column order
- Status bar: left/right swapped

Implemented via conditional `Layout::right_to_left()` and mirrored panel positions.

## Performance

- Table: `TableBuilder` with `Sense::click()` — handles 10k+ rows via virtualized rendering
- Segmented progress: only paints blocks within clip rect
- No per-frame allocations in hot paths
- State stored in `EframeApp` struct, mutated by UI events

## Error handling

- No backend integration yet — no real errors to handle
- Placeholder error states in UI (empty list view, error icons in status field)

## Mock data

`MockDownloadProvider` generates:
- 100 `DownloadItem`s with varied statuses, sizes (1MB–4GB), speeds
- Random progress values for downloading items
- Realistic file names, timestamps
- Category counts matching item distribution

## Non-goals (Phase 1)

- No connection to real download engine (`download.rs`)
- No real download start/pause/delete operations
- No real logging
- No configuration persistence
- No tray/minimize support

## Files created

| File | Lines (est.) | Purpose |
|---|---|---|
| `src/models/mod.rs` | 2 | module |
| `src/models/download.rs` | 80 | types, trait, mock provider |
| `src/gui/mod.rs` | 15 | run_gui() re-exports |
| `src/gui/app.rs` | 120 | eframe::App, layout tree |
| `src/gui/menu.rs` | 80 | menu bar |
| `src/gui/toolbar.rs` | 80 | toolbar |
| `src/gui/sidebar.rs` | 100 | sidebar |
| `src/gui/downloads.rs` | 150 | table view |
| `src/gui/bottom_panel.rs` | 150 | tabs + splitter |
| `src/gui/statusbar.rs` | 50 | status bar |
| `src/gui/theme.rs` | 60 | styling |
| `src/widgets/mod.rs` | 2 | module |
| `src/widgets/segmented_progress.rs` | 80 | block widget |

## Files modified

| File | Change |
|---|---|
| `Cargo.toml` | +eframe, +egui, +egui_extras |
| `src/main.rs` | +--gui flag, route to run_gui() |
