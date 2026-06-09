use eframe::egui;
use eframe::egui::{Frame, Margin, ScrollArea, SelectableLabel};
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
                *selected = label.to_string();
            }
        }
    });

    ui.separator();
    ui.add_space(8.0);

    Frame {
        inner_margin: Margin::symmetric(4.0, 4.0),
        fill: ui.style().visuals.window_fill,
        stroke: ui.style().visuals.widgets.noninteractive.bg_stroke,
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.set_min_size([ui.available_width(), 80.0].into());
        ui.vertical_centered(|ui| {
            ui.label("Advertisement Space");
        });
    });
}
