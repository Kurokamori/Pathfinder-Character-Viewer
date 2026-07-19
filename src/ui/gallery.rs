//! Gallery tab: upload and display character art.

use crate::app::{App, Message};
use crate::theme::Palette;
use crate::ui::images::ImageCache;
use crate::ui::widgets;
use iced::widget::{button, column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

const COLUMNS: usize = 3;
const TILE: f32 = 300.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();

    let header = row![
        widgets::heading(p, "Gallery"),
        Space::with_width(Length::Fill),
        widgets::primary_button(p, "+ Add Art", Message::GalleryAdd),
    ]
    .align_y(Alignment::Center);

    let body: Element<Message> = if app.character.gallery.is_empty() {
        widgets::placeholder(p, "No art yet. Click \"Add Art\" to upload images of your character.")
    } else {
        let mut grid = column![].spacing(16);
        let mut current = row![].spacing(16);
        for (index, path) in app.character.gallery.iter().enumerate() {
            current = current.push(tile(p, index, path, &app.image_cache));
            if index % COLUMNS == COLUMNS - 1 {
                grid = grid.push(current);
                current = row![].spacing(16);
            }
        }
        grid = grid.push(current);
        scrollable(grid).height(Length::Fill).into()
    };

    column![header, body]
        .spacing(16)
        .height(Length::Fill)
        .into()
}

fn tile<'a>(p: Palette, index: usize, path: &str, images: &ImageCache) -> Element<'a, Message> {
    let inner: Element<Message> = match images.handle(path) {
        Some(handle) => image(handle)
            .width(Length::Fixed(TILE))
            .height(Length::Fixed(TILE))
            .into(),
        None => container(text("Unavailable").size(12).color(p.text_dim))
            .width(Length::Fixed(TILE))
            .height(Length::Fixed(TILE))
            .center_x(Length::Fixed(TILE))
            .center_y(Length::Fixed(TILE))
            .into(),
    };
    let art = container(inner).style(crate::theme::stat_box(p));

    let remove = button(text("Remove").size(12))
        .padding([5, 12])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::GalleryRemove(index));

    container(column![art, remove].spacing(8).align_x(Alignment::Center))
        .padding(8)
        .style(crate::theme::card(p))
        .into()
}
