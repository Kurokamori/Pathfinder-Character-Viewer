//! The persistent character document. Self-contained so saves stay portable.

use super::compendium::ItemKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The six ability scores.
pub const ABILITIES: [&str; 6] = ["str", "dex", "con", "int", "wis", "cha"];

pub fn ability_name(key: &str) -> &'static str {
    match key {
        "str" => "Strength",
        "dex" => "Dexterity",
        "con" => "Constitution",
        "int" => "Intelligence",
        "wis" => "Wisdom",
        "cha" => "Charisma",
        _ => "Ability",
    }
}

/// A score assembled from independent, individually editable sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScore {
    pub base: i32,
    pub racial: i32,
    pub enhancement: i32,
    pub temp: i32,
}

impl Default for AbilityScore {
    fn default() -> Self {
        Self {
            base: 10,
            racial: 0,
            enhancement: 0,
            temp: 0,
        }
    }
}

impl AbilityScore {
    pub fn total(&self) -> i32 {
        self.base + self.racial + self.enhancement + self.temp
    }

    pub fn modifier(&self) -> i32 {
        (self.total() - 10).div_euclid(2)
    }
}

/// Creature size categories, ordered small-to-large.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Size {
    Fine,
    Diminutive,
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
    Colossal,
}

impl Size {
    pub const ALL: [Size; 9] = [
        Size::Fine,
        Size::Diminutive,
        Size::Tiny,
        Size::Small,
        Size::Medium,
        Size::Large,
        Size::Huge,
        Size::Gargantuan,
        Size::Colossal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Size::Fine => "Fine",
            Size::Diminutive => "Diminutive",
            Size::Tiny => "Tiny",
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Large => "Large",
            Size::Huge => "Huge",
            Size::Gargantuan => "Gargantuan",
            Size::Colossal => "Colossal",
        }
    }

    /// Size modifier to AC and attack rolls.
    pub fn ac_attack_mod(self) -> i32 {
        match self {
            Size::Fine => 8,
            Size::Diminutive => 4,
            Size::Tiny => 2,
            Size::Small => 1,
            Size::Medium => 0,
            Size::Large => -1,
            Size::Huge => -2,
            Size::Gargantuan => -4,
            Size::Colossal => -8,
        }
    }

    /// Special-size modifier to CMB and CMD (inverse of the AC/attack mod).
    pub fn cmb_cmd_mod(self) -> i32 {
        match self {
            Size::Fine => -8,
            Size::Diminutive => -4,
            Size::Tiny => -2,
            Size::Small => -1,
            Size::Medium => 0,
            Size::Large => 1,
            Size::Huge => 2,
            Size::Gargantuan => 4,
            Size::Colossal => 8,
        }
    }

    pub fn from_data(s: &str) -> Size {
        Self::parse(s)
    }

    fn parse(s: &str) -> Size {
        match s.to_lowercase().as_str() {
            "fine" => Size::Fine,
            "dim" | "diminutive" => Size::Diminutive,
            "tiny" => Size::Tiny,
            "sm" | "small" => Size::Small,
            "lg" | "large" => Size::Large,
            "huge" => Size::Huge,
            "grg" | "gargantuan" => Size::Gargantuan,
            "col" | "colossal" => Size::Colossal,
            _ => Size::Medium,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Size::Medium
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A single class + its level. Multiple entries support multiclassing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevel {
    pub tag: String,
    pub level: u32,
}

/// Coin pouch, tracked per denomination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Coins {
    pub pp: i64,
    pub gp: i64,
    pub sp: i64,
    pub cp: i64,
}

impl Coins {
    /// Total wealth expressed in gold pieces.
    pub fn as_gp(&self) -> f64 {
        self.pp as f64 * 10.0 + self.gp as f64 + self.sp as f64 / 10.0 + self.cp as f64 / 100.0
    }

    /// Total wealth in copper, the smallest denomination.
    ///
    /// Integer math, so it is exact where [`Coins::as_gp`] accumulates float
    /// error.
    pub fn as_cp(&self) -> i64 {
        self.pp * 1000 + self.gp * 100 + self.sp * 10 + self.cp
    }

    /// Total wealth carried up into the largest denominations that fit, as
    /// `(platinum, gold, silver, copper)`.
    ///
    /// Purses hold denominations exactly as entered, so 11 loose silver stays
    /// 11 silver in the model; this expresses the same value as 1 gp 1 sp for
    /// display. A negative balance signs every non-zero component, so the
    /// deficit stays visible even when the platinum place is empty.
    pub fn normalized(&self) -> (i64, i64, i64, i64) {
        let total = self.as_cp();
        let magnitude = total.unsigned_abs() as i64;
        let sign = if total < 0 { -1 } else { 1 };
        (
            sign * (magnitude / 1000),
            sign * (magnitude % 1000 / 100),
            sign * (magnitude % 100 / 10),
            sign * (magnitude % 10),
        )
    }

    /// Subtract a gp cost, drawing from gp then converting down as needed.
    pub fn spend_gp(&mut self, cost: f64) -> bool {
        let total_cp = (self.as_gp() * 100.0).round() as i64;
        let cost_cp = (cost * 100.0).round() as i64;
        if cost_cp > total_cp {
            return false;
        }
        let remaining = total_cp - cost_cp;
        self.pp = 0;
        self.gp = remaining / 100;
        self.sp = (remaining % 100) / 10;
        self.cp = remaining % 10;
        true
    }
}

/// Manual bonuses that feed the derived-stat engine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManualBonuses {
    pub natural_armor: i32,
    pub deflection: i32,
    pub dodge: i32,
    pub misc_ac: i32,
    pub init_misc: i32,
    pub bab_misc: i32,
    pub attack_misc: i32,
    pub cmb_misc: i32,
    pub cmd_misc: i32,
    pub fort_misc: i32,
    pub ref_misc: i32,
    pub will_misc: i32,
    pub hp_misc: i32,
    pub speed: i32,
}

/// Per-skill user input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillEntry {
    pub ranks: i32,
    pub misc: i32,
    /// Marks a skill as a class skill beyond what the class grants.
    pub class_skill_override: bool,
}

/// A homebrew skill defined entirely by the user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomSkill {
    pub uid: u64,
    pub name: String,
    /// Governing ability key (e.g. "int").
    pub ability: String,
    pub ranks: i32,
    pub misc: i32,
    pub class_skill: bool,
    pub trained_only: bool,
}

impl CustomSkill {
    pub fn ability_key(&self) -> &str {
        if self.ability.is_empty() {
            "int"
        } else {
            &self.ability
        }
    }
}

/// A wondrous/worn magic-item body slot. Armor, shields, and weapons are worn
/// through the `equipped` flag instead, since they feed the combat math; these
/// slots hold the one-per-location items (headbands, cloaks, rings, and so on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EquipSlot {
    Head,
    Headband,
    Eyes,
    Neck,
    Shoulders,
    Chest,
    Body,
    Belt,
    Wrists,
    Hands,
    RingLeft,
    RingRight,
    Feet,
}

impl EquipSlot {
    /// Every slot, in head-to-foot wearing order.
    pub const ALL: [EquipSlot; 13] = [
        EquipSlot::Head,
        EquipSlot::Headband,
        EquipSlot::Eyes,
        EquipSlot::Neck,
        EquipSlot::Shoulders,
        EquipSlot::Chest,
        EquipSlot::Body,
        EquipSlot::Belt,
        EquipSlot::Wrists,
        EquipSlot::Hands,
        EquipSlot::RingLeft,
        EquipSlot::RingRight,
        EquipSlot::Feet,
    ];

    pub fn label(self) -> &'static str {
        match self {
            EquipSlot::Head => "Head",
            EquipSlot::Headband => "Headband",
            EquipSlot::Eyes => "Eyes",
            EquipSlot::Neck => "Neck",
            EquipSlot::Shoulders => "Shoulders",
            EquipSlot::Chest => "Chest",
            EquipSlot::Body => "Body",
            EquipSlot::Belt => "Belt",
            EquipSlot::Wrists => "Wrists",
            EquipSlot::Hands => "Hands",
            EquipSlot::RingLeft => "Ring (left)",
            EquipSlot::RingRight => "Ring (right)",
            EquipSlot::Feet => "Feet",
        }
    }

    /// Kinds of item that conventionally occupy the slot, for the UI hint.
    pub fn hint(self) -> &'static str {
        match self {
            EquipSlot::Head => "hat, mask, helm",
            EquipSlot::Headband => "circlet, phylactery",
            EquipSlot::Eyes => "goggles, lenses",
            EquipSlot::Neck => "amulet, throat, periapt",
            EquipSlot::Shoulders => "cloak, mantle, cape",
            EquipSlot::Chest => "shirt, vest",
            EquipSlot::Body => "robe, clothes, vestments",
            EquipSlot::Belt => "belt, girdle, sash",
            EquipSlot::Wrists => "arms, bracers, bracelets",
            EquipSlot::Hands => "gloves, gauntlets",
            EquipSlot::RingLeft => "ring",
            EquipSlot::RingRight => "ring",
            EquipSlot::Feet => "boots, shoes, slippers",
        }
    }
}

/// An inventory line item, denormalized from the compendium for portability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub uid: u64,
    pub source_id: Option<String>,
    pub name: String,
    pub kind: ItemKind,
    pub quantity: u32,
    pub weight: f64,
    pub price: f64,
    pub equipped: bool,
    pub ac_bonus: i32,
    pub max_dex: Option<i32>,
    pub armor_check_penalty: i32,
    pub notes: String,
    /// The worn slot this item occupies, if any (wondrous items only).
    #[serde(default)]
    pub slot: Option<EquipSlot>,
}

/// A spell the character keeps prepared for the day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedSpell {
    pub uid: u64,
    pub spell_id: String,
    pub used: bool,
}

/// The character's spell repertoire.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Spellbook {
    /// Spell ids the character knows / has in their book.
    pub learned: Vec<String>,
    /// Prepared spells for the current day.
    pub prepared: Vec<PreparedSpell>,
    /// Spontaneous slot usage per spell level (index 0 = level 1).
    pub slots_used: BTreeMap<u8, u32>,
}

/// A full editable familiar / companion sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Familiar {
    pub name: String,
    pub species: String,
    pub portrait: Option<String>,
    #[serde(default)]
    pub size: Size,
    #[serde(default)]
    pub abilities: BTreeMap<String, i32>,
    pub hp_current: i32,
    pub hp_max: i32,
    pub natural_armor: i32,
    #[serde(default)]
    pub deflection: i32,
    #[serde(default)]
    pub bab: i32,
    #[serde(default)]
    pub hit_dice: String,
    #[serde(default)]
    pub fort_base: i32,
    #[serde(default)]
    pub ref_base: i32,
    #[serde(default)]
    pub will_base: i32,
    pub speed: String,
    pub senses: String,
    pub attacks: String,
    pub granted_ability: String,
    pub special: String,
    pub notes: String,
    #[serde(default)]
    pub skills: BTreeMap<String, SkillEntry>,
    #[serde(default)]
    pub custom_skills: Vec<CustomSkill>,
}

impl Default for Familiar {
    fn default() -> Self {
        let mut abilities = BTreeMap::new();
        for (key, value) in [
            ("str", 3),
            ("dex", 15),
            ("con", 10),
            ("int", 6),
            ("wis", 12),
            ("cha", 7),
        ] {
            abilities.insert(key.to_string(), value);
        }
        Familiar {
            name: "Familiar".to_string(),
            species: "Cat".to_string(),
            portrait: None,
            size: Size::Tiny,
            abilities,
            hp_current: 3,
            hp_max: 3,
            natural_armor: 0,
            deflection: 0,
            bab: 0,
            hit_dice: "1d8".to_string(),
            fort_base: 0,
            ref_base: 2,
            will_base: 2,
            speed: "30 ft.".to_string(),
            senses: "Low-light vision, scent".to_string(),
            attacks: "2 claws +4 (1d2-4)".to_string(),
            granted_ability: String::new(),
            special: String::new(),
            notes: String::new(),
            skills: BTreeMap::new(),
            custom_skills: Vec::new(),
        }
    }
}

impl Familiar {
    pub fn ability_score(&self, key: &str) -> i32 {
        self.abilities.get(key).copied().unwrap_or(10)
    }

    pub fn ability_mod(&self, key: &str) -> i32 {
        (self.ability_score(key) - 10).div_euclid(2)
    }
}

/// A named, tracked resource pool (ki, hexes/day, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub current: i32,
    pub max: i32,
}

/// A user-authored homebrew entry (feat, hex, spell, or racial trait).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomEntry {
    pub uid: u64,
    pub name: String,
    /// Spell level for custom spells; unused otherwise.
    pub level: i32,
    pub description: String,
}

/// The full character document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub player: String,
    pub race: String,
    pub size: Size,
    pub alignment: String,
    pub deity: String,
    pub gender: String,
    pub age: String,
    pub height: String,
    pub weight_desc: String,
    pub portrait: Option<String>,

    pub classes: Vec<ClassLevel>,
    pub abilities: BTreeMap<String, AbilityScore>,

    pub hp_current: i32,
    pub hp_temp: i32,
    pub nonlethal: i32,
    pub hp_rolled: i32,

    pub bonuses: ManualBonuses,
    pub base_speed: i32,

    pub skills: BTreeMap<String, SkillEntry>,
    pub feats: Vec<String>,
    /// Selected class abilities (hex ids, ninja-trick ids, etc.).
    pub selected_abilities: Vec<String>,
    /// Chosen patron/theme tags for witches.
    pub patron: String,

    pub coins: Coins,
    pub inventory: Vec<InventoryItem>,
    pub spellbook: Spellbook,
    pub familiar: Option<Familiar>,
    /// Named resource pools keyed by an identifier (e.g. "ki", "hexes").
    pub resources: BTreeMap<String, ResourcePool>,

    pub conditions: Vec<String>,
    pub notes: String,

    /// Narrative data
    pub origins: String,
    pub appearance: String,
    pub personality: String,
    pub backstory: String,
    pub affiliation: String,
    pub friends: String,
    pub foes: String,

    /// Monotonic counter for allocating item/spell uids.
    pub next_uid: u64,

    #[serde(default)]
    pub custom_feats: Vec<CustomEntry>,
    #[serde(default)]
    pub custom_abilities: Vec<CustomEntry>,
    #[serde(default)]
    pub custom_spells: Vec<CustomEntry>,
    #[serde(default)]
    pub racial_traits: Vec<CustomEntry>,
    #[serde(default)]
    pub custom_skills: Vec<CustomSkill>,
    #[serde(default)]
    pub gallery: Vec<String>,
    /// Free-form list of languages the character speaks, one per line.
    #[serde(default)]
    pub languages: String,
}

impl Character {
    pub fn new(id: u64, name: impl Into<String>, class_tag: impl Into<String>) -> Self {
        let mut abilities = BTreeMap::new();
        for key in ABILITIES {
            abilities.insert(key.to_string(), AbilityScore::default());
        }
        Character {
            id,
            name: name.into(),
            player: String::new(),
            race: String::new(),
            size: Size::Medium,
            alignment: String::new(),
            deity: String::new(),
            gender: String::new(),
            age: String::new(),
            height: String::new(),
            weight_desc: String::new(),
            portrait: None,
            classes: vec![ClassLevel {
                tag: class_tag.into(),
                level: 1,
            }],
            abilities,
            hp_current: 8,
            hp_temp: 0,
            nonlethal: 0,
            hp_rolled: 0,
            bonuses: ManualBonuses::default(),
            base_speed: 30,
            skills: BTreeMap::new(),
            feats: Vec::new(),
            selected_abilities: Vec::new(),
            patron: String::new(),
            coins: Coins::default(),
            inventory: Vec::new(),
            spellbook: Spellbook::default(),
            familiar: None,
            resources: BTreeMap::new(),
            conditions: Vec::new(),
            origins: String::new(),
            appearance: String::new(),
            personality: String::new(),
            backstory: String::new(),
            affiliation: String::new(),
            friends: String::new(),
            foes: String::new(),
            notes: String::new(),
            next_uid: 1,
            custom_feats: Vec::new(),
            custom_abilities: Vec::new(),
            custom_spells: Vec::new(),
            racial_traits: Vec::new(),
            custom_skills: Vec::new(),
            gallery: Vec::new(),
            languages: String::new(),
        }
    }

    pub fn ability(&self, key: &str) -> &AbilityScore {
        self.abilities
            .get(key)
            .expect("ability scores are always initialized")
    }

    pub fn ability_mut(&mut self, key: &str) -> &mut AbilityScore {
        self.abilities
            .entry(key.to_string())
            .or_insert_with(AbilityScore::default)
    }

    pub fn ability_mod(&self, key: &str) -> i32 {
        self.ability(key).modifier()
    }

    pub fn primary_class(&self) -> Option<&ClassLevel> {
        self.classes.first()
    }

    pub fn primary_tag(&self) -> String {
        self.classes
            .first()
            .map(|c| c.tag.clone())
            .unwrap_or_default()
    }

    pub fn total_level(&self) -> u32 {
        self.classes.iter().map(|c| c.level).sum()
    }

    pub fn class_level(&self, tag: &str) -> u32 {
        self.classes
            .iter()
            .filter(|c| c.tag == tag)
            .map(|c| c.level)
            .sum()
    }

    pub fn skill(&self, id: &str) -> SkillEntry {
        self.skills.get(id).cloned().unwrap_or_default()
    }

    pub fn skill_mut(&mut self, id: &str) -> &mut SkillEntry {
        self.skills.entry(id.to_string()).or_default()
    }

    pub fn alloc_uid(&mut self) -> u64 {
        let uid = self.next_uid;
        self.next_uid += 1;
        uid
    }
}

#[cfg(test)]
mod tests {
    use super::Coins;

    fn coins(pp: i64, gp: i64, sp: i64, cp: i64) -> Coins {
        Coins { pp, gp, sp, cp }
    }

    #[test]
    fn normalizes_loose_change_upward() {
        assert_eq!(coins(0, 0, 11, 0).normalized(), (0, 1, 1, 0));
        assert_eq!(coins(0, 0, 0, 1234).normalized(), (1, 2, 3, 4));
        assert_eq!(coins(0, 15, 0, 0).normalized(), (1, 5, 0, 0));
    }

    #[test]
    fn leaves_already_normal_purses_alone() {
        assert_eq!(coins(1, 2, 3, 4).normalized(), (1, 2, 3, 4));
        assert_eq!(coins(0, 0, 0, 0).normalized(), (0, 0, 0, 0));
    }

    /// A deficit must stay visibly negative even with an empty platinum place.
    #[test]
    fn signs_every_component_when_negative() {
        assert_eq!(coins(0, -1, -1, 0).normalized(), (0, -1, -1, 0));
        assert_eq!(coins(0, 0, -11, 0).normalized(), (0, -1, -1, 0));
    }

    #[test]
    fn copper_total_is_exact() {
        assert_eq!(coins(1, 2, 3, 4).as_cp(), 1234);
        assert_eq!(coins(0, 0, 11, 0).as_cp(), 110);
    }
}