//! Witch-specific screens: hex selection with patron/theme filtering, and the
//! editable familiar sheet.

use crate::app::{
    App, CustomSkillField, CustomSkillFlag, FamiliarField, Message, SkillScope,
};
use crate::model::character::{CustomSkill, Familiar, Size, ABILITIES};
use crate::model::compendium::{Ability, AbilityCategory};
use crate::rules::display::signed;
use crate::rules::skills::SKILLS;
use crate::theme::Palette;
use crate::ui::images::ImageCache;
use crate::ui::widgets::{self, caption};
use iced::widget::{
    button, checkbox, column, container, image, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

const HEX_CATEGORIES: [(&str, Option<AbilityCategory>); 4] = [
    ("All", None),
    ("Hexes", Some(AbilityCategory::Hex)),
    ("Major", Some(AbilityCategory::MajorHex)),
    ("Grand", Some(AbilityCategory::GrandHex)),
];

fn is_hex(category: AbilityCategory) -> bool {
    matches!(
        category,
        AbilityCategory::Hex | AbilityCategory::MajorHex | AbilityCategory::GrandHex
    )
}

pub fn hexes_view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let witch_level = app.character.class_level("witch");
    let allowance = crate::classes::hex_allowance(witch_level);
    let selected_count = app
        .character
        .selected_abilities
        .iter()
        .filter(|id| {
            app.game
                .ability(id)
                .map(|a| is_hex(a.category))
                .unwrap_or(false)
        })
        .count();

    let header = row![
        widgets::heading(p, "Hexes"),
        Space::with_width(Length::Fill),
        widgets::pill(p, format!("{selected_count} / {allowance} selected")),
    ]
    .align_y(Alignment::Center);

    let mut category_row = row![].spacing(6);
    for (label, cat) in HEX_CATEGORIES {
        let active = app.hex_category == cat;
        category_row = category_row.push(
            button(text(label).size(13))
                .padding([6, 14])
                .style(crate::theme::tab_button(p, active))
                .on_press(Message::HexCategoryFilter(cat)),
        );
    }

    let patrons = patron_names(app);
    let patron_pick = iced::widget::pick_list(
        patrons,
        Some(current_patron(app)),
        Message::SetPatron,
    )
    .placeholder("Choose patron")
    .padding([7, 10])
    .text_size(14)
    .style(crate::theme::dropdown(p));

    let search = text_input("Search hexes...", &app.hex_search)
        .on_input(Message::HexSearch)
        .padding([8, 12])
        .style(crate::theme::input(p));

    let controls = column![
        row![category_row, Space::with_width(Length::Fill)].align_y(Alignment::Center),
        row![
            column![caption(p, "Patron / Theme"), patron_pick].spacing(4).width(Length::Fixed(260.0)),
            column![caption(p, "Filter"), search].spacing(4).width(Length::Fill),
        ]
        .spacing(14),
    ]
    .spacing(12);

    let needle = app.hex_search.to_lowercase();
    let mut matches: Vec<&Ability> = app
        .game
        .compendium
        .abilities
        .iter()
        .filter(|a| a.classes.iter().any(|c| c == "witch"))
        .filter(|a| is_hex(a.category))
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
        let subtitle = format!(
            "{} · {}",
            ability.category.label(),
            ability.ability_type.to_uppercase()
        );
        list = list.push(widgets::browse_row(
            p,
            ability.name.clone(),
            subtitle,
            owned,
            active,
            Message::HexSelect(Some(ability.id.clone())),
            Message::HexToggle(ability.id.clone()),
        ));
    }

    let custom_section = column![
        row![
            widgets::caption(p, "Homebrew hexes"),
            Space::with_width(Length::Fill),
            widgets::ghost_button(
                p,
                "+ Custom Hex",
                Message::CustomAdd(crate::app::CustomList::Ability)
            ),
        ]
        .align_y(Alignment::Center),
        custom_hex_list(app, p),
    ]
    .spacing(8);

    let list_panel = container(widgets::browse_list(column![list, custom_section].spacing(14)))
        .width(Length::FillPortion(2))
        .height(Length::Fill);

    let detail = hex_detail(app, p);
    let detail_panel = container(scrollable(detail).height(Length::Fill))
        .width(Length::FillPortion(3))
        .height(Length::Fill);

    let body = row![list_panel, detail_panel]
        .spacing(18)
        .height(Length::Fill);

    column![header, controls, widgets::divider(p), body]
        .spacing(16)
        .height(Length::Fill)
        .into()
}

fn custom_hex_list<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
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

fn hex_detail<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    if let Some(ability) = app.selected_hex.as_ref().and_then(|id| app.game.ability(id)) {
        return widgets::detail_panel(
            p,
            ability.name.clone(),
            vec![
                ability.category.label().to_string(),
                ability.ability_type.to_uppercase(),
                ability.source.clone(),
            ],
            ability.description.clone(),
        );
    }
    if !app.character.patron.is_empty() {
        if let Some(patron) = app.game.compendium.abilities.iter().find(|a| {
            a.category == AbilityCategory::Patron
                && a.themes.first().map(|t| t == &app.character.patron).unwrap_or(false)
        }) {
            return widgets::detail_panel(
                p,
                format!("{} Patron", app.character.patron),
                vec!["Patron".to_string(), "Bonus spells".to_string()],
                patron.description.clone(),
            );
        }
    }
    widgets::placeholder(p, "Select a hex, or choose a patron to see its bonus spells.")
}

fn patron_names(app: &App) -> Vec<String> {
    let mut names: Vec<String> = app
        .game
        .compendium
        .abilities
        .iter()
        .filter(|a| a.category == AbilityCategory::Patron)
        .filter_map(|a| a.themes.first().cloned())
        .collect();
    names.sort();
    names.dedup();
    names
}

fn current_patron(app: &App) -> String {
    if app.character.patron.is_empty() {
        "None".to_string()
    } else {
        app.character.patron.clone()
    }
}

pub fn familiar_view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    match app.character.familiar.as_ref() {
        Some(fam) => familiar_sheet(p, fam, &app.image_cache),
        None => column![
            widgets::heading(p, "Familiar"),
            widgets::placeholder(p, "This character has no familiar."),
            widgets::primary_button(p, "Add familiar", Message::FamiliarToggle),
        ]
        .spacing(16)
        .into(),
    }
}

fn familiar_sheet<'a>(p: Palette, fam: &'a Familiar, images: &ImageCache) -> Element<'a, Message> {
    let header = row![
        widgets::heading(p, "Familiar"),
        Space::with_width(Length::Fill),
        widgets::ghost_button(p, "Remove", Message::FamiliarToggle),
    ]
    .align_y(Alignment::Center);

    let left = column![
        widgets::section(p, "Abilities", ability_grid(p, fam)),
        row![
            widgets::section(p, "Armor Class", ac_block(p, fam)),
            widgets::section(p, "Saves", saves_block(p, fam)),
        ].spacing(16),
        widgets::section(p, "Attacks", attacks_block(p, fam)),
        widgets::section(p, "Abilities & Notes", special_block(p, fam)),
    ]
    .spacing(16);

    let right = column![
        widgets::section(p, "Skills", familiar_skills(p, fam)),
        widgets::section(p, "Custom Skills", familiar_custom_skills(p, fam)),
    ]
    .spacing(16);

    scrollable(
        column![
            header,
            identity_panel(p, fam, images),
            row![
                column![left].width(Length::FillPortion(1)),
                column![right].width(Length::FillPortion(1)),
            ]
            .spacing(18),
        ]
        .spacing(16),
    )
    .height(Length::Fill)
    .into()
}

fn identity_panel<'a>(p: Palette, fam: &'a Familiar, images: &ImageCache) -> Element<'a, Message> {
    let handle = fam.portrait.as_deref().and_then(|path| images.handle(path));
    let portrait: Element<Message> = match handle {
        Some(handle) => container(
            image(handle)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
        )
        .style(crate::theme::stat_box(p))
        .into(),
        None => container(text("No image").size(12).color(p.text_dim))
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .center_x(Length::Fixed(120.0))
            .center_y(Length::Fixed(120.0))
            .style(crate::theme::stat_box(p))
            .into(),
    };

    let size_pick = column![
        caption(p, "Size"),
        pick_list(Size::ALL.to_vec(), Some(fam.size), Message::SetFamiliarSize)
            .padding([7, 10])
            .text_size(14)
            .width(Length::Fill)
            .style(crate::theme::dropdown(p)),
    ]
    .spacing(4)
    .width(Length::Fill);

    let fields = column![
        row![
            widgets::labeled_input(p, "Name", &fam.name, |v| Message::SetFamiliar(
                FamiliarField::Name,
                v
            )),
            widgets::labeled_input(p, "Species", &fam.species, |v| Message::SetFamiliar(
                FamiliarField::Species,
                v
            )),
            size_pick,
        ]
        .spacing(12),
        row![
            widgets::number_field(p, "HP", fam.hp_current, 80.0, |v| Message::SetFamiliar(
                FamiliarField::HpCurrent,
                v
            )),
            widgets::number_field(p, "Max HP", fam.hp_max, 80.0, |v| Message::SetFamiliar(
                FamiliarField::HpMax,
                v
            )),
            widgets::labeled_input(p, "Hit Dice", &fam.hit_dice, |v| Message::SetFamiliar(
                FamiliarField::HitDice,
                v
            )),
            widgets::labeled_input(p, "Speed", &fam.speed, |v| Message::SetFamiliar(
                FamiliarField::Speed,
                v
            )),
        ]
        .spacing(12),
    ]
    .spacing(12)
    .width(Length::Fill);

    widgets::section(
        p,
        "Identity",
        row![
            column![portrait, widgets::ghost_button(p, "Portrait", Message::ChangeFamiliarPortrait)]
                .spacing(8)
                .align_x(Alignment::Center),
            fields,
        ]
        .spacing(18),
    )
}

fn ability_grid<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let mut grid = row![].spacing(8);
    for key in ABILITIES {
        let score = fam.ability_score(key);
        let cell = container(
            column![
                text(key.to_uppercase()).size(10).color(p.text_dim),
                text_input("", &score.to_string())
                    .on_input(move |v| Message::SetFamiliarAbility(key.to_string(), v))
                    .padding([4, 6])
                    .size(14)
                    .width(Length::Fixed(52.0))
                    .style(crate::theme::input(p)),
                widgets::mod_badge(p, fam.ability_mod(key)),
            ]
            .spacing(3)
            .align_x(Alignment::Center),
        )
        .padding([8, 4])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(crate::theme::stat_box(p));
        grid = grid.push(cell);
    }
    grid.into()
}

fn ac_block<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let dex = fam.ability_mod("dex");
    let size_mod = fam.size.ac_attack_mod();
    let ac = 10 + dex + fam.natural_armor + fam.deflection + size_mod;
    let touch = 10 + dex + fam.deflection + size_mod;
    let flat = 10 + fam.natural_armor + fam.deflection + size_mod;

    let tiles = row![
        widgets::stat_tile(p, "AC", ac.to_string(), None),
        widgets::stat_tile(p, "Touch", touch.to_string(), None),
        widgets::stat_tile(p, "Flat", flat.to_string(), None),
    ]
    .spacing(10);

    let inputs = row![
        widgets::number_field(p, "Natural Armor", fam.natural_armor, 100.0, |v| {
            Message::SetFamiliar(FamiliarField::NaturalArmor, v)
        }),
        widgets::number_field(p, "Deflection", fam.deflection, 100.0, |v| Message::SetFamiliar(
            FamiliarField::Deflection,
            v
        )),
    ]
    .spacing(12);

    column![tiles, inputs].spacing(12).into()
}

fn saves_block<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let fort = fam.fort_base + fam.ability_mod("con");
    let reflex = fam.ref_base + fam.ability_mod("dex");
    let will = fam.will_base + fam.ability_mod("wis");

    let tiles = row![
        widgets::stat_tile(p, "Fort", signed(fort), None),
        widgets::stat_tile(p, "Ref", signed(reflex), None),
        widgets::stat_tile(p, "Will", signed(will), None),
    ]
    .spacing(10);

    let inputs = row![
        widgets::number_field(p, "Fort Base", fam.fort_base, 90.0, |v| Message::SetFamiliar(
            FamiliarField::FortBase,
            v
        )),
        widgets::number_field(p, "Ref Base", fam.ref_base, 90.0, |v| Message::SetFamiliar(
            FamiliarField::RefBase,
            v
        )),
        widgets::number_field(p, "Will Base", fam.will_base, 90.0, |v| Message::SetFamiliar(
            FamiliarField::WillBase,
            v
        )),
    ]
    .spacing(12);

    column![tiles, inputs].spacing(12).into()
}

fn attacks_block<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let str_mod = fam.ability_mod("str");
    let dex = fam.ability_mod("dex");
    let size_mod = fam.size.ac_attack_mod();
    let melee = fam.bab + str_mod + size_mod;
    let ranged = fam.bab + dex + size_mod;
    let cmd = 10 + fam.bab + str_mod + dex + fam.size.cmb_cmd_mod();

    let tiles = row![
        widgets::stat_tile(p, "Melee", signed(melee), None),
        widgets::stat_tile(p, "Ranged", signed(ranged), None),
        widgets::stat_tile(p, "CMD", cmd.to_string(), None),
    ]
    .spacing(10);

    let inputs = row![
        widgets::number_field(p, "Base Attack", fam.bab, 100.0, |v| Message::SetFamiliar(
            FamiliarField::Bab,
            v
        )),
        widgets::labeled_input(p, "Attack Routine", &fam.attacks, |v| Message::SetFamiliar(
            FamiliarField::Attacks,
            v
        )),
    ]
    .spacing(12);

    let senses = widgets::labeled_input(p, "Senses", &fam.senses, |v| {
        Message::SetFamiliar(FamiliarField::Senses, v)
    });

    column![tiles, inputs, senses].spacing(12).into()
}

fn familiar_skills<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let mut list = column![].spacing(4);
    for def in SKILLS {
        let entry = fam.skills.get(def.id).cloned().unwrap_or_default();
        let total = entry.ranks + fam.ability_mod(def.ability) + entry.misc;
        let ranks = text_input("", &entry.ranks.to_string())
            .on_input(move |v| Message::FamiliarSkillRanks(def.id.to_string(), v))
            .padding([4, 6])
            .size(12)
            .width(Length::Fixed(54.0))
            .style(crate::theme::input(p));
        let misc = text_input("", &entry.misc.to_string())
            .on_input(move |v| Message::FamiliarSkillMisc(def.id.to_string(), v))
            .padding([4, 6])
            .size(12)
            .width(Length::Fixed(54.0))
            .style(crate::theme::input(p));
        let name_color = if entry.ranks > 0 { p.text } else { p.text_dim };
        list = list.push(
            row![
                container(text(def.name).size(12).color(name_color)).width(Length::Fill),
                container(text(def.ability.to_uppercase()).size(11).color(p.text_dim))
                    .width(Length::Fixed(40.0)),
                ranks,
                misc,
                container(text(signed(total)).size(13).color(p.text)).width(Length::Fixed(42.0)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
    }
    scrollable(container(list).padding(iced::Padding::ZERO.right(12.0)))
        .height(Length::Fixed(1000.0))
        .into()
}

fn familiar_custom_skills<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    let mut list = column![].spacing(10);
    if fam.custom_skills.is_empty() {
        list = list.push(caption(p, "No custom skills."));
    }
    for skill in &fam.custom_skills {
        list = list.push(familiar_custom_skill_row(p, fam, skill));
    }
    let add = row![
        Space::with_width(Length::Fill),
        widgets::ghost_button(p, "+ Custom Skill", Message::CustomSkillAdd(SkillScope::Familiar)),
    ];
    column![list, add].spacing(12).into()
}

fn familiar_custom_skill_row<'a>(
    p: Palette,
    fam: &'a Familiar,
    skill: &'a CustomSkill,
) -> Element<'a, Message> {
    let uid = skill.uid;
    let ability_mod = fam.ability_mod(skill.ability_key());
    let class_bonus = if skill.class_skill && skill.ranks > 0 { 3 } else { 0 };
    let total = skill.ranks + ability_mod + skill.misc + class_bonus;

    let name = text_input("Skill name", &skill.name)
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Familiar, uid, CustomSkillField::Name, v))
        .padding([6, 10])
        .size(13)
        .style(crate::theme::input(p));

    let abilities: Vec<String> = ABILITIES.iter().map(|a| a.to_uppercase()).collect();
    let ability_pick = pick_list(abilities, Some(skill.ability_key().to_uppercase()), move |v| {
        Message::CustomSkillSet(SkillScope::Familiar, uid, CustomSkillField::Ability, v.to_lowercase())
    })
    .padding([5, 8])
    .text_size(12)
    .width(Length::Fixed(78.0))
    .style(crate::theme::dropdown(p));

    let ranks = text_input("", &skill.ranks.to_string())
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Familiar, uid, CustomSkillField::Ranks, v))
        .padding([5, 8])
        .size(12)
        .width(Length::Fixed(56.0))
        .style(crate::theme::input(p));

    let misc = text_input("", &skill.misc.to_string())
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Familiar, uid, CustomSkillField::Misc, v))
        .padding([5, 8])
        .size(12)
        .width(Length::Fixed(56.0))
        .style(crate::theme::input(p));

    let class_check = checkbox("Class", skill.class_skill)
        .on_toggle(move |_| Message::CustomSkillToggle(SkillScope::Familiar, uid, CustomSkillFlag::Class))
        .size(16)
        .text_size(12)
        .style(crate::theme::check(p));

    let remove = button(text("Remove").size(11))
        .padding([5, 10])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::CustomSkillRemove(SkillScope::Familiar, uid));

    let fields = row![
        ability_pick,
        ranks,
        misc,
        class_check,
        Space::with_width(Length::Fill),
        text(format!("Total {}", signed(total))).size(13).color(p.text),
        remove,
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(column![name, fields].spacing(8))
        .padding(12)
        .style(crate::theme::plain_row(p))
        .into()
}

fn special_block<'a>(p: Palette, fam: &'a Familiar) -> Element<'a, Message> {
    column![
        text_area_field(p, "Granted Ability", &fam.granted_ability, FamiliarField::Granted),
        text_area_field(p, "Special Qualities", &fam.special, FamiliarField::Special),
        text_area_field(p, "Notes", &fam.notes, FamiliarField::Notes),
        
    ]
    .spacing(12)
    .into()
}

fn text_area_field<'a>(
    p: Palette,
    label: &'a str,
    value: &'a str,
    field: FamiliarField,
) -> Element<'a, Message> {
    column![
        caption(p, label),
        text_input("", value)
            .on_input(move |v| Message::SetFamiliar(field, v))
            .padding([8, 12])
            .style(crate::theme::input(p)),
    ]
    .spacing(4)
    .into()
}
