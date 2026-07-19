//! Settings screen: exclude source books from browsing and selection.

use crate::app::{App, Message};
use crate::ui::widgets::{self, caption};
use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();

    let header = row![
        widgets::heading(p, "Books & Sources"),
        Space::with_width(Length::Fill),
        widgets::primary_button(p, "Done", Message::OpenSettings(false)),
    ]
    .align_y(Alignment::Center);

    let note = widgets::card(
        p,
        column![
            text("Exclude source books to hide their content from spell, hex, feat, and shop lists.")
                .size(13)
                .color(p.text),
            caption(
                p,
                "Note: source data is sparse in this dataset, so many entries are unlabeled.",
            ),
        ]
        .spacing(6),
    );

    let unlabeled = iced::widget::checkbox("Hide unlabeled entries", app.settings.exclude_unlabeled)
        .on_toggle(|_| Message::ToggleUnlabeled)
        .size(20)
        .text_size(14)
        .style(crate::theme::check(p));

    let sources = app.game.sources();
    let mut list = column![].spacing(10);
    if sources.is_empty() {
        list = list.push(caption(p, "No labeled sources found in the data."));
    }
    for source in sources {
        let included = !app.settings.excluded_books.contains(source);
        let src = source.clone();
        list = list.push(
            iced::widget::checkbox(source.clone(), included)
                .on_toggle(move |_| Message::ToggleBook(src.clone()))
                .size(20)
                .text_size(14)
                .style(crate::theme::check(p)),
        );
    }

    let body = column![
        header,
        note,
        widgets::section(p, "General", unlabeled),
        widgets::section(p, "Source Books", list),
    ]
    .spacing(18);

    container(scrollable(container(body).padding(40).max_width(900)).height(Length::Fill))
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
