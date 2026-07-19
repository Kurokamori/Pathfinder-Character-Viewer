#![windows_subsystem = "windows"]

use pathfinder_viewer::app::App;

fn main() -> iced::Result {
    iced::application("Pathfinder Viewer", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size(iced::Size::new(1360.0, 900.0))
        .antialiasing(true)
        .run_with(App::new)
}
