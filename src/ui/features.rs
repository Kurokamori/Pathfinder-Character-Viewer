//! Features tab: two columns. Left lists what you have (feats, racial traits,
//! class abilities) as expandable cards; right is the feat selector.

use crate::app::{App, CustomList, Message};
use crate::model::compendium::Feat;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    row![
        container(scrollable(owned_column(app, p)).height(Length::Fill))
            .width(Length::FillPortion(1))
            .height(Length::Fill),
        container(selector_column(app, p))
            .width(Length::FillPortion(1))
            .height(Length::Fill),
    ]
    .spacing(18)
    .height(Length::Fill)
    .into()
}

fn owned_column<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    column![
        owned_feats(app, p),
        racial_traits(app, p),
        languages(app, p),
        class_abilities(app, p),
    ]
    .spacing(18)
    .into()
}

fn expand_key_matches(app: &App, key: &str) -> bool {
    app.expanded_feature.as_deref() == Some(key)
}

fn owned_feats<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let mut list = column![].spacing(8);
    if ch.feats.is_empty() && ch.custom_feats.is_empty() {
        list = list.push(caption(p, "No feats selected yet. Add from the browser on the right."));
    }
    for id in &ch.feats {
        if let Some(feat) = app.game.feat(id) {
            let key = format!("feat:{id}");
            let action = button(text("Remove").size(11))
                .padding([6, 10])
                .style(crate::theme::ghost_button(p))
                .on_press(Message::FeatRemove(id.clone()));
            list = list.push(widgets::expandable_card(
                p,
                Message::FeatureExpand(key.clone()),
                expand_key_matches(app, &key),
                feat.name.clone(),
                feat.types.join(", "),
                feat.description.clone(),
                Some(action.into()),
            ));
        }
    }
    for entry in &ch.custom_feats {
        list = list.push(widgets::custom_row(
            p,
            CustomList::Feat,
            entry.uid,
            &entry.name,
            entry.level,
            app.editors
                .get(&crate::app::EditorTarget::CustomDesc(CustomList::Feat, entry.uid)),
            false,
        ));
    }

    let add = row![
        Space::with_width(Length::Fill),
        widgets::ghost_button(p, "+ Custom Feat", Message::CustomAdd(CustomList::Feat)),
    ];
    widgets::section(p, "Your Feats", column![list, add].spacing(12))
}

fn racial_traits<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let mut inner = column![].spacing(10);

    let blurb = if ch.race.is_empty() {
        "No race selected. Choose one on the General tab.".to_string()
    } else if let Some(race) = app.game.compendium.races.iter().find(|r| r.name == ch.race) {
        let mods = race
            .ability_mods
            .iter()
            .map(|(k, v)| format!("{} {}", k.to_uppercase(), crate::rules::display::signed(*v)))
            .collect::<Vec<_>>()
            .join(", ");
        if mods.is_empty() {
            format!("{} ({})", race.name, race.size)
        } else {
            format!("{} ({}) · {}", race.name, race.size, mods)
        }
    } else {
        ch.race.clone()
    };
    inner = inner.push(caption(p, blurb));

    for entry in &ch.racial_traits {
        inner = inner.push(widgets::custom_row(
            p,
            CustomList::RacialTrait,
            entry.uid,
            &entry.name,
            entry.level,
            app.editors
                .get(&crate::app::EditorTarget::CustomDesc(CustomList::RacialTrait, entry.uid)),
            false,
        ));
    }

    let add = row![
        Space::with_width(Length::Fill),
        widgets::ghost_button(p, "+ Racial Trait", Message::CustomAdd(CustomList::RacialTrait)),
    ];
    widgets::section(p, "Racial Traits", column![inner, add].spacing(12))
}

fn languages<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let target = crate::app::EditorTarget::Languages;
    let body = column![
        caption(
            p,
            "One per line. Bonus languages come from Intelligence and ranks in Linguistics.",
        ),
        widgets::growing_editor(p, app.editors.get(&target), "Common", target),
    ]
    .spacing(6);

    widgets::section(p, "Languages", body)
}

fn class_abilities<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let mut list = column![].spacing(8);
    if ch.selected_abilities.is_empty() && ch.custom_abilities.is_empty() {
        list = list.push(caption(p, "No class abilities selected yet."));
    }
    for id in &ch.selected_abilities {
        if let Some(ability) = app.game.ability(id) {
            let key = format!("ability:{id}");
            list = list.push(widgets::expandable_card(
                p,
                Message::FeatureExpand(key.clone()),
                expand_key_matches(app, &key),
                ability.name.clone(),
                ability.category.label().to_string(),
                ability.description.clone(),
                None,
            ));
        }
    }
    for entry in &ch.custom_abilities {
        list = list.push(widgets::custom_row(
            p,
            CustomList::Ability,
            entry.uid,
            &entry.name,
            entry.level,
            app.editors
                .get(&crate::app::EditorTarget::CustomDesc(CustomList::Ability, entry.uid)),
            false,
        ));
    }
    widgets::section(p, "Class Abilities", list)
}

fn selector_column<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let search = text_input("Search feats...", &app.feat_search)
        .on_input(Message::FeatSearch)
        .padding([8, 12])
        .style(crate::theme::input(p));

    let needle = app.feat_search.to_lowercase();
    let mut matches: Vec<&Feat> = app
        .game
        .compendium
        .feats
        .iter()
        .filter(|f| app.settings.allows(&f.source))
        .filter(|f| needle.is_empty() || f.name.to_lowercase().contains(&needle))
        .collect();
    matches.sort_by(|a, b| a.name.cmp(&b.name));

    let mut list = column![].spacing(8);
    for feat in matches.iter().take(300) {
        let owned = app.character.feats.contains(&feat.id);
        let expanded = app.selected_feat.as_deref() == Some(feat.id.as_str());
        let toggle = if owned {
            Message::FeatRemove(feat.id.clone())
        } else {
            Message::FeatAdd(feat.id.clone())
        };
        let action = {
            let base = button(text(if owned { "Remove" } else { "Add" }).size(11)).padding([6, 10]);
            if owned {
                base.style(crate::theme::danger_button(p)).on_press(toggle)
            } else {
                base.style(crate::theme::accent_button(p)).on_press(toggle)
            }
        };
        let expand = if expanded {
            Message::FeatSelect(None)
        } else {
            Message::FeatSelect(Some(feat.id.clone()))
        };
        list = list.push(widgets::expandable_card(
            p,
            expand,
            expanded,
            feat.name.clone(),
            feat.types.join(", "),
            feat.description.clone(),
            Some(action.into()),
        ));
    }
    if matches.len() > 300 {
        list = list.push(caption(p, format!("Showing first 300 of {}.", matches.len())));
    }

    column![
        widgets::heading(p, "Feat Browser"),
        search,
        widgets::browse_list(list),
    ]
    .spacing(14)
    .height(Length::Fill)
    .into()
}
