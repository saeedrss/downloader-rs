use eframe::egui;
use eframe::egui::{Color32, ScrollArea};
use egui_extras::{Column, TableBuilder};
use crate::models::download::{DownloadItem, DownloadStatus};

pub fn format_size(bytes: u64) -> String {
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
        TableBuilder::new(ui)
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
                body.rows(20.0, downloads.len(), |mut row| {
                    let row_idx = row.index();
                    let item = &downloads[row_idx];
                    let is_selected = selected.map(|s| s == item.id).unwrap_or(false);

                    row.set_selected(is_selected);

                    row.col(|ui: &mut egui::Ui| {
                        if ui.selectable_label(is_selected, &item.file_name).clicked() {
                            *selected = Some(item.id);
                        }
                    });
                    row.col(|ui: &mut egui::Ui| {
                        ui.colored_label(status_color(&item.status), item.status.label());
                    });
                    row.col(|ui: &mut egui::Ui| { ui.label(format_size(item.size)); });
                    row.col(|ui: &mut egui::Ui| { ui.label(format_size(item.downloaded)); });
                    row.col(|ui: &mut egui::Ui| {
                        if item.speed > 0 {
                            ui.label(format!("{}/s", format_size(item.speed)));
                        } else {
                            ui.label("-");
                        }
                    });
                    row.col(|ui: &mut egui::Ui| { ui.label(&item.remaining); });
                    row.col(|ui: &mut egui::Ui| {
                        let pb = egui::ProgressBar::new(item.progress)
                            .show_percentage()
                            .desired_width(100.0);
                        ui.add(pb);
                    });
                    row.col(|ui: &mut egui::Ui| { ui.label(&item.added_date); });
                });
            });
    });
}
