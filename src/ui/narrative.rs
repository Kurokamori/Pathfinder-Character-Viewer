//! Narrative tab: free-form prose describing who the character is — their look,
//! temperament, history, and the people they stand with and against.

use crate::app::{App, EditorTarget, Message};
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{column, row};
use iced::Element;

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();

    column![
        widgets::heading(p, "Narrative"),
        row![
            field(app, p, "Appearance", "How they look, dress, and carry themselves.", EditorTarget::Appearance),
            field(app, p, "Personality", "Temperament, quirks, values, and mannerisms.", EditorTarget::Personality),
        ]
        .spacing(16),
        row![
            field(app, p, "Origin", "Homeland, upbringing, and where they come from.", EditorTarget::Origins),
            field(app, p, "Affiliation", "Guilds, factions, or organizations they belong to.", EditorTarget::Affiliation),
        ]
        .spacing(16),
        row![
            field(app, p, "Friends & Allies", "Companions, contacts, and those who aid them.", EditorTarget::Friends),
            field(app, p, "Foes & Rivals", "Enemies, rivals, and those who stand against them.", EditorTarget::Foes),
        ]
        .spacing(16),
        field(app, p, "Backstory", "The full history that brought them to this point.", EditorTarget::Backstory),
    ]
    .spacing(16)
    .into()
}

/// A titled card wrapping a caption and a content-growing multiline editor for
/// one narrative field.
fn field<'a>(
    app: &'a App,
    p: Palette,
    title: &'a str,
    hint: &'a str,
    target: EditorTarget,
) -> Element<'a, Message> {
    let body = column![
        caption(p, hint),
        widgets::growing_editor(p, app.editors.get(&target), "", target),
    ]
    .spacing(6);

    widgets::section(p, title, body)
}
