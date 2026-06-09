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
