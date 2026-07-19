//! Spells tab: per-day slots, prepared list, and browse/learn/prepare/cast.

use crate::app::{App, Message, SpellFilter};
use crate::model::compendium::Spell;
use crate::rules::display;
use crate::rules::progression::ClassDef;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let casting = crate::rules::derived::casting_class(&app.character);
    let (class, def) = match casting {
        Some(c) => c,
        None => {
            return column![
                widgets::heading(p, "Spells"),
                widgets::placeholder(p, "This class does not cast spells."),
            ]
            .spacing(16)
            .into();
        }
    };
    let class_tag = class.tag.clone();
    let class_level = class.level;
    let ability_mod = app.character.ability_mod(def.casting_ability);

    column![
        slot_summary(app, p, &def, class_level, ability_mod),
        prepared_section(app, p, &def),
        browse_section(app, p, &class_tag),
    ]
    .spacing(18)
    .height(Length::Fill)
    .into()
}

fn slot_summary<'a>(
    app: &'a App,
    p: Palette,
    def: &ClassDef,
    class_level: u32,
    ability_mod: i32,
) -> Element<'a, Message> {
    let max_level = def.max_spell_level(class_level).unwrap_or(0);
    let mut tiles = row![].spacing(10);
    for lvl in 0..=max_level {
        let base = def.base_spells_per_day(class_level, lvl).unwrap_or(0);
        let bonus = def.bonus_spells(lvl, ability_mod);
        let total = base + bonus;
        let dc = 10 + lvl as i32 + ability_mod;
        let prepared = app
            .character
            .spellbook
            .prepared
            .iter()
            .filter(|pr| spell_level(app, &pr.spell_id, &def_tag(app)) == Some(lvl as u8))
            .count();
        tiles = tiles.push(widgets::stat_tile(
            p,
            spell_level_short(lvl),
            format!("{prepared}/{total}"),
            Some(format!("DC {dc}")),
        ));
    }

    let header = row![
        widgets::heading(p, "Spells per Day"),
        Space::with_width(Length::Fill),
        widgets::primary_button(p, "Rest (reset)", Message::SpellRest),
    ]
    .align_y(Alignment::Center);

    widgets::card(p, column![header, tiles].spacing(12))
}

fn def_tag(app: &App) -> String {
    crate::rules::derived::casting_class(&app.character)
        .map(|(c, _)| c.tag.clone())
        .unwrap_or_default()
}

fn prepared_section<'a>(app: &'a App, p: Palette, _def: &ClassDef) -> Element<'a, Message> {
    let tag = def_tag(app);
    if app.character.spellbook.prepared.is_empty() {
        return widgets::section(
            p,
            "Prepared Today",
            text("No spells prepared. Learn spells below, then prepare them.")
                .size(13)
                .color(p.text_dim),
        );
    }

    let mut prepared: Vec<_> = app.character.spellbook.prepared.iter().collect();
    prepared.sort_by_key(|pr| spell_level(app, &pr.spell_id, &tag).unwrap_or(0));

    let mut list = column![].spacing(8);
    for pr in prepared {
        let spell = app.game.spell(&pr.spell_id);
        let name = spell.map(|s| s.name.clone()).unwrap_or_else(|| pr.spell_id.clone());
        let lvl = spell_level(app, &pr.spell_id, &tag).unwrap_or(0);
        let name_color = if pr.used { p.text_dim } else { p.text };

        let cast_base = button(text("Cast").size(12))
            .padding([5, 12])
            .on_press(Message::SpellToggleUsed(pr.uid));
        let cast = if pr.used {
            cast_base.style(crate::theme::subtle_button(p))
        } else {
            cast_base.style(crate::theme::accent_button(p))
        };

        let remove = button(text("Remove").size(12))
            .padding([5, 10])
            .style(crate::theme::ghost_button(p))
            .on_press(Message::SpellUnprepare(pr.uid));

        let rowel = row![
            container(text(display::spell_level_label(lvl as usize)).size(11).color(p.text_dim))
                .width(Length::Fixed(70.0)),
            container(text(name).size(14).color(name_color)).width(Length::Fill),
            if pr.used {
                widgets::pill(p, "used")
            } else {
                Space::with_width(0).into()
            },
            cast,
            remove,
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        list = list.push(container(rowel).padding([8, 12]).style(crate::theme::plain_row(p)));
    }

    widgets::section(p, "Prepared Today", list)
}

fn browse_section<'a>(app: &'a App, p: Palette, class_tag: &str) -> Element<'a, Message> {
    let mut level_row = row![filter_button(p, app, SpellFilter::Level(None), "All")].spacing(6);
    for lvl in 0u8..=9 {
        level_row = level_row.push(filter_button(
            p,
            app,
            SpellFilter::Level(Some(lvl)),
            &spell_level_short(lvl as usize),
        ));
    }

    let mut known_row = row![filter_button(p, app, SpellFilter::Known(None), "K-All")].spacing(6);
    for lvl in 0u8..=9 {
        known_row = known_row.push(filter_button(
            p,
            app,
            SpellFilter::Known(Some(lvl)),
            &known_level_short(lvl as usize),
        ));
    }

    let search = text_input("Search spells...", &app.spell_search)
        .on_input(Message::SpellSearch)
        .padding([8, 12])
        .style(crate::theme::input(p));

    let needle = app.spell_search.to_lowercase();
    let mut matches: Vec<(&Spell, u8)> = app
        .game
        .spells_for_class(class_tag, &app.settings)
        .filter(|(s, lvl)| match app.spell_filter {
            SpellFilter::Level(None) => true,
            SpellFilter::Level(Some(f)) => *lvl == f,
            SpellFilter::Known(None) => app.character.spellbook.learned.contains(&s.id),
            SpellFilter::Known(Some(f)) => {
                *lvl == f && app.character.spellbook.learned.contains(&s.id)
            }
        })
        .filter(|(s, _)| needle.is_empty() || s.name.to_lowercase().contains(&needle))
        .collect();
    matches.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.name.cmp(&b.0.name)));

    let mut list = column![].spacing(8);
    for (spell, lvl) in matches.iter().take(400) {
        let owned = app.character.spellbook.learned.contains(&spell.id);
        let active = app.selected_spell.as_deref() == Some(spell.id.as_str());
        let subtitle = format!(
            "{} · {}",
            display::spell_level_label(*lvl as usize),
            display::school_name(&spell.school)
        );
        list = list.push(widgets::browse_row(
            p,
            spell.name.clone(),
            subtitle,
            owned,
            active,
            Message::SpellSelect(Some(spell.id.clone())),
            Message::SpellToggleLearned(spell.id.clone()),
        ));
    }
    if matches.len() > 400 {
        list = list.push(caption(p, format!("Showing first 400 of {} spells.", matches.len())));
    }

    let mut custom_list = column![].spacing(8);
    for entry in &app.character.custom_spells {
        custom_list = custom_list.push(widgets::custom_row(
            p,
            crate::app::CustomList::Spell,
            entry.uid,
            &entry.name,
            entry.level,
            app.editors
                .get(&crate::app::EditorTarget::CustomDesc(crate::app::CustomList::Spell, entry.uid)),
            true,
        ));
    }
    let custom_section = column![
        row![
            caption(p, "Homebrew spells"),
            Space::with_width(Length::Fill),
            widgets::ghost_button(
                p,
                "+ Custom Spell",
                Message::CustomAdd(crate::app::CustomList::Spell)
            ),
        ]
        .align_y(Alignment::Center),
        custom_list,
    ]
    .spacing(8);

    let list_panel = container(widgets::browse_list(column![list, custom_section].spacing(14)))
        .width(Length::FillPortion(2))
        .height(Length::Fill);

    let detail = spell_detail(app, p, class_tag);
    let detail_panel = container(scrollable(detail).height(Length::Fill))
        .width(Length::FillPortion(3))
        .height(Length::Fill);

    let controls = column![
        scrollable(level_row),
        scrollable(known_row),
        search,
    ]
    .spacing(10);

    column![
        widgets::heading(p, "Spellbook"),
        controls,
        row![list_panel, detail_panel].spacing(18).height(Length::Fill),
    ]
    .spacing(14)
    .height(Length::Fill)
    .into()
}

fn spell_detail<'a>(app: &'a App, p: Palette, class_tag: &str) -> Element<'a, Message> {
    let spell = match app.selected_spell.as_ref().and_then(|id| app.game.spell(id)) {
        Some(s) => s,
        None => return widgets::placeholder(p, "Select a spell to view its details."),
    };
    let lvl = spell.level_for(class_tag).unwrap_or(0);
    let learned = app.character.spellbook.learned.contains(&spell.id);

    let meta = [
        format!("{} {}", display::school_name(&spell.school), if spell.subschool.is_empty() { String::new() } else { format!("({})", spell.subschool) }),
        display::spell_level_label(lvl as usize),
        display::components_string(spell.verbal, spell.somatic, spell.material, spell.focus, spell.divine_focus),
    ];
    let mut meta_row = row![].spacing(6);
    for m in meta {
        if !m.trim().is_empty() {
            meta_row = meta_row.push(widgets::pill(p, m));
        }
    }

    let stat_line = |label: &'static str, value: &str| -> Element<Message> {
        if value.trim().is_empty() {
            Space::with_height(0).into()
        } else {
            row![
                text(label).size(12).color(p.text_dim).width(Length::Fixed(110.0)),
                text(value.to_string()).size(13).color(p.text),
            ]
            .spacing(8)
            .into()
        }
    };

    let learn_btn = if learned {
        widgets::ghost_button(p, "Forget", Message::SpellToggleLearned(spell.id.clone()))
    } else {
        widgets::primary_button(p, "Learn", Message::SpellToggleLearned(spell.id.clone()))
    };
    let prepare_btn: Element<Message> = if learned {
        widgets::primary_button(p, "Prepare", Message::SpellPrepare(spell.id.clone()))
    } else {
        Space::with_width(0).into()
    };

    let content = column![
        text(&spell.name).size(20).color(p.text),
        meta_row,
        row![learn_btn, prepare_btn].spacing(10),
        widgets::divider(p),
        stat_line("Casting Time", &spell.casting_time),
        stat_line("Range", &spell.range),
        stat_line("Target", &spell.target),
        stat_line("Duration", &spell.duration),
        stat_line("Saving Throw", &spell.save),
        stat_line("Spell Resist.", &spell.spell_resistance),
        widgets::divider(p),
        text(if spell.description.is_empty() {
            "No description available.".to_string()
        } else {
            spell.description.clone()
        })
        .size(13)
        .color(p.text),
    ]
    .spacing(10);

    widgets::card(p, content)
}

fn filter_button<'a>(
    p: Palette,
    app: &App,
    filter: SpellFilter,
    label: &str,
) -> Element<'a, Message> {
    let active = app.spell_filter == filter;
    button(text(label.to_string()).size(13))
        .padding([6, 12])
        .style(crate::theme::tab_button(p, active))
        .on_press(Message::SpellSetFilter(filter))
        .into()
}

fn spell_level(app: &App, spell_id: &str, class_tag: &str) -> Option<u8> {
    app.game.spell(spell_id).and_then(|s| s.level_for(class_tag))
}

fn spell_level_short(level: usize) -> String {
    if level == 0 {
        "Cant".to_string()
    } else {
        format!("L{level}")
    }
}

fn known_level_short(level: usize) -> String {
    if level == 0 {
        "K-Cant".to_string()
    } else {
        format!("K-{level}")
    }
}
