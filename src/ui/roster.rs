//! Roster screen: create, import, open, and delete characters.

use crate::app::{App, Message};
use crate::rules::progression;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();

    let title = column![
        text("Pathfinder Viewer").size(32).color(p.text),
        text("A character sheet for Pathfinder 1st Edition")
            .size(14)
            .color(p.text_dim),
    ]
    .spacing(4);

    let mut new_row = row![caption(p, "Create:")].spacing(10).align_y(Alignment::Center);
    for tag in progression::supported_tags() {
        let def = progression::class_def(tag);
        new_row = new_row.push(
            button(text(format!("New {}", def.name)).size(14))
                .padding([9, 18])
                .style(crate::theme::accent_button(Palette::new(crate::theme::accent_for(tag))))
                .on_press(Message::NewCharacter(tag.to_string())),
        );
    }
    new_row = new_row.push(Space::with_width(Length::Fill));
    new_row = new_row.push(widgets::ghost_button(p, "Import…", Message::ImportCharacter));

    let mut body = column![title, widgets::divider(p), new_row].spacing(20);

    if let Some(err) = &app.load_error {
        body = body.push(widgets::card(
            p,
            column![
                text("Game data not loaded").size(16).color(p.bad),
                text(err.clone()).size(13).color(p.text_dim),
            ]
            .spacing(6),
        ));
    }

    body = body.push(roster_list(app, p));

    container(scrollable(container(body).padding(40).max_width(1000)).height(Length::Fill))
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn roster_list<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    if app.roster.is_empty() {
        return widgets::card(
            p,
            text("No saved characters yet. Create one above to begin.")
                .size(14)
                .color(p.text_dim),
        );
    }

    let mut list = column![].spacing(12);
    for summary in &app.roster {
        let card = container(
            row![
                column![
                    text(if summary.name.is_empty() {
                        "Unnamed".to_string()
                    } else {
                        summary.name.clone()
                    })
                    .size(18)
                    .color(p.text),
                    text(summary.class_line.clone()).size(13).color(p.text_dim),
                ]
                .spacing(3)
                .width(Length::Fill),
                widgets::primary_button(p, "Open", Message::OpenCharacter(summary.id)),
                button(text("Delete").size(13))
                    .padding([8, 14])
                    .style(crate::theme::danger_button(p))
                    .on_press(Message::DeleteCharacter(summary.id)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding(18)
        .width(Length::Fill)
        .style(crate::theme::card(p));
        list = list.push(card);
    }

    column![widgets::heading(p, "Saved Characters"), list]
        .spacing(14)
        .into()
}
