//! General tab: identity, class/level management, and ability scores.

use crate::app::{AbilityPart, App, IdentityField, Message};
use crate::model::character::{ability_name, Character, Size, ABILITIES};
use crate::rules::progression;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, pick_list, row, text, Space};
use iced::{Alignment, Element, Length};

const COMMON_CONDITIONS: [&str; 12] = [
    "Blinded",
    "Confused",
    "Dazed",
    "Entangled",
    "Exhausted",
    "Fatigued",
    "Frightened",
    "Grappled",
    "Nauseated",
    "Prone",
    "Shaken",
    "Sickened",
];

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    column![
        identity_section(app, p),
        classes_section(app, p),
        abilities_section(app, p),
        conditions_section(app, p),
    ]
    .spacing(18)
    .into()
}

fn identity_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;

    let race_names: Vec<String> = {
        let mut names: Vec<String> = app
            .game
            .compendium
            .races
            .iter()
            .filter(|r| app.settings.allows(&r.source))
            .map(|r| r.name.clone())
            .collect();
        names.sort();
        names
    };
    let selected_race = if ch.race.is_empty() {
        None
    } else {
        Some(ch.race.clone())
    };

    let race_pick = column![
        caption(p, "Race"),
        pick_list(race_names, selected_race, Message::SetRace)
            .placeholder("Choose race")
            .padding([7, 10])
            .text_size(14)
            .width(Length::Fill)
            .style(crate::theme::dropdown(p)),
    ]
    .spacing(4)
    .width(Length::Fill);

    let size_pick = column![
        caption(p, "Size"),
        pick_list(Size::ALL.to_vec(), Some(ch.size), Message::SetSize)
            .padding([7, 10])
            .text_size(14)
            .width(Length::Fill)
            .style(crate::theme::dropdown(p)),
    ]
    .spacing(4)
    .width(Length::Fill);

    let row1 = row![
        widgets::labeled_input(p, "Character Name", &ch.name, |v| Message::SetIdentity(
            IdentityField::Name,
            v
        )),
        widgets::labeled_input(p, "Player", &ch.player, |v| Message::SetIdentity(
            IdentityField::Player,
            v
        )),
    ]
    .spacing(14);

    let row2 = row![race_pick, size_pick].spacing(14);

    let row3 = row![
        widgets::labeled_input(p, "Alignment", &ch.alignment, |v| Message::SetIdentity(
            IdentityField::Alignment,
            v
        )),
        widgets::labeled_input(p, "Deity", &ch.deity, |v| Message::SetIdentity(
            IdentityField::Deity,
            v
        )),
        widgets::labeled_input(p, "Gender", &ch.gender, |v| Message::SetIdentity(
            IdentityField::Gender,
            v
        )),
    ]
    .spacing(14);

    let row4 = row![
        widgets::labeled_input(p, "Age", &ch.age, |v| Message::SetIdentity(
            IdentityField::Age,
            v
        )),
        widgets::labeled_input(p, "Height", &ch.height, |v| Message::SetIdentity(
            IdentityField::Height,
            v
        )),
        widgets::labeled_input(p, "Weight", &ch.weight_desc, |v| Message::SetIdentity(
            IdentityField::Weight,
            v
        )),
    ]
    .spacing(14);

    widgets::section(
        p,
        "Identity",
        column![row1, row2, row3, row4].spacing(12),
    )
}

fn classes_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let mut list = column![].spacing(10);
    for (index, class) in ch.classes.iter().enumerate() {
        let def = progression::class_def(&class.tag);
        let stepper = widgets::stepper(
            p,
            "Level",
            class.level.to_string(),
            Message::LevelDelta(index, -1),
            Message::LevelDelta(index, 1),
        );
        let mut controls = row![
            column![
                text(def.name).size(16).color(p.text),
                text(format!("d{} · {} skill ranks/level", def.hit_die, def.skills_per_level))
                    .size(12)
                    .color(p.text_dim),
            ]
            .spacing(2)
            .width(Length::Fill),
            stepper,
        ]
        .spacing(14)
        .align_y(Alignment::Center);
        if ch.classes.len() > 1 {
            controls = controls.push(
                button(text("Remove").size(12))
                    .padding([6, 12])
                    .style(crate::theme::danger_button(p))
                    .on_press(Message::RemoveClass(index)),
            );
        }
        list = list.push(widgets::card(p, controls));
    }

    let mut add_row = row![caption(p, "Add class:")].spacing(8).align_y(Alignment::Center);
    for tag in progression::supported_tags() {
        let def = progression::class_def(tag);
        add_row = add_row.push(
            button(text(def.name).size(13))
                .padding([6, 12])
                .style(crate::theme::subtle_button(p))
                .on_press(Message::AddClass(tag.to_string())),
        );
    }

    widgets::section(p, "Classes", column![list, add_row].spacing(14))
}

fn abilities_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let header = row![
        container_label(p, "Ability", 120.0),
        container_label(p, "Base", 70.0),
        container_label(p, "Race", 70.0),
        container_label(p, "Enhance", 70.0),
        container_label(p, "Temp", 70.0),
        container_label(p, "Total", 70.0),
        container_label(p, "Mod", 60.0),
    ]
    .spacing(10);

    let mut rows = column![header].spacing(8);
    for key in ABILITIES {
        rows = rows.push(ability_row(p, ch, key));
    }

    widgets::section(p, "Ability Scores", rows)
}

fn container_label<'a>(p: Palette, label: &'a str, width: f32) -> Element<'a, Message> {
    iced::widget::container(text(label).size(11).color(p.text_dim))
        .width(Length::Fixed(width))
        .into()
}

fn ability_row<'a>(p: Palette, ch: &'a Character, key: &'static str) -> Element<'a, Message> {
    let score = ch.ability(key);
    let field = |value: i32, part: AbilityPart| {
        iced::widget::text_input("", &value.to_string())
            .on_input(move |v| Message::SetAbility(key.to_string(), part, v))
            .padding([6, 8])
            .size(14)
            .width(Length::Fixed(70.0))
            .style(crate::theme::input(p))
    };

    row![
        iced::widget::container(text(ability_name(key)).size(14).color(p.text))
            .width(Length::Fixed(120.0)),
        field(score.base, AbilityPart::Base),
        field(score.racial, AbilityPart::Racial),
        field(score.enhancement, AbilityPart::Enhancement),
        field(score.temp, AbilityPart::Temp),
        iced::widget::container(text(score.total().to_string()).size(16).color(p.text))
            .width(Length::Fixed(70.0)),
        iced::widget::container(widgets::mod_badge(p, score.modifier()))
            .width(Length::Fixed(60.0)),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn conditions_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let mut wrap = row![].spacing(8);
    let mut rows = column![].spacing(8);
    for (i, name) in COMMON_CONDITIONS.iter().enumerate() {
        let active = ch.conditions.iter().any(|c| c == name);
        wrap = wrap.push(
            button(text(*name).size(13))
                .padding([6, 12])
                .style(crate::theme::tab_button(p, active))
                .on_press(Message::ToggleCondition(name.to_string())),
        );
        if i % 4 == 3 {
            rows = rows.push(wrap);
            wrap = row![].spacing(8);
        }
    }
    rows = rows.push(wrap);
    rows = rows.push(Space::with_height(0));

    widgets::section(p, "Conditions", rows)
}
