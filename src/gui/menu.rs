use eframe::egui;
use crate::gui::app::UiSettings;

pub fn show(ctx: &egui::Context, settings: &mut UiSettings) {
    egui::TopBottomPanel::top("menubar").show(ctx, |ui: &mut egui::Ui| {
        egui::menu::bar(ui, |ui: &mut egui::Ui| {
            ui.menu_button("File", |ui: &mut egui::Ui| {
                if ui.button("New Download").clicked() { ui.close_menu(); }
                if ui.button("Open File").clicked() { ui.close_menu(); }
                ui.separator();
                if ui.button("Exit").clicked() { ui.close_menu(); }
            });
            ui.menu_button("Edit", |ui: &mut egui::Ui| {
                if ui.button("Copy").clicked() { ui.close_menu(); }
                if ui.button("Paste").clicked() { ui.close_menu(); }
                ui.separator();
                if ui.button("Select All").clicked() { ui.close_menu(); }
            });
            ui.menu_button("View", |ui: &mut egui::Ui| {
                if ui.checkbox(&mut true, "Categories").clicked() { ui.close_menu(); }
                if ui.checkbox(&mut true, "Details").clicked() { ui.close_menu(); }
                if ui.checkbox(&mut true, "Status Bar").clicked() { ui.close_menu(); }
                ui.separator();
                if ui.checkbox(&mut settings.rtl, "RTL Mode").clicked() { ui.close_menu(); }
            });
            ui.menu_button("Tools", |ui: &mut egui::Ui| {
                if ui.button("Scheduler").clicked() { ui.close_menu(); }
                if ui.button("Preferences").clicked() { ui.close_menu(); }
            });
            ui.menu_button("Help", |ui: &mut egui::Ui| {
                if ui.button("About").clicked() { ui.close_menu(); }
                if ui.button("Website").clicked() { ui.close_menu(); }
                if ui.button("Documentation").clicked() { ui.close_menu(); }
            });
        });
    });
}
