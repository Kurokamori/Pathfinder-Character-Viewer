//! Skills tab: full skill list with class/trained columns, totals, and custom skills.

use crate::app::{App, CustomSkillField, CustomSkillFlag, Message, SkillScope};
use crate::model::character::{CustomSkill, ABILITIES};
use crate::rules::derived::{self, DerivedStats};
use crate::rules::display::signed;
use crate::rules::skills::SKILLS;
use crate::theme::Palette;
use crate::ui::widgets;
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let d = app.derived();

    column![
        summary(app, p),
        widgets::section(p, "Skills", skill_table(app, p, &d)),
        custom_section(app, p),
        legend(p),
    ]
    .spacing(18)
    .into()
}

fn skill_table<'a>(app: &'a App, p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let ch = &app.character;
    let header = row![
        label_cell(p, "Class", 50.0),
        label_cell(p, "Skill", 208.0),
        label_cell(p, "Abil", 74.0),
        label_cell(p, "Trained", 66.0),
        label_cell(p, "Ranks", 68.0),
        label_cell(p, "Misc", 68.0),
        label_cell(p, "Class+", 54.0),
        label_cell(p, "Total", 56.0),
    ]
    .spacing(10);

    let mut rows = column![header].spacing(6);
    for def in SKILLS {
        let entry = ch.skill(def.id);
        let granted = derived::class_skill_granted(ch, def.id);
        let is_class = granted || entry.class_skill_override;
        let total = derived::skill_total(ch, d, def.id);
        let class_bonus = if is_class && entry.ranks > 0 { 3 } else { 0 };

        let class_toggle = {
            let base = checkbox("", is_class).size(18).style(crate::theme::check(p));
            if granted {
                base
            } else {
                let id = def.id.to_string();
                base.on_toggle(move |_| Message::ToggleClassSkill(id.clone()))
            }
        };

        let ranks_input = text_input("", &entry.ranks.to_string())
            .on_input(move |v| Message::SetSkillRanks(def.id.to_string(), v))
            .padding([5, 8])
            .size(13)
            .width(Length::Fixed(68.0))
            .style(crate::theme::input(p));

        let misc_input = text_input("", &entry.misc.to_string())
            .on_input(move |v| Message::SetSkillMisc(def.id.to_string(), v))
            .padding([5, 8])
            .size(13)
            .width(Length::Fixed(68.0))
            .style(crate::theme::input(p));

        let name_color = if def.trained_only { p.text_dim } else { p.text };
        let bonus_text = if class_bonus > 0 { "+3".to_string() } else { "—".to_string() };
        let trained_text = if def.trained_only { "Yes" } else { "—" };

        let rowel = row![
            container(class_toggle).width(Length::Fixed(50.0)),
            container(text(def.name).size(14).color(name_color)).width(Length::Fixed(208.0)),
            container(
                text(format!("{} {}", def.ability.to_uppercase(), signed(ch.ability_mod(def.ability))))
                    .size(12)
                    .color(p.text_dim),
            )
            .width(Length::Fixed(74.0)),
            container(text(trained_text).size(12).color(p.text_dim)).width(Length::Fixed(66.0)),
            container(ranks_input).width(Length::Fixed(68.0)),
            container(misc_input).width(Length::Fixed(68.0)),
            container(text(bonus_text).size(13).color(p.good)).width(Length::Fixed(54.0)),
            container(text(signed(total)).size(15).color(p.text)).width(Length::Fixed(56.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        rows = rows.push(rowel);
    }
    rows.into()
}

fn custom_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let mut list = column![].spacing(10);
    if app.character.custom_skills.is_empty() {
        list = list.push(widgets::caption(p, "No custom skills. Add homebrew skills below."));
    }
    for skill in &app.character.custom_skills {
        list = list.push(custom_skill_row(app, p, skill));
    }
    let add = row![
        iced::widget::Space::with_width(Length::Fill),
        widgets::ghost_button(p, "+ Custom Skill", Message::CustomSkillAdd(SkillScope::Character)),
    ];
    widgets::section(p, "Custom Skills", column![list, add].spacing(12))
}

fn custom_skill_row<'a>(app: &'a App, p: Palette, skill: &'a CustomSkill) -> Element<'a, Message> {
    let uid = skill.uid;
    let total = derived::custom_skill_total(&app.character, skill);

    let name = text_input("Skill name", &skill.name)
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Character, uid, CustomSkillField::Name, v))
        .padding([6, 10])
        .size(14)
        .style(crate::theme::input(p));

    let abilities: Vec<String> = ABILITIES.iter().map(|a| a.to_uppercase()).collect();
    let ability_pick = pick_list(
        abilities,
        Some(skill.ability_key().to_uppercase()),
        move |v| {
            Message::CustomSkillSet(SkillScope::Character, uid, CustomSkillField::Ability, v.to_lowercase())
        },
    )
    .padding([5, 8])
    .text_size(13)
    .width(Length::Fixed(80.0))
    .style(crate::theme::dropdown(p));

    let ranks = text_input("", &skill.ranks.to_string())
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Character, uid, CustomSkillField::Ranks, v))
        .padding([5, 8])
        .size(13)
        .width(Length::Fixed(64.0))
        .style(crate::theme::input(p));

    let misc = text_input("", &skill.misc.to_string())
        .on_input(move |v| Message::CustomSkillSet(SkillScope::Character, uid, CustomSkillField::Misc, v))
        .padding([5, 8])
        .size(13)
        .width(Length::Fixed(64.0))
        .style(crate::theme::input(p));

    let class_check = checkbox("Class", skill.class_skill)
        .on_toggle(move |_| Message::CustomSkillToggle(SkillScope::Character, uid, CustomSkillFlag::Class))
        .size(16)
        .text_size(12)
        .style(crate::theme::check(p));

    let trained_check = checkbox("Trained", skill.trained_only)
        .on_toggle(move |_| Message::CustomSkillToggle(SkillScope::Character, uid, CustomSkillFlag::Trained))
        .size(16)
        .text_size(12)
        .style(crate::theme::check(p));

    let remove = button(text("Remove").size(12))
        .padding([5, 10])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::CustomSkillRemove(SkillScope::Character, uid));

    let fields = row![
        field(p, "Ability", ability_pick.into()),
        field(p, "Ranks", ranks.into()),
        field(p, "Misc", misc.into()),
        class_check,
        trained_check,
        iced::widget::Space::with_width(Length::Fill),
        container(
            column![text("Total").size(11).color(p.text_dim), text(signed(total)).size(16).color(p.text)]
                .spacing(2)
                .align_x(Alignment::Center)
        ),
        remove,
    ]
    .spacing(12)
    .align_y(Alignment::End);

    container(column![name, fields].spacing(10))
        .padding(12)
        .style(crate::theme::plain_row(p))
        .into()
}

fn field<'a>(p: Palette, label: &'a str, input: Element<'a, Message>) -> Element<'a, Message> {
    column![text(label).size(11).color(p.text_dim), input].spacing(2).into()
}

fn summary<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let ch = &app.character;
    let spent: i32 = SKILLS.iter().map(|s| ch.skill(s.id).ranks).sum::<i32>()
        + ch.custom_skills.iter().map(|s| s.ranks).sum::<i32>();
    let per_level: u32 = ch
        .classes
        .iter()
        .map(|c| {
            let def = crate::rules::progression::class_def(&c.tag);
            let int_mod = ch.ability_mod("int").max(0) as u32;
            (def.skills_per_level + int_mod) * c.level
        })
        .sum();
    let tiles = row![
        widgets::stat_tile(p, "Ranks Spent", spent.to_string(), None),
        widgets::stat_tile(p, "Approx. Available", per_level.to_string(), Some("incl. Int".to_string())),
    ]
    .spacing(12);
    widgets::section(p, "Skill Points", tiles)
}

fn legend(p: Palette) -> Element<'static, Message> {
    widgets::card(
        p,
        text("Check the box to mark a class skill (+3 shown in Class+ once you have at least 1 rank). Dimmed skills are trained-only.")
            .size(12)
            .color(p.text_dim),
    )
}

fn label_cell<'a>(p: Palette, label: &'a str, width: f32) -> Element<'a, Message> {
    container(text(label).size(11).color(p.text_dim))
        .width(Length::Fixed(width))
        .into()
}
