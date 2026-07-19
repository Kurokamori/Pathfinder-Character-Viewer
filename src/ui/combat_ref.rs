//! Combat Reference tab: a compact, at-a-glance view of everything a caster can
//! reach for in a fight — prepared spells (with cast tracking) and learned hexes
//! or equivalent class abilities. Each entry expands in place to reveal its full
//! details, the way inventory rows do. Nothing here is editable beyond marking a
//! spell cast and resting.

use crate::app::{App, Message};
use crate::model::compendium::{Ability, AbilityCategory, Spell};
use crate::rules::derived;
use crate::rules::display;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();

    let mut sections = column![widgets::heading(p, "Combat Reference")].spacing(18);
    sections = sections.push(prepared_spells(app, p));

    if let Some(hexes) = hex_abilities(app, p) {
        sections = sections.push(hexes);
    }
    if let Some(other) = other_abilities(app, p) {
        sections = sections.push(other);
    }

    sections.into()
}

fn is_expanded(app: &App, key: &str) -> bool {
    app.expanded_combat_ref.as_deref() == Some(key)
}

fn prepared_spells<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let casting = derived::casting_class(&app.character);
    let class_tag = match casting {
        Some((class, _def)) => class.tag.clone(),
        None => {
            return widgets::section(
                p,
                "Prepared Spells",
                caption(p, "This class does not cast spells."),
            );
        }
    };

    let header = row![
        widgets::heading(p, "Prepared Spells"),
        Space::with_width(Length::Fill),
        widgets::primary_button(p, "Rest (reset)", Message::SpellRest),
    ]
    .align_y(Alignment::Center);

    if app.character.spellbook.prepared.is_empty() {
        return widgets::card(
            p,
            column![
                header,
                caption(p, "Nothing prepared. Prepare spells on the Spells tab."),
            ]
            .spacing(12),
        );
    }

    let mut prepared: Vec<_> = app.character.spellbook.prepared.iter().collect();
    prepared.sort_by_key(|pr| {
        app.game
            .spell(&pr.spell_id)
            .and_then(|s| s.level_for(&class_tag))
            .unwrap_or(0)
    });

    let mut list = column![].spacing(6);
    for pr in prepared {
        let spell = app.game.spell(&pr.spell_id);
        let name = spell
            .map(|s| s.name.clone())
            .unwrap_or_else(|| pr.spell_id.clone());
        let lvl = spell.and_then(|s| s.level_for(&class_tag)).unwrap_or(0) as usize;
        let dc = derived::spell_save_dc(&app.character, lvl);
        let name_color = if pr.used { p.text_dim } else { p.text };

        let key = format!("spell:{}", pr.uid);
        let expanded = is_expanded(app, &key);

        let cast_base = button(text("Cast").size(12))
            .padding([5, 12])
            .on_press(Message::SpellToggleUsed(pr.uid));
        let cast = if pr.used {
            cast_base.style(crate::theme::subtle_button(p))
        } else {
            cast_base.style(crate::theme::accent_button(p))
        };

        let dc_pill: Element<Message> = match dc {
            Some(dc) if lvl > 0 => widgets::pill(p, format!("DC {dc}")),
            _ => Space::with_width(0).into(),
        };

        let head_button = button(
            row![
                container(
                    text(display::spell_level_label(lvl))
                        .size(11)
                        .color(p.text_dim)
                )
                .width(Length::Fixed(70.0)),
                container(text(name).size(14).color(name_color)).width(Length::Fill),
                text(if expanded { "Hide" } else { "Details" })
                    .size(11)
                    .color(p.text_dim),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(crate::theme::list_button(p, expanded))
        .on_press(Message::CombatRefExpand(key));

        let head = row![
            container(head_button).width(Length::Fill),
            dc_pill,
            if pr.used {
                widgets::pill(p, "used")
            } else {
                Space::with_width(0).into()
            },
            cast,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut entry = column![head].spacing(8);
        if expanded {
            entry = entry.push(spell_detail(p, spell, lvl));
        }
        list = list.push(entry);
    }

    widgets::card(p, column![header, list].spacing(12))
}

fn spell_detail<'a>(p: Palette, spell: Option<&'a Spell>, lvl: usize) -> Element<'a, Message> {
    let spell = match spell {
        Some(s) => s,
        None => return widgets::info_box(p, "No compendium data for this spell.".to_string()),
    };

    let mut meta = vec![
        format!(
            "{} · {}",
            display::school_name(&spell.school),
            display::spell_level_label(lvl)
        ),
        display::components_string(
            spell.verbal,
            spell.somatic,
            spell.material,
            spell.focus,
            spell.divine_focus,
        ),
    ];
    meta.retain(|m| !m.trim().is_empty());

    let mut meta_row = row![].spacing(6);
    for m in meta {
        meta_row = meta_row.push(widgets::pill(p, m));
    }

    let mut lines = column![meta_row].spacing(6);
    for (label, value) in [
        ("Casting Time", &spell.casting_time),
        ("Range", &spell.range),
        ("Target", &spell.target),
        ("Duration", &spell.duration),
        ("Saving Throw", &spell.save),
        ("Spell Resist.", &spell.spell_resistance),
    ] {
        if !value.trim().is_empty() {
            lines = lines.push(
                row![
                    text(label).size(12).color(p.text_dim).width(Length::Fixed(110.0)),
                    text(value.clone()).size(12).color(p.text),
                ]
                .spacing(8),
            );
        }
    }
    lines = lines.push(widgets::info_box(p, spell.description.clone()));

    container(lines)
        .padding([4, 8])
        .width(Length::Fill)
        .into()
}

fn is_hex(category: AbilityCategory) -> bool {
    matches!(
        category,
        AbilityCategory::Hex | AbilityCategory::MajorHex | AbilityCategory::GrandHex
    )
}

fn hex_abilities<'a>(app: &'a App, p: Palette) -> Option<Element<'a, Message>> {
    let owned: Vec<&Ability> = app
        .character
        .selected_abilities
        .iter()
        .filter_map(|id| app.game.ability(id))
        .filter(|a| is_hex(a.category))
        .collect();

    let has_custom = !app.character.custom_abilities.is_empty();
    if owned.is_empty() && !has_custom {
        return None;
    }

    let witch_level = app.character.class_level("witch");
    let int_mod = app.character.ability_mod("int");
    let hex_dc = 10 + witch_level as i32 / 2 + int_mod;

    let header = row![
        widgets::heading(p, "Hexes"),
        Space::with_width(Length::Fill),
        widgets::pill(p, format!("Save DC {hex_dc}")),
    ]
    .align_y(Alignment::Center);

    let mut list = column![].spacing(6);
    for ability in owned {
        list = list.push(ability_row(
            app,
            p,
            &format!("ability:{}", ability.id),
            &ability.name,
            &format!(
                "{} · {}",
                ability.category.label(),
                ability.ability_type.to_uppercase()
            ),
            &ability.description,
        ));
    }
    for entry in &app.character.custom_abilities {
        list = list.push(ability_row(
            app,
            p,
            &format!("custom-ability:{}", entry.uid),
            &entry.name,
            "Homebrew hex",
            &entry.description,
        ));
    }

    Some(widgets::card(p, column![header, list].spacing(12)))
}

/// Selected class abilities that are not hexes (e.g. ninja tricks), so the
/// reference stays useful for non-witch casters and multiclass characters.
fn other_abilities<'a>(app: &'a App, p: Palette) -> Option<Element<'a, Message>> {
    let owned: Vec<&Ability> = app
        .character
        .selected_abilities
        .iter()
        .filter_map(|id| app.game.ability(id))
        .filter(|a| !is_hex(a.category))
        .collect();

    if owned.is_empty() {
        return None;
    }

    let mut list = column![].spacing(6);
    for ability in owned {
        list = list.push(ability_row(
            app,
            p,
            &format!("ability:{}", ability.id),
            &ability.name,
            ability.category.label(),
            &ability.description,
        ));
    }

    Some(widgets::section(p, "Other Abilities", list))
}

fn ability_row<'a>(
    app: &'a App,
    p: Palette,
    key: &str,
    name: &str,
    subtitle: &str,
    description: &str,
) -> Element<'a, Message> {
    let expanded = is_expanded(app, key);

    let head = button(
        row![
            container(text(name.to_string()).size(14).color(p.text)).width(Length::Fill),
            text(subtitle.to_string()).size(11).color(p.text_dim),
            text(if expanded { "Hide" } else { "Details" })
                .size(11)
                .color(p.text_dim),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([7, 12])
    .width(Length::Fill)
    .style(crate::theme::list_button(p, expanded))
    .on_press(Message::CombatRefExpand(key.to_string()));

    let mut entry = column![head].spacing(8);
    if expanded {
        entry = entry.push(widgets::info_box(p, description.to_string()));
    }
    entry.into()
}
