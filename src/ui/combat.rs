//! Combat tab: attack/defense summary and manual bonus editing, two columns.

use crate::app::{App, BonusField, Message};
use crate::model::character::InventoryItem;
use crate::model::compendium::{ItemKind, WeaponStats};
use crate::rules::derived::DerivedStats;
use crate::rules::display::signed;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let d = app.derived();
    let ch = &app.character;

    let attacks = row![
        widgets::stat_tile(p, "Melee", signed(d.melee_attack), Some("attack".to_string())),
        widgets::stat_tile(p, "Ranged", signed(d.ranged_attack), Some("attack".to_string())),
    ]
    .spacing(10);
    let attacks_b = row![
        widgets::stat_tile(p, "CMB", signed(d.cmb), None),
        widgets::stat_tile(p, "CMD", d.cmd.to_string(), None),
        widgets::stat_tile(p, "BAB", signed(d.bab), None),
    ]
    .spacing(10);
    let attacks_section = widgets::section(p, "Attacks", row![attacks, attacks_b].spacing(10));

    let defenses = row![
        widgets::stat_tile(p, "AC", d.ac.to_string(), None),
        widgets::stat_tile(p, "Touch", d.touch.to_string(), None),
        widgets::stat_tile(p, "Flat", d.flat_footed.to_string(), None),
    ]
    .spacing(10);
    let defenses2 = row![
        widgets::stat_tile(p, "Armor", signed(d.armor_bonus), None),
        widgets::stat_tile(p, "Shield", signed(d.shield_bonus), None),
    ]
    .spacing(10);
    let defenses_section =
        widgets::section(p, "Defenses", column![defenses, defenses2].spacing(10));

    let ac_bonuses = column![
        row![
            widgets::number_field(p, "Natural Armor", ch.bonuses.natural_armor, 110.0, |v| {
                Message::SetBonus(BonusField::NaturalArmor, v)
            }),
            widgets::number_field(p, "Deflection", ch.bonuses.deflection, 110.0, |v| {
                Message::SetBonus(BonusField::Deflection, v)
            }),
            widgets::number_field(p, "Dodge", ch.bonuses.dodge, 110.0, |v| Message::SetBonus(
                BonusField::Dodge,
                v
            )),
            widgets::number_field(p, "Misc AC", ch.bonuses.misc_ac, 110.0, |v| Message::SetBonus(
                BonusField::MiscAc,
                v
            )),
        ].spacing(12),
    ]
    .spacing(12);

    let save_bonuses = column![
        row![
            widgets::number_field(p, "Fort Misc", ch.bonuses.fort_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Fort, v)
            }),
            widgets::number_field(p, "Ref Misc", ch.bonuses.ref_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Ref, v)
            }),
            widgets::number_field(p, "Will Misc", ch.bonuses.will_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Will, v)
            }),
            widgets::number_field(p, "Init Misc", ch.bonuses.init_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Init, v)
            }),
        ].spacing(12),
    ]
    .spacing(12);

    let attack_bonuses = column![
        row![
            widgets::number_field(p, "BAB Misc", ch.bonuses.bab_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Bab, v)
            }),
            widgets::number_field(p, "Attack Misc", ch.bonuses.attack_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Attack, v)
            }),

            widgets::number_field(p, "CMB Misc", ch.bonuses.cmb_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Cmb, v)
            }),
            widgets::number_field(p, "CMD Misc", ch.bonuses.cmd_misc, 110.0, |v| {
                Message::SetBonus(BonusField::Cmd, v)
            }),
        ].spacing(12),
    ]
    .spacing(12);

    let other = row![
        widgets::number_field(p, "HP Misc", ch.bonuses.hp_misc, 110.0, |v| Message::SetBonus(
            BonusField::HpMisc,
            v
        )),
        widgets::number_field(p, "Base Speed", ch.base_speed, 110.0, |v| Message::SetBonus(
            BonusField::Speed,
            v
        )),
    ].spacing(12);

    let left = column![
        attacks_section,
        weapon_attacks_section(app, p, &d),
        defenses_section,
        equipped_note(app, p, &d)
    ]
    .spacing(18);
    let right = column![
        widgets::section(p, "Armor Class Bonuses", ac_bonuses),
        widgets::section(p, "Saves & Initiative", save_bonuses),
        widgets::section(p, "Attack Bonuses", attack_bonuses),
        widgets::section(p, "Other", other),
    ]
    .spacing(18);

    row![
        column![left].width(Length::FillPortion(1)),
        column![right].width(Length::FillPortion(1)),
    ]
    .spacing(18)
    .into()
}

/// Lists each equipped weapon with its computed attack bonus, damage, and crit.
fn weapon_attacks_section<'a>(
    app: &'a App,
    p: Palette,
    d: &DerivedStats,
) -> Element<'a, Message> {
    let weapons: Vec<&InventoryItem> = app
        .character
        .inventory
        .iter()
        .filter(|i| i.equipped && i.kind == ItemKind::Weapon)
        .collect();

    if weapons.is_empty() {
        return widgets::section(
            p,
            "Weapon Attacks",
            caption(p, "No weapons equipped. Equip a weapon in the Inventory tab."),
        );
    }

    let str_mod = app.character.ability_mod("str");
    let mut list = column![].spacing(8);
    for item in weapons {
        let stats = item
            .source_id
            .as_ref()
            .and_then(|id| app.game.item(id))
            .and_then(|template| template.weapon.as_ref());
        list = list.push(weapon_row(p, d, str_mod, &item.name, stats));
    }

    widgets::section(p, "Weapon Attacks", list)
}

fn weapon_row<'a>(
    p: Palette,
    d: &DerivedStats,
    str_mod: i32,
    name: &str,
    stats: Option<&WeaponStats>,
) -> Element<'a, Message> {
    let name_cell = container(text(name.to_string()).size(14).color(p.text))
        .width(Length::FillPortion(3));

    let body: Element<Message> = match stats {
        Some(w) => {
            let attack_total = if w.melee {
                d.melee_attack
            } else {
                d.ranged_attack
            };
            let damage_mod = if w.damage_ability == "str" { str_mod } else { 0 };
            let damage = if w.damage.is_empty() {
                "—".to_string()
            } else if damage_mod == 0 {
                w.damage.clone()
            } else {
                format!("{}{}", w.damage, signed(damage_mod))
            };
            let mut meta = vec![
                if w.melee { "Melee".to_string() } else { "Ranged".to_string() },
            ];
            if !w.range.is_empty() {
                meta.push(w.range.clone());
            }
            if !w.damage_types.is_empty() {
                meta.push(w.damage_types.join(", "));
            }

            row![
                stat_cell(p, "Attack", signed(attack_total)),
                stat_cell(p, "Damage", damage),
                stat_cell(p, "Crit", w.crit_label()),
                container(caption(p, meta.join(" · "))).width(Length::FillPortion(3)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(6))
            .into()
        }
        None => container(caption(p, "No attack data — enter manually below."))
            .width(Length::FillPortion(6))
            .into(),
    };

    container(row![name_cell, body].spacing(12).align_y(Alignment::Center))
        .padding([8, 12])
        .style(crate::theme::plain_row(p))
        .into()
}

fn stat_cell<'a>(p: Palette, label: &'a str, value: String) -> Element<'a, Message> {
    column![
        caption(p, label),
        text(value).size(16).color(p.text),
    ]
    .spacing(2)
    .into()
}

fn equipped_note<'a>(app: &'a App, p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let equipped: Vec<String> = app
        .character
        .inventory
        .iter()
        .filter(|i| i.equipped)
        .map(|i| i.name.clone())
        .collect();
    let text = if equipped.is_empty() {
        "No armor or shields equipped. Equip items in the Inventory tab.".to_string()
    } else {
        format!(
            "Equipped: {}  ·  Armor check penalty {}",
            equipped.join(", "),
            signed(d.armor_check_penalty)
        )
    };
    widgets::card(p, iced::widget::text(text).size(13).color(p.text_dim))
}
