//! Per-class mechanical progression: BAB, saves, casting, and class features.
//! The Foundry packs omit much of this, so the authoritative tables live here.

/// Base attack bonus growth rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bab {
    Full,
    ThreeQuarter,
    Half,
}

impl Bab {
    pub fn at(self, level: u32) -> i32 {
        let l = level as i32;
        match self {
            Bab::Full => l,
            Bab::ThreeQuarter => l * 3 / 4,
            Bab::Half => l / 2,
        }
    }
}

/// Saving-throw growth rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Save {
    Good,
    Poor,
}

impl Save {
    pub fn at(self, level: u32) -> i32 {
        let l = level as i32;
        match self {
            Save::Good => 2 + l / 2,
            Save::Poor => l / 3,
        }
    }
}

/// How a class casts spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Casting {
    None,
    /// Prepares spells from a known list (witch, wizard).
    Prepared,
    /// Casts spontaneously from spells known (sorcerer, bard).
    Spontaneous,
}

/// The standard full-caster (9-level) spells-per-day table, indexed
/// `[class_level-1][spell_level]` where spell_level is 0..=9.
pub const FULL_CASTER: [[i8; 10]; 20] = [
    [3, 1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 2, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 2, 1, -1, -1, -1, -1, -1, -1, -1],
    [4, 3, 2, -1, -1, -1, -1, -1, -1, -1],
    [4, 3, 2, 1, -1, -1, -1, -1, -1, -1],
    [4, 3, 3, 2, -1, -1, -1, -1, -1, -1],
    [4, 4, 3, 2, 1, -1, -1, -1, -1, -1],
    [4, 4, 3, 3, 2, -1, -1, -1, -1, -1],
    [4, 4, 4, 3, 2, 1, -1, -1, -1, -1],
    [4, 4, 4, 3, 3, 2, -1, -1, -1, -1],
    [4, 4, 4, 4, 3, 2, 1, -1, -1, -1],
    [4, 4, 4, 4, 3, 3, 2, -1, -1, -1],
    [4, 4, 4, 4, 4, 3, 2, 1, -1, -1],
    [4, 4, 4, 4, 4, 3, 3, 2, -1, -1],
    [4, 4, 4, 4, 4, 4, 3, 2, 1, -1],
    [4, 4, 4, 4, 4, 4, 3, 3, 2, -1],
    [4, 4, 4, 4, 4, 4, 4, 3, 2, 1],
    [4, 4, 4, 4, 4, 4, 4, 3, 3, 2],
    [4, 4, 4, 4, 4, 4, 4, 4, 3, 3],
    [4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
];

/// A class's full mechanical definition.
#[derive(Debug, Clone, Copy)]
pub struct ClassDef {
    pub tag: &'static str,
    pub name: &'static str,
    pub hit_die: u32,
    pub bab: Bab,
    pub fort: Save,
    pub reflex: Save,
    pub will: Save,
    pub skills_per_level: u32,
    pub casting: Casting,
    pub casting_ability: &'static str,
    pub class_skills: &'static [&'static str],
    pub has_familiar: bool,
    /// Spells-per-day table if a spellcaster.
    pub spells_per_day: Option<&'static [[i8; 10]; 20]>,
}

impl ClassDef {
    /// Base spells per day (before ability bonus) for a class level and spell level.
    pub fn base_spells_per_day(&self, class_level: u32, spell_level: usize) -> Option<u32> {
        let table = self.spells_per_day?;
        if class_level == 0 || spell_level > 9 {
            return None;
        }
        let value = table[(class_level as usize - 1).min(19)][spell_level];
        if value < 0 {
            None
        } else {
            Some(value as u32)
        }
    }

    /// Bonus spells per day from a high casting ability score.
    pub fn bonus_spells(&self, spell_level: usize, ability_mod: i32) -> u32 {
        if spell_level == 0 || spell_level > 9 {
            return 0;
        }
        let level = spell_level as i32;
        if ability_mod < level {
            0
        } else {
            ((ability_mod - level) / 4 + 1) as u32
        }
    }

    /// Highest spell level castable at a given class level.
    pub fn max_spell_level(&self, class_level: u32) -> Option<usize> {
        let table = self.spells_per_day?;
        if class_level == 0 {
            return None;
        }
        let row = &table[(class_level as usize - 1).min(19)];
        (1..=9).rev().find(|&lvl| row[lvl] >= 0)
    }
}

const WITCH_SKILLS: &[&str] = &[
    "crf", "fly", "hea", "int", "kar", "khi", "kna", "kpl", "pro", "spl", "umd",
];

const NINJA_SKILLS: &[&str] = &[
    "acr", "apr", "blf", "clm", "crf", "dip", "dis", "esc", "int", "klo", "lin", "per",
    "prf", "pro", "sen", "slt", "ste", "swm", "umd",
];

const GENERIC_SKILLS: &[&str] = &[];

pub const CLASSES: &[ClassDef] = &[
    ClassDef {
        tag: "witch",
        name: "Witch",
        hit_die: 6,
        bab: Bab::ThreeQuarter,
        fort: Save::Poor,
        reflex: Save::Poor,
        will: Save::Good,
        skills_per_level: 2,
        casting: Casting::Prepared,
        casting_ability: "int",
        class_skills: WITCH_SKILLS,
        has_familiar: true,
        spells_per_day: Some(&FULL_CASTER),
    },
    ClassDef {
        tag: "ninja",
        name: "Ninja",
        hit_die: 8,
        bab: Bab::ThreeQuarter,
        fort: Save::Poor,
        reflex: Save::Good,
        will: Save::Poor,
        skills_per_level: 8,
        casting: Casting::None,
        casting_ability: "cha",
        class_skills: NINJA_SKILLS,
        has_familiar: false,
        spells_per_day: None,
    },
];

/// A safe default for classes without an explicit definition.
pub const GENERIC: ClassDef = ClassDef {
    tag: "",
    name: "Adventurer",
    hit_die: 8,
    bab: Bab::ThreeQuarter,
    fort: Save::Poor,
    reflex: Save::Poor,
    will: Save::Poor,
    skills_per_level: 2,
    casting: Casting::None,
    casting_ability: "int",
    class_skills: GENERIC_SKILLS,
    has_familiar: false,
    spells_per_day: None,
};

pub fn class_def(tag: &str) -> ClassDef {
    CLASSES
        .iter()
        .find(|c| c.tag == tag)
        .copied()
        .unwrap_or(GENERIC)
}

/// Class tags that ship with a full custom module/layout.
pub fn supported_tags() -> Vec<&'static str> {
    CLASSES.iter().map(|c| c.tag).collect()
}
