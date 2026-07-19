//! Ninja-specific screen: the ki pool tracker and ninja/master trick selection.

use crate::app::{App, Message};
use crate::model::compendium::{Ability, AbilityCategory};
use crate::ui::widgets;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

const TRICK_CATEGORIES: [(&str, Option<AbilityCategory>); 3] = [
    ("All", None),
    ("Tricks", Some(AbilityCategory::NinjaTrick)),
    ("Master", Some(AbilityCategory::MasterTrick)),
];

fn is_trick(category: AbilityCategory) -> bool {
    matches!(
        category,
        AbilityCategory::NinjaTrick | AbilityCategory::MasterTrick
    )
}

pub fn ki_view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let ninja_level = app.character.class_level("ninja");
    let allowance = crate::classes::trick_allowance(ninja_level);
    let sneak = crate::classes::sneak_attack_dice(ninja_level);
    let selected_count = app
        .character
        .selected_abilities
        .iter()
        .filter(|id| {
            app.game
                .ability(id)
                .map(|a| is_trick(a.category))
                .unwrap_or(false)
        })
        .count();

    let ki = app.character.resources.get("ki");
    let (ki_current, ki_max) = ki.map(|k| (k.current, k.max)).unwrap_or((0, 0));

    let ki_pool = widgets::card(
        p,
        row![
            column![
                text("Ki Pool").size(13).color(p.text_dim),
                text(format!("{ki_current} / {ki_max}")).size(30).color(p.accent),
            ]
            .spacing(2),
            Space::with_width(Length::Fill),
            row![
                button(text("Spend").size(13))
                    .padding([8, 14])
                    .style(crate::theme::subtle_button(p))
                    .on_press(Message::KiDelta(-1)),
                button(text("Regain").size(13))
                    .padding([8, 14])
                    .style(crate::theme::subtle_button(p))
                    .on_press(Message::KiDelta(1)),
                button(text("Rest").size(13))
                    .padding([8, 14])
                    .style(crate::theme::accent_button(p))
                    .on_press(Message::KiRest),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center),
    );

    let summary = row![
        widgets::stat_tile(p, "Ninja Level", ninja_level.to_string(), None),
        widgets::stat_tile(p, "Sneak Attack", format!("{sneak}d6"), None),
        widgets::stat_tile(
            p,
            "Tricks",
            format!("{selected_count} / {allowance}"),
            Some("selected".to_string())
        ),
    ]
    .spacing(12);

    let mut category_row = row![].spacing(6);
    for (label, cat) in TRICK_CATEGORIES {
        let active = app.hex_category == cat;
        category_row = category_row.push(
            button(text(label).size(13))
                .padding([6, 14])
                .style(crate::theme::tab_button(p, active))
                .on_press(Message::HexCategoryFilter(cat)),
        );
    }

    let search = text_input("Search tricks...", &app.hex_search)
        .on_input(Message::HexSearch)
        .padding([8, 12])
        .style(crate::theme::input(p));

    let controls = row![
        category_row,
        Space::with_width(Length::Fill),
        container(search).width(Length::Fixed(280.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let needle = app.hex_search.to_lowercase();
    let mut matches: Vec<&Ability> = app
        .game
        .compendium
        .abilities
        .iter()
        .filter(|a| a.classes.iter().any(|c| c == "ninja"))
        .filter(|a| is_trick(a.category))
        .filter(|a| app.settings.allows(&a.source))
        .filter(|a| match app.hex_category {
            Some(cat) => a.category == cat,
            None => true,
        })
        .filter(|a| needle.is_empty() || a.name.to_lowercase().contains(&needle))
        .collect();
    matches.sort_by(|a, b| a.name.cmp(&b.name));

    let mut list = column![].spacing(8);
    for ability in matches {
        let owned = app.character.selected_abilities.contains(&ability.id);
        let active = app.selected_hex.as_deref() == Some(ability.id.as_str());
        list = list.push(widgets::browse_row(
            p,
            ability.name.clone(),
            ability.category.label().to_string(),
            owned,
            active,
            Message::HexSelect(Some(ability.id.clone())),
            Message::TrickToggle(ability.id.clone()),
        ));
    }

    let custom_section = column![
        row![
            widgets::caption(p, "Homebrew tricks"),
            Space::with_width(Length::Fill),
            widgets::ghost_button(
                p,
                "+ Custom Trick",
                Message::CustomAdd(crate::app::CustomList::Ability)
            ),
        ]
        .align_y(Alignment::Center),
        custom_trick_list(app, p),
    ]
    .spacing(8);

    let list_panel = container(widgets::browse_list(column![list, custom_section].spacing(14)))
        .width(Length::FillPortion(2))
        .height(Length::Fill);

    let detail = match app.selected_hex.as_ref().and_then(|id| app.game.ability(id)) {
        Some(ability) => widgets::detail_panel(
            p,
            ability.name.clone(),
            vec![ability.category.label().to_string(), ability.source.clone()],
            ability.description.clone(),
        ),
        None => widgets::placeholder(p, "Select a trick to read its description."),
    };
    let detail_panel = container(scrollable(detail).height(Length::Fill))
        .width(Length::FillPortion(3))
        .height(Length::Fill);

    let body = row![list_panel, detail_panel]
        .spacing(18)
        .height(Length::Fill);

    column![
        widgets::heading(p, "Ki & Tricks"),
        ki_pool,
        summary,
        widgets::divider(p),
        controls,
        body,
    ]
    .spacing(16)
    .height(Length::Fill)
    .into()
}

fn custom_trick_list<'a>(app: &'a App, p: crate::theme::Palette) -> Element<'a, Message> {
    let mut list = column![].spacing(8);
    for entry in &app.character.custom_abilities {
        list = list.push(widgets::custom_row(
            p,
            crate::app::CustomList::Ability,
            entry.uid,
            &entry.name,
            entry.level,
            &entry.description,
            false,
        ));
    }
    list.into()
}
