fn main() -> iced::Result {
    iced::application("Pathfinder Viewer", update, view).run()
}

#[derive(Debug, Default)]
struct App {
    ticks: u32,
}

#[derive(Debug, Clone)]
enum Message {
    Ping,
}

fn update(state: &mut App, message: Message) {
    match message {
        Message::Ping => state.ticks += 1,
    }
}

fn view(state: &App) -> iced::Element<'_, Message> {
    use iced::widget::{button, column, text};
    column![
        text(format!("Pathfinder Viewer — ticks: {}", state.ticks)),
        button("Ping").on_press(Message::Ping),
    ]
    .into()
}
