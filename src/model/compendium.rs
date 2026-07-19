//! Consolidated game-data schema shared by the data pipeline and the app.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A spell entry usable by one or more classes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub school: String,
    pub subschool: String,
    pub descriptors: Vec<String>,
    pub verbal: bool,
    pub somatic: bool,
    pub material: bool,
    pub focus: bool,
    pub divine_focus: bool,
    /// Class tag -> minimum spell level for that class.
    pub class_levels: BTreeMap<String, u8>,
    pub casting_time: String,
    pub range: String,
    pub target: String,
    pub duration: String,
    pub save: String,
    pub spell_resistance: String,
    pub description: String,
    pub source: String,
}

impl Spell {
    /// Spell level for a given class tag, if the class can learn it.
    pub fn level_for(&self, class_tag: &str) -> Option<u8> {
        self.class_levels.get(class_tag).copied()
    }
}

/// Category of a class ability used for tab grouping and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AbilityCategory {
    Hex,
    MajorHex,
    GrandHex,
    Patron,
    NinjaTrick,
    MasterTrick,
    Other,
}

impl AbilityCategory {
    pub fn label(self) -> &'static str {
        match self {
            AbilityCategory::Hex => "Hex",
            AbilityCategory::MajorHex => "Major Hex",
            AbilityCategory::GrandHex => "Grand Hex",
            AbilityCategory::Patron => "Patron",
            AbilityCategory::NinjaTrick => "Ninja Trick",
            AbilityCategory::MasterTrick => "Master Trick",
            AbilityCategory::Other => "Feature",
        }
    }
}

/// A class ability: hex, ninja trick, patron, or generic class feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub category: AbilityCategory,
    /// Class tags this ability belongs to (lowercase, e.g. "witch").
    pub classes: Vec<String>,
    /// "su", "ex", "sp", or "".
    pub ability_type: String,
    /// Optional theme/patron tags for filtering (e.g. patron themes).
    pub themes: Vec<String>,
    pub description: String,
    pub source: String,
}

/// A feat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feat {
    pub id: String,
    pub name: String,
    pub types: Vec<String>,
    pub description: String,
    pub source: String,
}

/// A playable race.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub id: String,
    pub name: String,
    pub size: String,
    /// Ability key -> modifier (e.g. "int" -> 2).
    pub ability_mods: BTreeMap<String, i32>,
    pub description: String,
    pub source: String,
}

/// Broad category for shop/inventory browsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemKind {
    Weapon,
    Armor,
    Shield,
    Gear,
    Consumable,
    Magic,
}

impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            ItemKind::Weapon => "Weapons",
            ItemKind::Armor => "Armor",
            ItemKind::Shield => "Shields",
            ItemKind::Gear => "Gear",
            ItemKind::Consumable => "Consumables",
            ItemKind::Magic => "Magic Items",
        }
    }

    pub const ALL: [ItemKind; 6] = [
        ItemKind::Weapon,
        ItemKind::Armor,
        ItemKind::Shield,
        ItemKind::Gear,
        ItemKind::Consumable,
        ItemKind::Magic,
    ];
}

/// Combat statistics for a weapon, parsed from the weapon's attack action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStats {
    /// Base damage dice at Medium size, e.g. "1d8". Empty if unknown.
    pub damage: String,
    /// Low end of the critical threat range (e.g. 19 for 19-20). 20 means natural 20 only.
    pub crit_range: i32,
    /// Critical damage multiplier (2 for x2, 3 for x3).
    pub crit_mult: i32,
    /// Ability key that governs the attack roll ("str" or "dex").
    pub attack_ability: String,
    /// Ability key added to damage ("str"), or empty for none.
    pub damage_ability: String,
    /// True for a melee weapon, false for a ranged weapon.
    pub melee: bool,
    /// Formatted range increment for ranged weapons (e.g. "100 ft."), empty for melee.
    pub range: String,
    /// Damage types dealt (e.g. ["slashing"]).
    pub damage_types: Vec<String>,
}

impl WeaponStats {
    /// Threat range and multiplier as a single label, e.g. "19-20/x2".
    pub fn crit_label(&self) -> String {
        let range = if self.crit_range >= 20 {
            "20".to_string()
        } else {
            format!("{}-20", self.crit_range)
        };
        format!("{range}/x{}", self.crit_mult)
    }
}

/// A purchasable / equippable item template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub subtype: String,
    /// Price in gold pieces.
    pub price: f64,
    /// Weight in pounds.
    pub weight: f64,
    /// Armor/shield AC bonus.
    pub ac_bonus: i32,
    pub max_dex: Option<i32>,
    pub armor_check_penalty: i32,
    pub description: String,
    pub source: String,
    /// Weapon combat statistics, present only for weapons.
    #[serde(default)]
    pub weapon: Option<WeaponStats>,
}

/// A class description entry from the compendium (mechanics live in `rules::progression`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEntry {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// The whole loaded game database.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Compendium {
    pub spells: Vec<Spell>,
    pub abilities: Vec<Ability>,
    pub feats: Vec<Feat>,
    pub races: Vec<Race>,
    pub items: Vec<Item>,
    pub classes: Vec<ClassEntry>,
}

impl Compendium {
    /// Distinct, sorted source-book names present in the data (excluding unknown).
    pub fn sources(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for s in &self.spells {
            if !s.source.is_empty() {
                set.insert(s.source.clone());
            }
        }
        for a in &self.abilities {
            if !a.source.is_empty() {
                set.insert(a.source.clone());
            }
        }
        for f in &self.feats {
            if !f.source.is_empty() {
                set.insert(f.source.clone());
            }
        }
        for i in &self.items {
            if !i.source.is_empty() {
                set.insert(i.source.clone());
            }
        }
        set.into_iter().collect()
    }
}
