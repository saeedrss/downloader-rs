use eframe::egui::{self, Button, Frame, Margin};

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
                toolbar_button(ui, "New", true);
                toolbar_button(ui, "Start", true);
                toolbar_button(ui, "Pause", true);
                toolbar_button(ui, "Delete", false);
                toolbar_button(ui, "Scheduler", true);
                toolbar_button(ui, "Report Bug", true);
                toolbar_button(ui, "Preferences", true);
            });
        });
}

fn toolbar_button(ui: &mut egui::Ui, label: &str, enabled: bool) {
    if ui.add_enabled(enabled, Button::new(label)).clicked() {
        // Future: dispatch actions
    }
}
