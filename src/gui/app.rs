use eframe::egui;
use crate::models::download::{DownloadItem, DownloadProvider, MockDownloadProvider};
use crate::gui::theme;

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
        theme::apply_theme(ctx);
        
        if self.settings.rtl {
            let style = ctx.style();
            ctx.set_style(style.clone());
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
