//! Computes all derived statistics from a character's raw values.

use super::progression::{self, Casting};
use super::skills::skill_def;
use crate::model::character::Character;

/// The full set of computed combat/defense statistics.
#[derive(Debug, Clone)]
pub struct DerivedStats {
    pub bab: i32,
    pub max_hp: i32,
    pub initiative: i32,
    pub fort: i32,
    pub reflex: i32,
    pub will: i32,
    pub ac: i32,
    pub touch: i32,
    pub flat_footed: i32,
    pub armor_bonus: i32,
    pub shield_bonus: i32,
    pub max_dex: Option<i32>,
    pub armor_check_penalty: i32,
    pub cmb: i32,
    pub cmd: i32,
    pub melee_attack: i32,
    pub ranged_attack: i32,
    pub size_mod: i32,
    pub carry_light: i32,
    pub carry_medium: i32,
    pub carry_heavy: i32,
}

fn equipped_defense(character: &Character) -> (i32, i32, Option<i32>, i32) {
    use crate::model::compendium::ItemKind;
    let mut armor = 0;
    let mut shield = 0;
    let mut max_dex: Option<i32> = None;
    let mut acp = 0;
    for item in character.inventory.iter().filter(|i| i.equipped) {
        match item.kind {
            ItemKind::Armor => {
                armor += item.ac_bonus;
                acp += item.armor_check_penalty;
                if let Some(cap) = item.max_dex {
                    max_dex = Some(max_dex.map_or(cap, |m| m.min(cap)));
                }
            }
            ItemKind::Shield => {
                shield += item.ac_bonus;
                acp += item.armor_check_penalty;
                if let Some(cap) = item.max_dex {
                    max_dex = Some(max_dex.map_or(cap, |m| m.min(cap)));
                }
            }
            _ => {}
        }
    }
    (armor, shield, max_dex, acp)
}

fn carrying_capacity(str_score: i32) -> (i32, i32, i32) {
    const HEAVY: [i32; 30] = [
        10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 115, 130, 150, 175, 200, 230, 260, 300, 350, 400,
        460, 520, 600, 700, 800, 920, 1040, 1200, 1400, 1600,
    ];
    let clamped = str_score.clamp(1, 29) as usize;
    let heavy = if str_score <= 0 {
        0
    } else if str_score <= 30 {
        HEAVY[clamped - 1]
    } else {
        let over = ((str_score - 20) / 10) as u32;
        HEAVY[(((str_score - 1) % 10) + 19) as usize] * 4i32.pow(over.saturating_sub(1))
    };
    (heavy / 3, heavy * 2 / 3, heavy)
}

pub fn compute(character: &Character) -> DerivedStats {
    let str_mod = character.ability_mod("str");
    let dex_mod = character.ability_mod("dex");
    let con_mod = character.ability_mod("con");
    let wis_mod = character.ability_mod("wis");
    let b = &character.bonuses;
    let size = character.size;
    let size_mod = size.ac_attack_mod();

    let mut bab = b.bab_misc;
    let mut fort_base = 0;
    let mut ref_base = 0;
    let mut will_base = 0;
    for class in &character.classes {
        let def = progression::class_def(&class.tag);
        bab += def.bab.at(class.level);
        fort_base += def.fort.at(class.level);
        ref_base += def.reflex.at(class.level);
        will_base += def.will.at(class.level);
    }

    let max_hp = character.hp_rolled + con_mod * character.total_level() as i32 + b.hp_misc;

    let (armor_bonus, shield_bonus, max_dex, armor_check_penalty) = equipped_defense(character);
    let effective_dex = match max_dex {
        Some(cap) => dex_mod.min(cap),
        None => dex_mod,
    };

    let ac = 10 + armor_bonus + shield_bonus + effective_dex + size_mod
        + b.natural_armor + b.deflection + b.dodge + b.misc_ac;
    let touch = 10 + effective_dex + size_mod + b.deflection + b.dodge + b.misc_ac;
    let flat_footed = 10 + armor_bonus + shield_bonus + size_mod
        + b.natural_armor + b.deflection + b.misc_ac;

    let cmb = bab + str_mod + size.cmb_cmd_mod() + b.cmb_misc;
    let cmd = 10 + bab + str_mod + dex_mod + size.cmb_cmd_mod() + b.deflection + b.dodge + b.cmd_misc;

    let melee_attack = bab + str_mod + size_mod + b.attack_misc;
    let ranged_attack = bab + dex_mod + size_mod + b.attack_misc;

    let (carry_light, carry_medium, carry_heavy) = carrying_capacity(character.ability("str").total());

    DerivedStats {
        bab,
        max_hp,
        initiative: dex_mod + b.init_misc,
        fort: fort_base + con_mod + b.fort_misc,
        reflex: ref_base + dex_mod + b.ref_misc,
        will: will_base + wis_mod + b.will_misc,
        ac,
        touch,
        flat_footed,
        armor_bonus,
        shield_bonus,
        max_dex,
        armor_check_penalty,
        cmb,
        cmd,
        melee_attack,
        ranged_attack,
        size_mod,
        carry_light,
        carry_medium,
        carry_heavy,
    }
}

/// Whether a skill is a class skill for the character (any of its classes, or an override).
pub fn is_class_skill(character: &Character, skill_id: &str) -> bool {
    if character.skill(skill_id).class_skill_override {
        return true;
    }
    character.classes.iter().any(|class| {
        progression::class_def(&class.tag)
            .class_skills
            .contains(&skill_id)
    })
}

/// Total modifier for a skill, including ranks, ability, class bonus, and armor penalty.
pub fn skill_total(character: &Character, derived: &DerivedStats, skill_id: &str) -> i32 {
    let def = match skill_def(skill_id) {
        Some(d) => d,
        None => return 0,
    };
    let entry = character.skill(skill_id);
    let ability = character.ability_mod(def.ability);
    let class_bonus = if is_class_skill(character, skill_id) && entry.ranks > 0 {
        3
    } else {
        0
    };
    let penalty = if def.armor_check {
        derived.armor_check_penalty
    } else {
        0
    };
    entry.ranks + ability + class_bonus + entry.misc + penalty
}

/// The character's primary spellcasting class definition, if any.
pub fn casting_class(character: &Character) -> Option<(&crate::model::character::ClassLevel, progression::ClassDef)> {
    character.classes.iter().find_map(|class| {
        let def = progression::class_def(&class.tag);
        if def.casting != Casting::None {
            Some((class, def))
        } else {
            None
        }
    })
}

/// Save DC for a spell of the given level cast by the character.
pub fn spell_save_dc(character: &Character, spell_level: usize) -> Option<i32> {
    let (_, def) = casting_class(character)?;
    let ability_mod = character.ability_mod(def.casting_ability);
    Some(10 + spell_level as i32 + ability_mod)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::Character;

    #[test]
    fn witch_class_skill_bonus() {
        let mut ch = Character::new(1, "Test", "witch");
        ch.ability_mut("int").base = 14;
        ch.skill_mut("spl").ranks = 1;
        let d = compute(&ch);
        assert!(is_class_skill(&ch, "spl"), "spellcraft should be a witch class skill");
        let total = skill_total(&ch, &d, "spl");
        assert_eq!(total, 1 + 2 + 3, "1 rank + 2 int + 3 class = 6, got {total}");
    }
}
