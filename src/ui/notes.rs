//! Notes tab: a large multi-line editor for free-form character notes.

use crate::app::{App, Message};
use crate::ui::widgets;
use iced::widget::{column, text_editor};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let editor = text_editor(&app.notes_content)
        .placeholder("Write anything about your character...")
        .padding(14)
        .height(Length::Fixed(460.0))
        .on_action(Message::NotesAction);

    column![widgets::heading(p, "Notes"), widgets::card(p, editor)]
        .spacing(16)
        .into()
}
