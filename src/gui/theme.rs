use eframe::egui::{self, Color32, Style, Visuals};

pub fn apply_theme(ctx: &egui::Context) {
    let mut style: Style = (*ctx.style()).clone();
    
    style.spacing.item_spacing = egui::vec2(4.0, 2.0);
    style.spacing.button_padding = egui::vec2(4.0, 1.0);
    style.spacing.indent = 8.0;
    
    ctx.set_style(style);
    
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
