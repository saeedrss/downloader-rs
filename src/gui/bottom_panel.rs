use eframe::egui;
use eframe::egui::{Grid, ScrollArea};
use crate::models::download::DownloadItem;
use crate::gui::downloads::format_size;

#[derive(PartialEq)]
enum BottomTab {
    Progress,
    Properties,
    Log,
}

pub fn show(ui: &mut egui::Ui, selected: Option<&DownloadItem>) {
    let mut current_tab = BottomTab::Progress;

    ui.horizontal(|ui: &mut egui::Ui| {
        ui.selectable_value(&mut current_tab, BottomTab::Progress, "Progress");
        ui.selectable_value(&mut current_tab, BottomTab::Properties, "Properties");
        ui.selectable_value(&mut current_tab, BottomTab::Log, "Log");
    });
    ui.separator();

    match current_tab {
        BottomTab::Progress => {
            crate::widgets::segmented_progress::show(ui, 2400, 3000, 60);
        }
        BottomTab::Properties => {
            if let Some(item) = selected {
                Grid::new("props").striped(true).show(ui, |ui| {
                    ui.label("File Name:"); ui.label(&item.file_name); ui.end_row();
                    ui.label("Size:"); ui.label(&format_size(item.size)); ui.end_row();
                    ui.label("Downloaded:"); ui.label(&format_size(item.downloaded)); ui.end_row();
                    ui.label("Status:"); ui.label(item.status.label()); ui.end_row();
                    ui.label("Speed:");
                    if item.speed > 0 {
                        ui.label(format!("{}/s", format_size(item.speed)));
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
