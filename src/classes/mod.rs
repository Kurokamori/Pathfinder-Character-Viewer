//! Class registry. Adding a class means: a progression entry in
//! `rules::progression`, an accent in `theme`, a tab list here, and any
//! custom tab views in a submodule.

pub mod ninja;
pub mod witch;

use crate::data::GameData;
use crate::model::character::{Character, Familiar, ResourcePool};
use crate::rules::progression;
use crate::ui::Tab;

/// The ordered tabs shown for a class.
pub fn tabs_for(tag: &str) -> Vec<Tab> {
    match tag {
        "witch" => vec![
            Tab::General,
            Tab::Combat,
            Tab::CombatRef,
            Tab::Skills,
            Tab::Spells,
            Tab::Hexes,
            Tab::Familiar,
            Tab::Inventory,
            Tab::Shop,
            Tab::Features,
            Tab::Narrative,
            Tab::Gallery,
            Tab::Notes,
        ],
        "ninja" => vec![
            Tab::General,
            Tab::Combat,
            Tab::Skills,
            Tab::Ki,
            Tab::Inventory,
            Tab::Shop,
            Tab::Features,
            Tab::Narrative,
            Tab::Gallery,
            Tab::Notes,
        ],
        _ => vec![
            Tab::General,
            Tab::Combat,
            Tab::Skills,
            Tab::Inventory,
            Tab::Shop,
            Tab::Features,
            Tab::Narrative,
            Tab::Gallery,
            Tab::Notes,
        ],
    }
}

/// Seed class-specific state on a freshly created character.
pub fn init_character(character: &mut Character, tag: &str, game: &GameData) {
    let def = progression::class_def(tag);
    character.hp_rolled = def.hit_die as i32;
    character.hp_current = def.hit_die as i32;

    if def.has_familiar && character.familiar.is_none() {
        character.familiar = Some(Familiar::default());
    }

    sync(character, game);
}

/// Recompute derived class resources after level/ability changes.
pub fn sync(character: &mut Character, _game: &GameData) {
    let ninja_level = character.class_level("ninja");
    if ninja_level >= 2 {
        let cha_mod = character.ability_mod("cha");
        let max = (ninja_level as i32 / 2 + cha_mod).max(0);
        let pool = character
            .resources
            .entry("ki".to_string())
            .or_insert(ResourcePool { current: max, max });
        pool.max = max;
        if pool.current > max {
            pool.current = max;
        }
    } else {
        character.resources.remove("ki");
    }
}

/// Number of hexes a witch of the given level may select.
pub fn hex_allowance(witch_level: u32) -> u32 {
    if witch_level == 0 {
        0
    } else {
        1 + witch_level / 2
    }
}

/// Number of ninja tricks a ninja of the given level may select.
pub fn trick_allowance(ninja_level: u32) -> u32 {
    ninja_level / 2
}

/// Ninja sneak-attack dice at a given level.
pub fn sneak_attack_dice(ninja_level: u32) -> u32 {
    ninja_level / 2
}
