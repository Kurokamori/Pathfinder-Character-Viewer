//! The fixed Pathfinder 1e skill list. Ids match Foundry's class-skill keys.

/// A skill definition.
#[derive(Debug, Clone, Copy)]
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    /// Governing ability key.
    pub ability: &'static str,
    /// Whether the armor check penalty applies.
    pub armor_check: bool,
    /// Whether ranks are required before the skill can be used.
    pub trained_only: bool,
}

pub const SKILLS: &[SkillDef] = &[
    SkillDef { id: "acr", name: "Acrobatics", ability: "dex", armor_check: true, trained_only: false },
    SkillDef { id: "apr", name: "Appraise", ability: "int", armor_check: false, trained_only: false },
    SkillDef { id: "art", name: "Artistry", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "blf", name: "Bluff", ability: "cha", armor_check: false, trained_only: false },
    SkillDef { id: "clm", name: "Climb", ability: "str", armor_check: true, trained_only: false },
    SkillDef { id: "crf", name: "Craft", ability: "int", armor_check: false, trained_only: false },
    SkillDef { id: "dip", name: "Diplomacy", ability: "cha", armor_check: false, trained_only: false },
    SkillDef { id: "dev", name: "Disable Device", ability: "dex", armor_check: true, trained_only: true },
    SkillDef { id: "dis", name: "Disguise", ability: "cha", armor_check: false, trained_only: false },
    SkillDef { id: "esc", name: "Escape Artist", ability: "dex", armor_check: true, trained_only: false },
    SkillDef { id: "fly", name: "Fly", ability: "dex", armor_check: true, trained_only: false },
    SkillDef { id: "han", name: "Handle Animal", ability: "cha", armor_check: false, trained_only: true },
    SkillDef { id: "hea", name: "Heal", ability: "wis", armor_check: false, trained_only: false },
    SkillDef { id: "int", name: "Intimidate", ability: "cha", armor_check: false, trained_only: false },
    SkillDef { id: "kar", name: "Knowledge (arcana)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kdu", name: "Knowledge (dungeoneering)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "ken", name: "Knowledge (engineering)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kge", name: "Knowledge (geography)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "khi", name: "Knowledge (history)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "klo", name: "Knowledge (local)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kna", name: "Knowledge (nature)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kno", name: "Knowledge (nobility)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kpl", name: "Knowledge (planes)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "kre", name: "Knowledge (religion)", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "lin", name: "Linguistics", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "lor", name: "Lore", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "per", name: "Perception", ability: "wis", armor_check: false, trained_only: false },
    SkillDef { id: "prf", name: "Perform", ability: "cha", armor_check: false, trained_only: false },
    SkillDef { id: "pro", name: "Profession", ability: "wis", armor_check: false, trained_only: true },
    SkillDef { id: "rid", name: "Ride", ability: "dex", armor_check: true, trained_only: false },
    SkillDef { id: "sen", name: "Sense Motive", ability: "wis", armor_check: false, trained_only: false },
    SkillDef { id: "slt", name: "Sleight of Hand", ability: "dex", armor_check: true, trained_only: true },
    SkillDef { id: "spl", name: "Spellcraft", ability: "int", armor_check: false, trained_only: true },
    SkillDef { id: "ste", name: "Stealth", ability: "dex", armor_check: true, trained_only: false },
    SkillDef { id: "sur", name: "Survival", ability: "wis", armor_check: false, trained_only: false },
    SkillDef { id: "swm", name: "Swim", ability: "str", armor_check: true, trained_only: false },
    SkillDef { id: "umd", name: "Use Magic Device", ability: "cha", armor_check: false, trained_only: true },
];

pub fn skill_def(id: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.id == id)
}
