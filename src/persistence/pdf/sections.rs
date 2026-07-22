//! Per-tab section renderers for the PDF character sheet.
//!
//! Each function mirrors one tab of the viewer. Sections whose underlying data
//! is empty render nothing at all, so a low-level character does not produce
//! pages of blank headings.

use super::layout::{Cell, Pdf};
use crate::data::GameData;
use crate::model::character::{ability_name, Character, ABILITIES};
use crate::rules::derived::{self, DerivedStats};
use crate::rules::display::{components_string, school_name, signed, spell_level_label};
use crate::rules::progression::{self, Casting};
use crate::rules::skills::SKILLS;

/// Human-readable "Witch 5 / Rogue 2" summary of the class line.
pub fn class_line(ch: &Character) -> String {
    if ch.classes.is_empty() {
        return String::new();
    }
    ch.classes
        .iter()
        .map(|c| {
            let def = progression::class_def(&c.tag);
            format!("{} {}", def.name, c.level)
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

pub fn identity(pdf: &mut Pdf, ch: &Character) {
    let mut subtitle_parts: Vec<String> = Vec::new();
    let classes = class_line(ch);
    if !classes.is_empty() {
        subtitle_parts.push(classes);
    }
    if !ch.race.is_empty() {
        subtitle_parts.push(ch.race.clone());
    }
    subtitle_parts.push(format!("Level {}", ch.total_level()));

    let name = if ch.name.trim().is_empty() {
        "Unnamed Character"
    } else {
        ch.name.trim()
    };
    pdf.title(name, &subtitle_parts.join("  \u{00B7}  "));

    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut push = |label: &str, value: &str| {
        if !value.trim().is_empty() {
            pairs.push((label.to_string(), value.trim().to_string()));
        }
    };
    push("Player", &ch.player);
    push("Alignment", &ch.alignment);
    push("Deity", &ch.deity);
    push("Size", ch.size.label());
    push("Gender", &ch.gender);
    push("Age", &ch.age);
    push("Height", &ch.height);
    push("Weight", &ch.weight_desc);
    if !ch.patron.trim().is_empty() {
        push("Patron", &ch.patron);
    }

    if !pairs.is_empty() {
        pdf.grid(&pairs, 3);
    }
}

pub fn abilities(pdf: &mut Pdf, ch: &Character) {
    pdf.section("Ability Scores");
    pdf.header_row(&[
        Cell::new(0.0, "Ability"),
        Cell::new(46.0, "Score"),
        Cell::new(66.0, "Mod"),
        Cell::new(84.0, "Base"),
        Cell::new(104.0, "Racial"),
        Cell::new(126.0, "Enhance"),
        Cell::new(152.0, "Temp"),
    ]);
    for key in ABILITIES {
        let score = ch.ability(key);
        pdf.row(
            &[
                Cell::bold(0.0, ability_name(key)),
                Cell::new(46.0, score.total().to_string()),
                Cell::bold(66.0, signed(score.modifier())),
                Cell::new(84.0, score.base.to_string()),
                Cell::new(104.0, signed(score.racial)),
                Cell::new(126.0, signed(score.enhancement)),
                Cell::new(152.0, signed(score.temp)),
            ],
            9.0,
        );
    }
}

pub fn combat(pdf: &mut Pdf, ch: &Character, d: &DerivedStats) {
    pdf.section("Combat");

    let hp_line = if ch.hp_temp > 0 {
        format!("{} / {} (+{} temp)", ch.hp_current, d.max_hp, ch.hp_temp)
    } else {
        format!("{} / {}", ch.hp_current, d.max_hp)
    };

    let mut vitals: Vec<(String, String)> = vec![
        ("HP".to_string(), hp_line),
        ("AC".to_string(), d.ac.to_string()),
        ("Touch".to_string(), d.touch.to_string()),
        ("Flat-Footed".to_string(), d.flat_footed.to_string()),
        ("Init".to_string(), signed(d.initiative)),
        ("BAB".to_string(), signed(d.bab)),
    ];
    if ch.nonlethal > 0 {
        vitals.push(("Nonlethal".to_string(), ch.nonlethal.to_string()));
    }
    pdf.grid(&vitals, 3);

    pdf.grid(
        &[
            ("Fort".to_string(), signed(d.fort)),
            ("Ref".to_string(), signed(d.reflex)),
            ("Will".to_string(), signed(d.will)),
            ("Melee".to_string(), signed(d.melee_attack)),
            ("Ranged".to_string(), signed(d.ranged_attack)),
            ("Speed".to_string(), format!("{} ft.", ch.base_speed)),
            ("CMB".to_string(), signed(d.cmb)),
            ("CMD".to_string(), d.cmd.to_string()),
            ("ACP".to_string(), d.armor_check_penalty.to_string()),
        ],
        3,
    );

    let max_dex = match d.max_dex {
        Some(value) => signed(value),
        None => "-".to_string(),
    };
    pdf.grid(
        &[
            ("Armor".to_string(), signed(d.armor_bonus)),
            ("Shield".to_string(), signed(d.shield_bonus)),
            ("Max Dex".to_string(), max_dex),
        ],
        3,
    );

    pdf.caption(&format!(
        "Carrying capacity - light {} lb., medium {} lb., heavy {} lb.",
        d.carry_light, d.carry_medium, d.carry_heavy
    ));

    if !ch.conditions.is_empty() {
        pdf.subheading("Active Conditions");
        pdf.paragraph(&ch.conditions.join(", "), 9.0, 3.0);
    }
}

pub fn skills(pdf: &mut Pdf, ch: &Character, d: &DerivedStats) {
    pdf.section("Skills");
    pdf.header_row(&[
        Cell::new(0.0, "Skill"),
        Cell::new(62.0, "Ability"),
        Cell::new(88.0, "Total"),
        Cell::new(108.0, "Ranks"),
        Cell::new(128.0, "Misc"),
        Cell::new(148.0, "Class"),
    ]);

    for def in SKILLS {
        let entry = ch.skill(def.id);
        let is_class = derived::is_class_skill(ch, def.id);
        let total = derived::skill_total(ch, d, def.id);
        if entry.ranks == 0 && entry.misc == 0 && def.trained_only {
            continue;
        }
        pdf.row(
            &[
                Cell::new(0.0, def.name),
                Cell::new(
                    62.0,
                    format!(
                        "{} {}",
                        def.ability.to_uppercase(),
                        signed(ch.ability_mod(def.ability))
                    ),
                ),
                Cell::bold(88.0, signed(total)),
                Cell::new(108.0, entry.ranks.to_string()),
                Cell::new(128.0, signed(entry.misc)),
                Cell::new(148.0, if is_class { "yes" } else { "" }),
            ],
            8.5,
        );
    }

    if !ch.custom_skills.is_empty() {
        pdf.subheading("Custom Skills");
        for skill in &ch.custom_skills {
            let total = derived::custom_skill_total(ch, skill);
            pdf.row(
                &[
                    Cell::new(0.0, skill.name.clone()),
                    Cell::new(
                        62.0,
                        format!(
                            "{} {}",
                            skill.ability_key().to_uppercase(),
                            signed(ch.ability_mod(skill.ability_key()))
                        ),
                    ),
                    Cell::bold(88.0, signed(total)),
                    Cell::new(108.0, skill.ranks.to_string()),
                    Cell::new(128.0, signed(skill.misc)),
                    Cell::new(148.0, if skill.class_skill { "yes" } else { "" }),
                ],
                8.5,
            );
        }
    }

    let spent: i32 = SKILLS.iter().map(|s| ch.skill(s.id).ranks).sum::<i32>()
        + ch.custom_skills.iter().map(|s| s.ranks).sum::<i32>();
    pdf.caption(&format!(
        "Trained-only skills with no ranks are omitted. {spent} ranks spent."
    ));
}

pub fn feats(pdf: &mut Pdf, ch: &Character, game: &GameData) {
    if ch.feats.is_empty() && ch.custom_feats.is_empty() {
        return;
    }
    pdf.section("Feats");

    for id in &ch.feats {
        if let Some(feat) = game.feat(id) {
            let types = feat.types.join(", ");
            if types.is_empty() {
                pdf.bullet(&feat.name);
            } else {
                pdf.bullet(&format!("{} ({})", feat.name, types));
            }
        }
    }

    for entry in &ch.custom_feats {
        pdf.bullet(&format!("{} (homebrew)", entry.name));
        if !entry.description.trim().is_empty() {
            pdf.paragraph(&entry.description, 8.5, 4.0);
        }
    }
}

pub fn racial_traits(pdf: &mut Pdf, ch: &Character) {
    if ch.racial_traits.is_empty() {
        return;
    }
    pdf.section("Racial Traits");
    for entry in &ch.racial_traits {
        pdf.subheading(&entry.name);
        if !entry.description.trim().is_empty() {
            pdf.paragraph(&entry.description, 8.5, 3.0);
        }
    }
}

pub fn languages(pdf: &mut Pdf, ch: &Character) {
    if ch.languages.trim().is_empty() {
        return;
    }
    pdf.section("Languages");
    pdf.paragraph(ch.languages.trim(), 9.0, 0.0);
}

pub fn class_abilities(pdf: &mut Pdf, ch: &Character, game: &GameData) {
    if ch.selected_abilities.is_empty() && ch.custom_abilities.is_empty() {
        return;
    }
    pdf.section("Class Abilities");

    for id in &ch.selected_abilities {
        if let Some(ability) = game.ability(id) {
            let mut tag = ability.category.label().to_string();
            if !ability.ability_type.is_empty() {
                tag = format!("{} \u{00B7} {}", tag, ability.ability_type.to_uppercase());
            }
            pdf.subheading(&format!("{} ({})", ability.name, tag));
            if !ability.description.trim().is_empty() {
                pdf.paragraph(&ability.description, 8.5, 3.0);
            }
        }
    }

    for entry in &ch.custom_abilities {
        pdf.subheading(&format!("{} (homebrew)", entry.name));
        if !entry.description.trim().is_empty() {
            pdf.paragraph(&entry.description, 8.5, 3.0);
        }
    }
}

pub fn spells(pdf: &mut Pdf, ch: &Character, game: &GameData) {
    let casting = derived::casting_class(ch);
    let (class, def) = match casting {
        Some(pair) => pair,
        None => return,
    };
    if matches!(def.casting, Casting::None) {
        return;
    }
    if ch.spellbook.learned.is_empty() && ch.spellbook.prepared.is_empty() {
        return;
    }

    pdf.section("Spells");
    let ability_mod = ch.ability_mod(def.casting_ability);
    pdf.caption(&format!(
        "{} casting - {} \u{00B7} key ability {} ({})",
        def.name,
        match def.casting {
            Casting::Prepared => "prepared",
            Casting::Spontaneous => "spontaneous",
            Casting::None => "none",
        },
        def.casting_ability.to_uppercase(),
        signed(ability_mod)
    ));

    let max_level = def.max_spell_level(class.level).unwrap_or(0);
    pdf.subheading("Spells per Day");
    pdf.header_row(&[
        Cell::new(0.0, "Level"),
        Cell::new(40.0, "Base"),
        Cell::new(62.0, "Bonus"),
        Cell::new(86.0, "Total"),
        Cell::new(110.0, "Save DC"),
    ]);
    for lvl in 0..=max_level {
        let base = def.base_spells_per_day(class.level, lvl).unwrap_or(0);
        let bonus = def.bonus_spells(lvl, ability_mod);
        pdf.row(
            &[
                Cell::new(0.0, spell_level_label(lvl)),
                Cell::new(40.0, base.to_string()),
                Cell::new(62.0, bonus.to_string()),
                Cell::bold(86.0, (base + bonus).to_string()),
                Cell::new(110.0, (10 + lvl as i32 + ability_mod).to_string()),
            ],
            8.5,
        );
    }

    if !ch.spellbook.prepared.is_empty() {
        pdf.subheading("Prepared");
        let mut prepared: Vec<_> = ch.spellbook.prepared.iter().collect();
        prepared.sort_by_key(|pr| {
            game.spell(&pr.spell_id)
                .and_then(|s| s.level_for(&class.tag))
                .unwrap_or(0)
        });
        for pr in prepared {
            if let Some(spell) = game.spell(&pr.spell_id) {
                let level = spell.level_for(&class.tag).unwrap_or(0);
                let mark = if pr.used { " [used]" } else { "" };
                pdf.bullet(&format!(
                    "{} - {}{}",
                    spell_level_label(level as usize),
                    spell.name,
                    mark
                ));
            }
        }
    }

    if !ch.spellbook.learned.is_empty() {
        pdf.subheading("Known / Spellbook");
        let mut by_level: std::collections::BTreeMap<u8, Vec<&str>> =
            std::collections::BTreeMap::new();
        for id in &ch.spellbook.learned {
            if let Some(spell) = game.spell(id) {
                let level = spell.level_for(&class.tag).unwrap_or(0);
                by_level.entry(level).or_default().push(&spell.name);
            }
        }
        for (level, mut names) in by_level {
            names.sort_unstable();
            pdf.ensure(10.0);
            pdf.row(
                &[Cell::bold(0.0, spell_level_label(level as usize))],
                9.0,
            );
            pdf.paragraph(&names.join(", "), 8.5, 4.0);
        }
    }

    spell_details(pdf, ch, game, &class.tag);
}

/// Full stat blocks for prepared spells, so the sheet is playable on its own.
fn spell_details(pdf: &mut Pdf, ch: &Character, game: &GameData, class_tag: &str) {
    let mut ids: Vec<&String> = ch.spellbook.prepared.iter().map(|p| &p.spell_id).collect();
    if ids.is_empty() {
        ids = ch.spellbook.learned.iter().collect();
    }
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return;
    }

    pdf.section("Spell Reference");
    for id in ids {
        let spell = match game.spell(id) {
            Some(spell) => spell,
            None => continue,
        };
        let level = spell.level_for(class_tag).unwrap_or(0);
        pdf.ensure(28.0);
        pdf.subheading(&format!(
            "{} ({}, {})",
            spell.name,
            spell_level_label(level as usize),
            school_name(&spell.school)
        ));

        let components = components_string(
            spell.verbal,
            spell.somatic,
            spell.material,
            spell.focus,
            spell.divine_focus,
        );
        let mut facts: Vec<(String, String)> = Vec::new();
        let mut push = |label: &str, value: &str| {
            if !value.trim().is_empty() {
                facts.push((label.to_string(), value.trim().to_string()));
            }
        };
        push("Casting", &spell.casting_time);
        push("Range", &spell.range);
        push("Target", &spell.target);
        push("Duration", &spell.duration);
        push("Save", &spell.save);
        push("SR", &spell.spell_resistance);
        push("Components", &components);
        pdf.grid(&facts, 2);

        if !spell.description.trim().is_empty() {
            pdf.paragraph(&spell.description, 8.5, 3.0);
        }
    }
}

pub fn familiar(pdf: &mut Pdf, ch: &Character) {
    let familiar = match ch.familiar.as_ref() {
        Some(familiar) => familiar,
        None => return,
    };

    pdf.section("Familiar");
    let title = if familiar.name.trim().is_empty() {
        familiar.species.clone()
    } else {
        format!("{} ({})", familiar.name, familiar.species)
    };
    if !title.trim().is_empty() {
        pdf.subheading(title.trim());
    }

    let ac = 10 + familiar.ability_mod("dex") + familiar.natural_armor + familiar.deflection;
    pdf.grid(
        &[
            (
                "HP".to_string(),
                format!("{} / {}", familiar.hp_current, familiar.hp_max),
            ),
            ("AC".to_string(), ac.to_string()),
            ("Size".to_string(), familiar.size.label().to_string()),
            ("BAB".to_string(), signed(familiar.bab)),
            ("HD".to_string(), familiar.hit_dice.clone()),
            ("Speed".to_string(), familiar.speed.clone()),
            ("Fort".to_string(), signed(familiar.fort_base)),
            ("Ref".to_string(), signed(familiar.ref_base)),
            ("Will".to_string(), signed(familiar.will_base)),
        ],
        3,
    );

    let scores: Vec<(String, String)> = ABILITIES
        .iter()
        .map(|key| {
            (
                key.to_uppercase(),
                format!(
                    "{} ({})",
                    familiar.ability_score(key),
                    signed(familiar.ability_mod(key))
                ),
            )
        })
        .collect();
    pdf.grid(&scores, 3);

    pdf.labelled_block("Senses", &familiar.senses);
    pdf.labelled_block("Attacks", &familiar.attacks);
    pdf.labelled_block("Granted Ability", &familiar.granted_ability);
    pdf.labelled_block("Special", &familiar.special);
    pdf.labelled_block("Notes", &familiar.notes);
}

pub fn inventory(pdf: &mut Pdf, ch: &Character, d: &DerivedStats) {
    pdf.section("Inventory");

    let coins = &ch.coins;
    pdf.grid(
        &[
            ("PP".to_string(), coins.pp.to_string()),
            ("GP".to_string(), coins.gp.to_string()),
            ("SP".to_string(), coins.sp.to_string()),
            ("CP".to_string(), coins.cp.to_string()),
        ],
        4,
    );

    let (pp, gp, sp, cp) = coins.normalized();
    pdf.caption(&format!(
        "Total wealth: {pp} pp / {gp} gp / {sp} sp / {cp} cp."
    ));

    if ch.inventory.is_empty() {
        pdf.caption("No items carried.");
        return;
    }

    pdf.header_row(&[
        Cell::new(0.0, "Item"),
        Cell::new(88.0, "Qty"),
        Cell::new(104.0, "Wt"),
        Cell::new(124.0, "Price"),
        Cell::new(150.0, "Equipped"),
    ]);

    let mut total_weight: f64 = 0.0;
    for item in &ch.inventory {
        total_weight += item.weight * item.quantity as f64;
        let slot = item
            .slot
            .map(|s| format!(" [{}]", s.label()))
            .unwrap_or_default();
        pdf.row(
            &[
                Cell::new(0.0, format!("{}{}", item.name, slot)),
                Cell::new(88.0, item.quantity.to_string()),
                Cell::new(104.0, format!("{:.1}", item.weight)),
                Cell::new(124.0, format!("{:.2}", item.price)),
                Cell::new(150.0, if item.equipped { "yes" } else { "" }),
            ],
            8.5,
        );
        if !item.notes.trim().is_empty() {
            pdf.paragraph(&item.notes, 8.0, 4.0);
        }
    }

    let load = if total_weight <= d.carry_light as f64 {
        "light"
    } else if total_weight <= d.carry_medium as f64 {
        "medium"
    } else if total_weight <= d.carry_heavy as f64 {
        "heavy"
    } else {
        "overloaded"
    };
    pdf.caption(&format!(
        "Total carried {total_weight:.1} lb. - {load} load."
    ));
}

pub fn resources(pdf: &mut Pdf, ch: &Character) {
    if ch.resources.is_empty() {
        return;
    }
    pdf.section("Resource Pools");
    let pairs: Vec<(String, String)> = ch
        .resources
        .iter()
        .map(|(name, pool)| {
            (
                title_case(name),
                format!("{} / {}", pool.current, pool.max),
            )
        })
        .collect();
    pdf.grid(&pairs, 3);
}

pub fn narrative(pdf: &mut Pdf, ch: &Character) {
    let blocks: [(&str, &String); 7] = [
        ("Origins", &ch.origins),
        ("Appearance", &ch.appearance),
        ("Personality", &ch.personality),
        ("Backstory", &ch.backstory),
        ("Affiliations", &ch.affiliation),
        ("Friends", &ch.friends),
        ("Foes", &ch.foes),
    ];
    if blocks.iter().all(|(_, body)| body.trim().is_empty()) {
        return;
    }

    pdf.section("Narrative");
    for (label, body) in blocks {
        pdf.labelled_block(label, body);
    }
}

pub fn notes(pdf: &mut Pdf, ch: &Character) {
    if ch.notes.trim().is_empty() {
        return;
    }
    pdf.section("Notes");
    pdf.paragraph(ch.notes.trim(), 9.0, 0.0);
}

/// Capitalise a resource key like "ki" for display.
fn title_case(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
