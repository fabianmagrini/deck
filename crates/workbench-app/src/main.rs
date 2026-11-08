mod actions;
mod app;
mod theme;
mod views;

use gpui::Application;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Application::new().run(app::setup);
}
