//! The persistent left panel: identity, vitals, abilities, and defenses.

use crate::app::{App, Message};
use crate::model::character::{Character, ABILITIES};
use crate::rules::derived::DerivedStats;
use crate::rules::display::signed;
use crate::theme::{self, Palette};
use crate::ui::images::ImageCache;
use crate::ui::widgets;
use iced::widget::{button, column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};
use crate::persistence;

const WIDTH: f32 = 372.0;
const PORTRAIT: f32 = 112.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let ch = &app.character;
    let d = app.derived();

    let content = column![
        identity(p, ch, &app.image_cache),
        vitals(p, ch, &d),
        abilities(p, ch),
        defenses(p, &d),
        saves_init(p, &d),
        movement(p, ch, &d),
    ]
    .spacing(12)
    .padding(16);

    container(scrollable(content).height(Length::Fill))
        .width(Length::Fixed(WIDTH))
        .height(Length::Fill)
        .style(theme::sidebar(p))
        .into()
}

fn compact_section<'a>(
    p: Palette,
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![text(title).size(12).color(p.text_dim), content.into()].spacing(8),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::card(p))
    .into()
}

fn compact_tile<'a>(p: Palette, label: &'a str, value: String) -> Element<'a, Message> {
    container(
        column![
            text(label).size(10).color(p.text_dim),
            text(value).size(18).color(p.text),
        ]
        .spacing(1)
        .align_x(Alignment::Center),
    )
    .padding([8, 8])
    .width(Length::Fill)
    .center_x(Length::Fill)
    .style(theme::stat_box(p))
    .into()
}

fn identity<'a>(p: Palette, ch: &'a Character, images: &ImageCache) -> Element<'a, Message> {
    let handle = ch.portrait.as_deref().and_then(|path| images.handle(path));
    let portrait: Element<Message> = match handle {
        Some(handle) => image(handle)
            .width(Length::Fixed(PORTRAIT))
            .height(Length::Fixed(PORTRAIT))
            .into(),
        None => container(text(initials(&ch.name)).size(30).color(p.accent))
            .width(Length::Fixed(PORTRAIT))
            .height(Length::Fixed(PORTRAIT))
            .center_x(Length::Fixed(PORTRAIT))
            .center_y(Length::Fixed(PORTRAIT))
            .style(theme::stat_box(p))
            .into(),
    };

    let class_line = ch
        .classes
        .iter()
        .map(|c| format!("{} {}", persistence::title_case(&c.tag), c.level))
        .collect::<Vec<_>>()
        .join(" / ");

    let name = if ch.name.is_empty() {
        "Unnamed".to_string()
    } else {
        ch.name.clone()
    };

    let race = if ch.race.is_empty() { "—" } else { &ch.race };
    let race_line = format!("{} {} · {}", ch.size.label(), race, class_line);

    row![
        column![
            portrait,
            button(text("Portrait").size(11))
                .padding([3, 8])
                .style(theme::ghost_button(p))
                .on_press(Message::ChangePortrait)
        ]
        .spacing(6)
        .align_x(Alignment::Center),
        column![
            text(name).size(20).color(p.text),
            text(race_line).size(12).color(p.text_dim),
            Space::with_height(2),
            widgets::pill(p, format!("Level {}", ch.total_level())),
        ]
        .spacing(4)
        .width(Length::Fill),
    ]
    .spacing(14)
    .into()
}

fn vitals<'a>(p: Palette, ch: &'a Character, d: &DerivedStats) -> Element<'a, Message> {
    let hp_line = row![
        column![
            text("Hit Points").size(11).color(p.text_dim),
            text(format!("{} / {}", ch.hp_current, d.max_hp))
                .size(21)
                .color(p.text),
        ]
        .spacing(1),
        Space::with_width(Length::Fill),
        row![
            dmg_button(p, "-5", Message::HpDelta(-5)),
            dmg_button(p, "-1", Message::HpDelta(-1)),
            dmg_button(p, "+1", Message::HpDelta(1)),
            dmg_button(p, "+5", Message::HpDelta(5)),
        ]
        .spacing(4),
    ]
    .align_y(Alignment::Center);

    let extras = row![
        widgets::number_field(p, "Temp HP", ch.hp_temp, 64.0, Message::SetHpTemp),
        widgets::number_field(p, "Nonlethal", ch.nonlethal, 64.0, Message::SetNonlethal),
        widgets::number_field(p, "Rolled HP", ch.hp_rolled, 64.0, Message::SetRolledHp),
    ]
    .spacing(8);

    container(column![hp_line, extras].spacing(10))
        .padding(12)
        .width(Length::Fill)
        .style(theme::card(p))
        .into()
}

fn dmg_button<'a>(p: Palette, label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([4, 8])
        .style(theme::subtle_button(p))
        .on_press(msg)
        .into()
}

fn abilities<'a>(p: Palette, ch: &'a Character) -> Element<'a, Message> {
    let mut grid = column![].spacing(8);
    let mut current = row![].spacing(8);
    for (i, key) in ABILITIES.iter().enumerate() {
        let score = ch.ability(key);
        let cell = container(
            column![
                text(key.to_uppercase()).size(10).color(p.text_dim),
                text(score.total().to_string()).size(17).color(p.text),
                widgets::mod_badge(p, score.modifier()),
            ]
            .spacing(0)
            .align_x(Alignment::Center),
        )
        .padding([8, 6])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(theme::stat_box(p));
        current = current.push(cell);
        if i % 3 == 2 {
            grid = grid.push(current);
            current = row![].spacing(8);
        }
    }
    compact_section(p, "Abilities", grid)
}

fn defenses<'a>(p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let tiles = row![
        compact_tile(p, "AC", d.ac.to_string()),
        compact_tile(p, "Touch", d.touch.to_string()),
        compact_tile(p, "Flat", d.flat_footed.to_string()),
    ]
    .spacing(8);
    compact_section(p, "Armor Class", tiles)
}

fn saves_init<'a>(p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let tiles = row![
        compact_tile(p, "Fort", signed(d.fort)),
        compact_tile(p, "Ref", signed(d.reflex)),
        compact_tile(p, "Will", signed(d.will)),
        compact_tile(p, "Init", signed(d.initiative)),
    ]
    .spacing(8);
    compact_section(p, "Saves & Initiative", tiles)
}

fn movement<'a>(p: Palette, ch: &'a Character, d: &DerivedStats) -> Element<'a, Message> {
    let tiles = row![
        compact_tile(p, "Speed", format!("{} ft", ch.base_speed)),
        compact_tile(p, "BAB", signed(d.bab)),
        compact_tile(p, "CMD", d.cmd.to_string()),
    ]
    .spacing(8);
    compact_section(p, "Movement & Maneuvers", tiles)
}

fn initials(name: &str) -> String {
    let letters: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if letters.is_empty() {
        "?".to_string()
    } else {
        letters
    }
}
