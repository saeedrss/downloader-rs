# EGUI Orbit Downloader GUI — Implementation Plan

> **For agentic workers:** Use subagent-driven-development to implement this plan task-by-task.

**Goal:** Add an egui desktop GUI layer to the existing download manager, recreating the classic Orbit Downloader layout.

**Architecture:** New `gui/`, `models/`, `widgets/` modules with a `DownloadProvider` abstraction layer. No changes to existing download engine. `--gui` flag routes to eframe.

**Tech Stack:** Rust, eframe, egui, egui_extras

---

### Task 1: Update Cargo.toml and main.rs entry point

**Files:**
- Modify: `Cargo.toml` — add eframe, egui, egui_extras dependencies
- Modify: `src/main.rs` — add `--gui` flag, route to `gui::run_gui()`

- [ ] **Step 1: Add dependencies to Cargo.toml**

After line 15 (closing dependency), add:

```toml
eframe = { version = "0.29", features = ["default"] }
egui_extras = { version = "0.29", features = ["default"] }
```

- [ ] **Step 2: Add `--gui` flag to Args struct in main.rs**

After line 55 (`temp_dir`), add:

```rust
    #[arg(long, default_value_t = false)]
    gui: bool,
```

- [ ] **Step 3: Add conditional GUI launch in main()**

Before `let proxy_urls = parse_proxies(&args.proxies);` (around line 498), add:

```rust
    if args.gui {
        gui::run_gui();
        return Ok(());
    }
```

- [ ] **Step 4: Add `mod gui;` at top of main.rs**

After line 3 (`mod tui;`), add:

```rust
mod gui;
mod models;
mod widgets;
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build --features gui` (or just `cargo build` if features not used)
Expected: compiles (may fail because `gui::run_gui()` doesn't exist yet — that's fine, it's created in Task 2)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: add --gui flag and eframe dependencies"
```

---

### Task 2: Create models — DownloadItem, DownloadStatus, DownloadProvider trait

**Files:**
- Create: `src/models/mod.rs`
- Create: `src/models/download.rs`

- [ ] **Step 1: Create `src/models/mod.rs`**

```rust
pub mod download;
```

- [ ] **Step 2: Create `src/models/download.rs`**

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    Downloading,
    Completed,
    Paused,
    Error,
    Queued,
}

impl DownloadStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Downloading => "Downloading",
            Self::Completed => "Completed",
            Self::Paused => "Paused",
            Self::Error => "Error",
            Self::Queued => "Queued",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DownloadItem {
    pub id: u64,
    pub file_name: String,
    pub status: DownloadStatus,
    pub size: u64,
    pub downloaded: u64,
    pub speed: u64,
    pub progress: f32,
    pub remaining: String,
    pub added_date: String,
}

pub trait DownloadProvider {
    fn downloads(&self) -> Vec<DownloadItem>;
    fn categories(&self) -> Vec<(String, usize)>;
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: compiles (unused type warnings are expected)

- [ ] **Step 4: Commit**

```bash
git add src/models/
git commit -m "feat: add DownloadItem, DownloadStatus, DownloadProvider trait"
```

---

### Task 3: Create MockDownloadProvider

**Files:**
- Modify: `src/models/download.rs` — add mock provider

- [ ] **Step 1: Add MockDownloadProvider to `src/models/download.rs`**

Append after the `DownloadProvider` trait:

```rust
use rand::Rng;

pub struct MockDownloadProvider;

impl MockDownloadProvider {
    pub fn new() -> Self {
        Self
    }
}

impl DownloadProvider for MockDownloadProvider {
    fn downloads(&self) -> Vec<DownloadItem> {
        let names = [
            "ubuntu-24.04-desktop-amd64.iso",
            "python-3.12.0-amd64.exe",
            "vscode_1.95.3_amd64.deb",
            "node-v22.0.0-x64.msi",
            "docker-desktop-4.34.0-amd64.exe",
            "ffmpeg-7.1-full_build.zip",
            "libreoffice-24.8.3_Win_x86-64.msi",
            "goland-2024.3.1.exe",
            "postgresql-17.2-1-windows-x64.exe",
            "rust-1.83.0-x86_64-pc-windows-msvc.msi",
            "nginx-1.26.2.zip",
            "mysql-9.1.0-winx64.msi",
            "git-2.47.1-64-bit.exe",
            "teamviewer_15.58.4.exe",
            "steam_latest.exe",
            "discord-0.0.616.exe",
            "obs-studio-31.0.1-full.zip",
            "blender-4.3.2-windows-x64.msi",
            "gimp-2.10.38-setup.exe",
            "inkscape-1.4.1-x64.exe",
            "virtualbox-7.1.4-165100-Win.exe",
            "vagrant_2.4.3_windows_amd64.msi",
            "terraform_1.10.3_windows_amd64.zip",
            "kubernetes-node-v1.32.0.msi",
            "dotnet-sdk-9.0.102-win-x64.exe",
            "jdk-21_windows-x64_bin.exe",
            "android-studio-2024.3.1.12-windows.exe",
            "unityhub-3.10.0.exe",
            "matlab_R2024b_Win64.iso",
            "autocad-2025-win64.exe",
            "photoshop-2025-win64.exe",
            "office-2024-pro-win64.iso",
            "visual-studio-2022-community.exe",
            "sql-server-2022-developer.exe",
            "mongodb-8.0.4-windows-x64.msi",
            "redis-7.4.1-Windows-x64.msi",
            "elasticsearch-8.17.0-windows-x86_64.zip",
            "kibana-8.17.0-windows-x86_64.zip",
            "grafana-11.4.0.windows-amd64.msi",
            "prometheus-3.1.0.windows-amd64.zip",
            "wolfram-mathematica-14.1-win64.iso",
            "solidworks-2025-sp0-win64.iso",
            "catia-v6-2025-win64.iso",
            "ansys-2025-r1-win64.iso",
            "comsol-6.3-win64.iso",
            "stata-19-win64.exe",
            "spss-29-win64.exe",
            "sas-9.4-win64.iso",
            "eviews-14-win64.exe",
            "minitab-22-win64.exe",
        ];
        let statuses = [
            DownloadStatus::Downloading,
            DownloadStatus::Completed,
            DownloadStatus::Paused,
            DownloadStatus::Error,
            DownloadStatus::Queued,
        ];
        let mut rng = rand::thread_rng();
        names
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                let status = statuses[rng.gen_range(0..statuses.len())].clone();
                let size = rng.gen_range(50_000_000u64..8_000_000_000u64);
                let downloaded = match status {
                    DownloadStatus::Completed => size,
                    DownloadStatus::Downloading => rng.gen_range(0..size),
                    _ => 0,
                };
                let progress = if size > 0 {
                    downloaded as f32 / size as f32
                } else {
                    0.0
                };
                let speed = if status == DownloadStatus::Downloading {
                    rng.gen_range(500_000..50_000_000)
                } else {
                    0
                };
                let remaining = if speed > 0 && progress < 1.0 {
                    let secs = (size - downloaded) / speed;
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    "-".to_string()
                };
                DownloadItem {
                    id: i as u64,
                    file_name: name.to_string(),
                    status,
                    size,
                    downloaded,
                    speed,
                    progress,
                    remaining,
                    added_date: format!(
                        "2025-{:02}-{:02}",
                        rng.gen_range(1..12),
                        rng.gen_range(1..28)
                    ),
                }
            })
            .collect()
    }

    fn categories(&self) -> Vec<(String, usize)> {
        let downloads = self.downloads();
        let total = downloads.len();
        let downloading = downloads.iter().filter(|d| d.status == DownloadStatus::Downloading).count();
        let completed = downloads.iter().filter(|d| d.status == DownloadStatus::Completed).count();
        let paused = downloads.iter().filter(|d| d.status == DownloadStatus::Paused).count();
        let error = downloads.iter().filter(|d| d.status == DownloadStatus::Error).count();
        let inactive = downloads.iter().filter(|d| d.status == DownloadStatus::Queued).count();
        vec![
            ("All".into(), total),
            ("Downloading".into(), downloading),
            ("Completed".into(), completed),
            ("Inactive".into(), inactive),
            ("Software Updates".into(), 0),
            ("Uncategorized".into(), error + paused),
        ]
    }
}
```

- [ ] **Step 2: Add `rand` dependency to Cargo.toml**

Add after the eframe/egui lines:

```toml
rand = "0.8"
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: compiles with unused warnings

- [ ] **Step 4: Commit**

```bash
git add src/models/ Cargo.toml
git commit -m "feat: add MockDownloadProvider with realistic data"
```

---

### Task 4: Create theme module

**Files:**
- Create: `src/gui/mod.rs`
- Create: `src/gui/theme.rs`

- [ ] **Step 1: Create `src/gui/mod.rs`**

```rust
pub mod app;
pub mod menu;
pub mod toolbar;
pub mod sidebar;
pub mod downloads;
pub mod bottom_panel;
pub mod statusbar;
pub mod theme;

pub fn run_gui() {
    app::run_app();
}
```

- [ ] **Step 2: Create `src/gui/theme.rs`**

```rust
use egui::{Color32, FontData, FontDefinitions, FontFamily, Style, Visuals};

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Compact spacing
    style.spacing.item_spacing = egui::vec2(4.0, 2.0);
    style.spacing.button_padding = egui::vec2(4.0, 1.0);
    style.spacing.indent = 8.0;
    style.spacing.menu_margin = egui::Margin::symmetric(1.0, 1.0);
    
    // Small fonts
    let mut font_def = FontDefinitions::default();
    if let Some(font) = font_def.font_data.get_mut("Hack") {
        // use existing
    }
    
    ctx.set_style(style);
    
    // Classic desktop visuals
    ctx.set_visuals(Visuals {
        window_rounding: egui::Rounding::ZERO,
        window_shadow: egui::epaint::Shadow::NONE,
        window_fill: Color32::from_rgb(240, 240, 240),
        panel_fill: Color32::from_rgb(240, 240, 240),
        faint_bg_color: Color32::from_rgb(245, 245, 245),
        extreme_bg_color: Color32::from_rgb(255, 255, 255),
        code_bg_color: Color32::from_rgb(235, 235, 235),
        warn_fg_color: Color32::from_rgb(200, 120, 0),
        error_fg_color: Color32::from_rgb(200, 40, 40),
        hyperlink_color: Color32::from_rgb(0, 80, 200),
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgb(0, 120, 215),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 100, 200)),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(240, 240, 240),
                weak_bg_fill: Color32::from_rgb(235, 235, 235),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                rounding: egui::Rounding::ZERO,
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(50, 50, 50)),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(235, 235, 235),
                weak_bg_fill: Color32::from_rgb(230, 230, 230),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(180, 180, 180)),
                rounding: egui::Rounding::ZERO,
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(220, 230, 240),
                weak_bg_fill: Color32::from_rgb(210, 225, 240),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 120, 215)),
                rounding: egui::Rounding::ZERO,
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(0, 0, 0)),
                expansion: 0.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(200, 215, 230),
                weak_bg_fill: Color32::from_rgb(190, 210, 230),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 100, 200)),
                rounding: egui::Rounding::ZERO,
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(0, 0, 0)),
                expansion: 0.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(210, 210, 210),
                weak_bg_fill: Color32::from_rgb(200, 200, 200),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(160, 160, 160)),
                rounding: egui::Rounding::ZERO,
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
                expansion: 0.0,
            },
        },
        ..Default::default()
    });
}
```

- [ ] **Step 3: Commit**

```bash
git add src/gui/
git commit -m "feat: add gui module and theme (compact WinXP style)"
```

---

### Task 5: Create App struct, UiSettings, and run_app

**Files:**
- Create: `src/gui/app.rs`

- [ ] **Step 1: Create `src/gui/app.rs`**

```rust
use eframe::egui;
use crate::models::download::{DownloadItem, DownloadProvider, MockDownloadProvider};

pub struct UiSettings {
    pub rtl: bool,
}

pub struct EframeApp {
    pub provider: Box<dyn DownloadProvider>,
    pub downloads: Vec<DownloadItem>,
    pub settings: UiSettings,
    pub selected_category: String,
    pub selected_download: Option<u64>,
}

impl Default for EframeApp {
    fn default() -> Self {
        let provider = Box::new(MockDownloadProvider::new());
        let downloads = provider.downloads();
        Self {
            provider,
            downloads,
            settings: UiSettings { rtl: false },
            selected_category: "All".into(),
            selected_download: None,
        }
    }
}

impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::gui::theme::apply_theme(ctx);
        
        if self.settings.rtl {
            ctx.set_layout(egui::Layout::right_to_left(egui::Align::TOP));
        }
        
        crate::gui::menu::show(ctx, &mut self.settings);
        crate::gui::toolbar::show(ctx);
        
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| {
                crate::gui::sidebar::show(ui, &mut self.selected_category, &self.downloads);
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::gui::downloads::show(ui, &self.downloads, &mut self.selected_download);
        });
        
        egui::TopBottomPanel::bottom("statusbar")
            .min_height(22.0)
            .show(ctx, |ui| {
                crate::gui::statusbar::show(ui);
            });
    }
}

pub fn run_app() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Downloader"),
        ..Default::default()
    };
    eframe::run_native(
        "Downloader",
        options,
        Box::new(|_cc| Ok(Box::new(EframeApp::default()))),
    )
    .expect("Failed to launch GUI window");
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: compiles (GUI won't display panels yet — they return nothing — but app struct exists)

- [ ] **Step 3: Commit**

```bash
git add src/gui/app.rs
git commit -m "feat: add EframeApp, UiSettings, and run_app entry point"
```

---

### Task 6: Create Menu Bar

**Files:**
- Create: `src/gui/menu.rs`

- [ ] **Step 1: Create `src/gui/menu.rs`**

```rust
use egui::menu;
use crate::gui::app::UiSettings;

pub fn show(ctx: &egui::Context, settings: &mut UiSettings) {
    menu::bar(ctx, "menubar", |ui| {
        menu::menu(ui, "File", |ui| {
            if ui.button("New Download").clicked() { ui.close_menu(); }
            if ui.button("Open File").clicked() { ui.close_menu(); }
            ui.separator();
            if ui.button("Exit").clicked() { ui.close_menu(); }
        });
        menu::menu(ui, "Edit", |ui| {
            if ui.button("Copy").clicked() { ui.close_menu(); }
            if ui.button("Paste").clicked() { ui.close_menu(); }
            ui.separator();
            if ui.button("Select All").clicked() { ui.close_menu(); }
        });
        menu::menu(ui, "View", |ui| {
            if ui.checkbox(&mut true, "Categories").clicked() { ui.close_menu(); }
            if ui.checkbox(&mut true, "Details").clicked() { ui.close_menu(); }
            if ui.checkbox(&mut true, "Status Bar").clicked() { ui.close_menu(); }
            ui.separator();
            if ui.checkbox(&mut settings.rtl, "RTL Mode").clicked() { ui.close_menu(); }
        });
        menu::menu(ui, "Tools", |ui| {
            if ui.button("Scheduler").clicked() { ui.close_menu(); }
            if ui.button("Preferences").clicked() { ui.close_menu(); }
        });
        menu::menu(ui, "Help", |ui| {
            if ui.button("About").clicked() { ui.close_menu(); }
            if ui.button("Website").clicked() { ui.close_menu(); }
            if ui.button("Documentation").clicked() { ui.close_menu(); }
        });
    });
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/gui/menu.rs
git commit -m "feat: add menu bar (File/Edit/View/Tools/Help)"
```

---

### Task 7: Create Toolbar

**Files:**
- Create: `src/gui/toolbar.rs`

- [ ] **Step 1: Create `src/gui/toolbar.rs`**

```rust
use egui::{Button, Frame, Margin, Vec2};

pub fn show(ctx: &egui::Context) {
    egui::TopBottomPanel::top("toolbar")
        .min_height(32.0)
        .frame(Frame {
            inner_margin: Margin::symmetric(4.0, 2.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(28.0);
                toolbar_button(ui, "⏵ New", true);
                toolbar_button(ui, "▶ Start", true);
                toolbar_button(ui, "⏸ Pause", true);
                toolbar_button(ui, "✕ Delete", true);
                toolbar_button(ui, "⏰ Scheduler", true);
                toolbar_button(ui, "☰ Report", true);
                toolbar_button(ui, "⚙ Preferences", true);
            });
        });
}

fn toolbar_button(ui: &mut egui::Ui, label: &str, enabled: bool) {
    let mut btn = Button::new(label);
    if !enabled {
        btn = btn.fill(ui.style().visuals.widgets.inactive.bg_fill);
    }
    let resp = ui.add_enabled(enabled, btn);
    if resp.clicked() {
        // Future: dispatch actions
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/gui/toolbar.rs
git commit -m "feat: add toolbar (New/Start/Pause/Delete/Scheduler/Report/Preferences)"
```

---

### Task 8: Create Status Bar

**Files:**
- Create: `src/gui/statusbar.rs`

- [ ] **Step 1: Create `src/gui/statusbar.rs`**

```rust
use egui::{Frame, Hyperlink, Margin};

pub fn show(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.set_height(20.0);
        ui.label("Ready");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(Hyperlink::new("https://github.com/saeedrss/downloader-rs").text("Support"));
            ui.separator();
            ui.add(Hyperlink::new("https://github.com/saeedrss/downloader-rs").text("Docs"));
            ui.separator();
            ui.add(Hyperlink::new("https://github.com/saeedrss/downloader-rs").text("Website"));
        });
    });
}
```

- [ ] **Step 2: Commit**

```bash
git add src/gui/statusbar.rs
git commit -m "feat: add status bar with status text and hyperlinks"
```

---

### Task 9: Create Sidebar

**Files:**
- Create: `src/gui/sidebar.rs`

- [ ] **Step 1: Create `src/gui/sidebar.rs`**

```rust
use egui::{Frame, Margin, ScrollArea, SelectableLabel};
use crate::models::download::DownloadItem;

pub fn show(ui: &mut egui::Ui, selected: &mut String, downloads: &[DownloadItem]) {
    ui.heading("Categories");
    ui.separator();

    let categories = vec![
        ("All", downloads.len()),
        ("Downloading", downloads.iter().filter(|d| d.status.label() == "Downloading").count()),
        ("Completed", downloads.iter().filter(|d| d.status.label() == "Completed").count()),
        ("Inactive", downloads.iter().filter(|d| d.status.label() == "Queued").count()),
        ("Software Updates", 0usize),
        ("Uncategorized", downloads.iter().filter(|d| {
            matches!(d.status.label(), "Error" | "Paused")
        }).count()),
    ];

    ScrollArea::vertical().show(ui, |ui| {
        for (label, count) in &categories {
            let text = format!("{}  [{}]", label, count);
            let resp = ui.add(SelectableLabel::new(selected == *label, text));
            if resp.clicked() {
                selected = label.to_string();
            }
        }
    });

    ui.separator();
    ui.add_space(8.0);

    // Info box placeholder
    Frame {
        inner_margin: Margin::symmetric(4.0, 4.0),
        fill: ui.style().visuals.window_fill,
        stroke: ui.style().visuals.widgets.noninteractive.bg_stroke,
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.set_min_size([ui.available_width(), 80.0].into());
        ui.vertical_centered(|ui| {
            ui.label("Advertisement");
            ui.label("Space");
        });
    });
}
```

- [ ] **Step 2: Commit**

```bash
git add src/gui/sidebar.rs
git commit -m "feat: add sidebar with categories and info box"
```

---

### Task 10: Create Download Table

**Files:**
- Create: `src/gui/downloads.rs`

- [ ] **Step 1: Create `src/gui/downloads.rs`**

```rust
use egui::{Color32, Frame, Margin, ScrollArea, Sense};
use egui_extras::{Column, TableBuilder};
use crate::models::download::{DownloadItem, DownloadStatus};

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

fn status_color(status: &DownloadStatus) -> Color32 {
    match status {
        DownloadStatus::Downloading => Color32::from_rgb(0, 120, 0),
        DownloadStatus::Completed => Color32::from_rgb(0, 80, 200),
        DownloadStatus::Paused => Color32::from_rgb(200, 120, 0),
        DownloadStatus::Error => Color32::from_rgb(200, 40, 40),
        DownloadStatus::Queued => Color32::from_rgb(120, 120, 120),
    }
}

pub fn show(ui: &mut egui::Ui, downloads: &[DownloadItem], selected: &mut Option<u64>) {
    ScrollArea::horizontal().show(ui, |ui| {
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(250.0).resizable(true).clip(true))
            .column(Column::initial(100.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(90.0).resizable(true))
            .column(Column::initial(80.0).resizable(true))
            .column(Column::initial(80.0).resizable(true))
            .column(Column::initial(140.0).resizable(true))
            .column(Column::initial(100.0).resizable(true))
            .header(20.0, |mut header| {
                header.col(|ui| { ui.strong("File Name"); });
                header.col(|ui| { ui.strong("Status"); });
                header.col(|ui| { ui.strong("Size"); });
                header.col(|ui| { ui.strong("Downloaded"); });
                header.col(|ui| { ui.strong("Speed"); });
                header.col(|ui| { ui.strong("Remaining"); });
                header.col(|ui| { ui.strong("Progress"); });
                header.col(|ui| { ui.strong("Added Date"); });
            })
            .body(|body| {
                body.rows(20.0, downloads.len(), |row_idx, mut row| {
                    let item = &downloads[row_idx];
                    let is_selected = selected.map(|s| s == item.id).unwrap_or(false);
                    
                    row.set_selected(is_selected);
                    
                    row.col(|ui| {
                        if ui.selectable_label(is_selected, &item.file_name).clicked() {
                            *selected = Some(item.id);
                        }
                    });
                    row.col(|ui| {
                        ui.colored_label(status_color(&item.status), item.status.label());
                    });
                    row.col(|ui| { ui.label(format_size(item.size)); });
                    row.col(|ui| { ui.label(format_size(item.downloaded)); });
                    row.col(|ui| {
                        if item.speed > 0 {
                            ui.label(format!("{}/s", format_size(item.speed)));
                        } else {
                            ui.label("-");
                        }
                    });
                    row.col(|ui| { ui.label(&item.remaining); });
                    row.col(|ui| {
                        let pb = egui::ProgressBar::new(item.progress)
                            .show_percentage()
                            .desired_width(100.0);
                        ui.add(pb);
                    });
                    row.col(|ui| { ui.label(&item.added_date); });
                });
            });
    });
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/gui/downloads.rs
git commit -m "feat: add download table with sortable/scrollable columns"
```

---

### Task 11: Create Bottom Panel (tabs + splitter)

**Files:**
- Create: `src/gui/bottom_panel.rs`

- [ ] **Step 1: Create `src/gui/bottom_panel.rs`**

```rust
use egui::{Frame, Grid, ScrollArea};
use crate::models::download::DownloadItem;

enum BottomTab {
    Progress,
    Properties,
    Log,
}

pub fn show(ui: &mut egui::Ui, selected: Option<&DownloadItem>) {
    let mut current_tab = BottomTab::Progress;
    
    ui.horizontal(|ui| {
        ui.selectable_value(&mut current_tab, BottomTab::Progress, "Progress");
        ui.selectable_value(&mut current_tab, BottomTab::Properties, "Properties");
        ui.selectable_value(&mut current_tab, BottomTab::Log, "Log");
    });
    ui.separator();

    match current_tab {
        BottomTab::Progress => {
            // Placeholder for segmented progress widget
            crate::widgets::segmented_progress::show(ui, 2400, 3000, 60);
        }
        BottomTab::Properties => {
            if let Some(item) = selected {
                Grid::new("props").striped(true).show(ui, |ui| {
                    ui.label("File Name:"); ui.label(&item.file_name); ui.end_row();
                    ui.label("Size:"); ui.label(crate::gui::downloads::format_size(item.size)); ui.end_row();
                    ui.label("Downloaded:"); ui.label(crate::gui::downloads::format_size(item.downloaded)); ui.end_row();
                    ui.label("Status:"); ui.label(item.status.label()); ui.end_row();
                    ui.label("Speed:"); 
                    if item.speed > 0 {
                        ui.label(format!("{}/s", crate::gui::downloads::format_size(item.speed)));
                    } else {
                        ui.label("-");
                    }
                    ui.end_row();
                    ui.label("Added:"); ui.label(&item.added_date); ui.end_row();
                });
            } else {
                ui.label("Select a download to view properties");
            }
        }
        BottomTab::Log => {
            let mut log_text = String::new();
            ScrollArea::vertical().show(ui, |ui| {
                ui.text_edit_multiline(&mut log_text);
            });
        }
    }
}
```

- [ ] **Step 2: Create module file for widgets**

Create `src/widgets/mod.rs`:

```rust
pub mod segmented_progress;
```

- [ ] **Step 3: Create `src/widgets/segmented_progress.rs`**

```rust
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, downloaded: usize, total: usize, cols: usize) {
    if total == 0 { return; }
    
    let rows = (total + cols - 1) / cols;
    let block_size = 6.0;
    let gap = 1.0;
    let total_w = cols as f32 * (block_size + gap);
    let total_h = rows as f32 * (block_size + gap);
    
    let (resp, painter) = ui.allocate_painter(
        Vec2::new(total_w, total_h.min(150.0)),
        egui::Sense::hover(),
    );
    
    let clip_rect = resp.rect;
    let origin = clip_rect.left_top();
    
    let filled_color = Color32::from_rgb(0, 180, 80);
    let empty_color = Color32::from_rgb(220, 220, 220);
    
    let visible_rows = (clip_rect.height() / (block_size + gap)) as usize + 1;
    let start_row = 0usize;
    let end_row = (start_row + visible_rows).min(rows);
    
    for row in start_row..end_row {
        for col in 0..cols {
            let idx = row * cols + col;
            if idx >= total { break; }
            
            let x = origin.x + col as f32 * (block_size + gap);
            let y = origin.y + row as f32 * (block_size + gap);
            let rect = Rect::from_min_size(
                Pos2::new(x, y),
                Vec2::splat(block_size),
            );
            
            let color = if idx < downloaded { filled_color } else { empty_color };
            painter.rect_filled(rect, Rounding::ZERO, color);
        }
    }
}
```

- [ ] **Step 4: Update `app.rs` to show bottom panel**

In `app.rs`, add the bottom panel call after the CentralPanel. The CentralPanel currently shows only the downloads table. Change it to use a vertical split:

Replace the CentralPanel section in `app.rs` with:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(180.0)
        .min_height(60.0)
        .show_inside(ui, |ui| {
            let sel = self.selected_download.and_then(|id| {
                self.downloads.iter().find(|d| d.id == id)
            });
            crate::gui::bottom_panel::show(ui, sel);
        });
    
    crate::gui::downloads::show(ui, &self.downloads, &mut self.selected_download);
});
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build`
Expected: compiles cleanly

- [ ] **Step 6: Commit**

```bash
git add src/gui/bottom_panel.rs src/widgets/
git commit -m "feat: add bottom panel with tabs and segmented progress widget"
```

---

### Task 12: Add public format_size to downloads mod

- [ ] **Step 1: Make `format_size` public in `downloads.rs`**

The `bottom_panel.rs` uses `crate::gui::downloads::format_size`. Make the function `pub` (remove `pub` if it's currently private):

Change `fn format_size(` to `pub fn format_size(` in `src/gui/downloads.rs`.

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/gui/downloads.rs
git commit -m "chore: make format_size public for cross-module use"
```

---

### Task 13: Final integration test

- [ ] **Step 1: Full build check**

Run: `cargo build`
Expected: clean compile, no warnings

- [ ] **Step 2: Smoke test GUI launch**

Run: `cargo run -- --gui`
Expected: eframe window opens with Orbit-style layout (menu, toolbar, sidebar, table, bottom panel, status bar)

- [ ] **Step 3: Verify TUI still works**

Run: `cargo run -- --url "http://example.com/f" --proxies "socks5://127.0.0.1:9050" --size 5`
Expected: original TUI opens (no GUI)

- [ ] **Step 4: Commit if needed**

```bash
git add -A
git commit -m "feat: add egui desktop GUI layer (Orbit Downloader style)"
```
