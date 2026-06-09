use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    ui.horizontal(|ui: &mut egui::Ui| {
        ui.set_height(20.0);
        ui.label("Ready");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui: &mut egui::Ui| {
            ui.add(egui::Hyperlink::from_label_and_url("Support", "https://github.com/saeedrss/downloader-rs"));
            ui.separator();
            ui.add(egui::Hyperlink::from_label_and_url("Docs", "https://github.com/saeedrss/downloader-rs"));
            ui.separator();
            ui.add(egui::Hyperlink::from_label_and_url("Website", "https://github.com/saeedrss/downloader-rs"));
        });
    });
}
