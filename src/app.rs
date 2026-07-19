//! Application state, the message set, and the update loop.

use crate::data::loader;
use crate::data::GameData;
use crate::model::character::{Character, Size};
use crate::model::compendium::{AbilityCategory, Compendium, ItemKind};
use crate::model::settings::Settings;
use crate::persistence::{self, CharacterSummary};
use crate::theme::{self, Palette};
use crate::ui::{self, Tab};
use crate::{classes, rules};
use iced::{Color, Element, Subscription, Task};
use std::time::Duration;

/// Which source of an ability score an edit targets.
#[derive(Debug, Clone, Copy)]
pub enum AbilityPart {
    Base,
    Racial,
    Enhancement,
    Temp,
}

/// A manual combat/defense bonus field.
#[derive(Debug, Clone, Copy)]
pub enum BonusField {
    NaturalArmor,
    Deflection,
    Dodge,
    MiscAc,
    Init,
    Bab,
    Attack,
    Cmb,
    Cmd,
    Fort,
    Ref,
    Will,
    HpMisc,
    Speed,
}

/// A free-text identity field.
#[derive(Debug, Clone, Copy)]
pub enum IdentityField {
    Name,
    Player,
    Alignment,
    Deity,
    Gender,
    Age,
    Height,
    Weight,
}

/// Which homebrew list a custom-entry message targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomList {
    Feat,
    Ability,
    Spell,
    RacialTrait,
}

/// Identifies a multiline text-editor field. Doubles as the key for the
/// `App::editors` map, so it must stay hashable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTarget {
    FamiliarGranted,
    FamiliarSpecial,
    FamiliarNotes,
    /// A homebrew entry's description, keyed by list and entry uid.
    CustomDesc(CustomList, u64),
    /// An inventory item's notes/description, keyed by item uid.
    InventoryNotes(u64),
}

/// Which spells the browse list shows.
///
/// `Level` filters the whole class list by spell level (`None` = every level).
/// `Known` restricts to spells the character has learned, optionally further
/// narrowed to a single level (`None` = every known spell).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellFilter {
    Level(Option<u8>),
    Known(Option<u8>),
}

impl Default for SpellFilter {
    fn default() -> Self {
        SpellFilter::Level(None)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CoinField {
    Pp,
    Gp,
    Sp,
    Cp,
}

#[derive(Debug, Clone, Copy)]
pub enum FamiliarField {
    Name,
    Species,
    HpCurrent,
    HpMax,
    NaturalArmor,
    Deflection,
    Bab,
    HitDice,
    FortBase,
    RefBase,
    WillBase,
    Speed,
    Senses,
    Attacks,
    Granted,
    Special,
    Notes,
}

/// Whether a skill message targets the character or its familiar.
#[derive(Debug, Clone, Copy)]
pub enum SkillScope {
    Character,
    Familiar,
}

#[derive(Debug, Clone, Copy)]
pub enum CustomSkillField {
    Name,
    Ability,
    Ranks,
    Misc,
}

#[derive(Debug, Clone, Copy)]
pub enum CustomSkillFlag {
    Class,
    Trained,
}

/// Every user interaction, grouped by area.
#[derive(Debug, Clone)]
pub enum Message {
    ShowRoster,
    NewCharacter(String),
    OpenCharacter(u64),
    DeleteCharacter(u64),
    ImportCharacter,
    ExportCharacter,
    OpenSettings(bool),
    SelectTab(Tab),
    SaveNow,
    AutosaveTick,
    ChangePortrait,
    ChangeFamiliarPortrait,
    NoOp,

    SetIdentity(IdentityField, String),
    SetRace(String),
    SetSize(Size),
    LevelDelta(usize, i32),
    AddClass(String),
    RemoveClass(usize),

    SetAbility(String, AbilityPart, String),

    SetHpCurrent(String),
    SetHpTemp(String),
    SetNonlethal(String),
    SetRolledHp(String),
    HpDelta(i32),

    SetBonus(BonusField, String),

    SetSkillRanks(String, String),
    SetSkillMisc(String, String),
    ToggleClassSkill(String),

    CustomSkillAdd(SkillScope),
    CustomSkillRemove(SkillScope, u64),
    CustomSkillSet(SkillScope, u64, CustomSkillField, String),
    CustomSkillToggle(SkillScope, u64, CustomSkillFlag),

    SpellSearch(String),
    SpellSetFilter(SpellFilter),
    SpellToggleLearned(String),
    SpellPrepare(String),
    SpellUnprepare(u64),
    SpellToggleUsed(u64),
    SpellRest,
    SpellSelect(Option<String>),

    HexSearch(String),
    HexCategoryFilter(Option<AbilityCategory>),
    HexToggle(String),
    HexSelect(Option<String>),
    SetPatron(String),

    TrickToggle(String),
    KiDelta(i32),
    KiRest,

    ShopKind(ItemKind),
    ShopSearch(String),
    ShopBuy(String),
    ShopAdd(String),
    ShopSelect(Option<String>),
    InvAdd,
    InvRemove(u64),
    InvExpand(u64),
    InvToggleEquip(u64),
    InvSetQty(u64, String),
    InvSetName(u64, String),
    InvSetWeight(u64, String),
    InvSetPrice(u64, String),
    SetCoins(CoinField, String),

    CustomAdd(CustomList),
    CustomRemove(CustomList, u64),
    CustomSetName(CustomList, u64, String),
    CustomSetLevel(CustomList, u64, String),
    CustomSetDesc(CustomList, u64, String),

    FamiliarToggle,
    SetFamiliar(FamiliarField, String),
    SetFamiliarSize(Size),
    SetFamiliarAbility(String, String),
    FamiliarSkillRanks(String, String),
    FamiliarSkillMisc(String, String),

    GalleryAdd,
    GalleryRemove(usize),
    FeatureExpand(String),
    CombatRefExpand(String),

    FeatSearch(String),
    FeatAdd(String),
    FeatRemove(String),
    FeatSelect(Option<String>),

    ToggleBook(String),
    ToggleUnlabeled,

    NotesAction(iced::widget::text_editor::Action),
    EditorAction(EditorTarget, iced::widget::text_editor::Action),
    ToggleCondition(String),
}

/// The whole application.
pub struct App {
    pub game: GameData,
    pub load_error: Option<String>,
    pub settings: Settings,
    pub character: Character,
    pub roster: Vec<CharacterSummary>,
    pub tabs: Vec<Tab>,
    pub active_tab: Tab,
    pub on_roster: bool,
    pub show_settings: bool,
    pub dirty: bool,
    pub accent: Color,

    pub spell_search: String,
    pub spell_filter: SpellFilter,
    pub selected_spell: Option<String>,
    pub hex_search: String,
    pub hex_category: Option<AbilityCategory>,
    pub selected_hex: Option<String>,
    pub shop_kind: ItemKind,
    pub shop_search: String,
    pub selected_item: Option<String>,
    pub expanded_item: Option<u64>,
    pub feat_search: String,
    pub selected_feat: Option<String>,
    pub expanded_feature: Option<String>,
    pub expanded_combat_ref: Option<String>,
    pub notes_content: iced::widget::text_editor::Content,
    /// Live multiline editor buffers, keyed by the field they edit.
    pub editors: std::collections::HashMap<EditorTarget, iced::widget::text_editor::Content>,
    pub image_cache: crate::ui::images::ImageCache,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (compendium, load_error) = match loader::load() {
            Ok(c) => (c, None),
            Err(e) => (Compendium::default(), Some(e)),
        };
        let game = GameData::new(compendium);
        let settings = persistence::load_settings();
        let roster = persistence::list_characters();

        let mut app = App {
            game,
            load_error,
            settings,
            character: Character::new(0, "", ""),
            roster,
            tabs: Vec::new(),
            active_tab: Tab::General,
            on_roster: true,
            show_settings: false,
            dirty: false,
            accent: theme::accent_for(""),
            spell_search: String::new(),
            spell_filter: SpellFilter::default(),
            selected_spell: None,
            hex_search: String::new(),
            hex_category: None,
            selected_hex: None,
            shop_kind: ItemKind::Weapon,
            shop_search: String::new(),
            selected_item: None,
            expanded_item: None,
            feat_search: String::new(),
            selected_feat: None,
            expanded_feature: None,
            expanded_combat_ref: None,
            notes_content: iced::widget::text_editor::Content::new(),
            editors: std::collections::HashMap::new(),
            image_cache: crate::ui::images::ImageCache::default(),
        };

        if let Some(id) = app.settings.last_character {
            if let Ok(character) = persistence::load_character_by_id(id) {
                app.adopt_character(character);
                app.on_roster = false;
            }
        }

        (app, Task::none())
    }

    pub fn palette(&self) -> Palette {
        Palette::new(self.accent)
    }

    /// Install a character as the active one and derive its layout.
    fn adopt_character(&mut self, mut character: Character) {
        let tag = character.primary_tag();
        classes::sync(&mut character, &self.game);
        self.accent = theme::accent_for(&tag);
        self.tabs = classes::tabs_for(&tag);
        self.active_tab = self.tabs.first().copied().unwrap_or(Tab::General);
        self.notes_content = iced::widget::text_editor::Content::with_text(&character.notes);
        self.image_cache.clear();
        self.character = character;
        self.editors.clear();
        self.sync_editors();
        self.selected_spell = None;
        self.selected_hex = None;
        self.selected_item = None;
        self.selected_feat = None;
        self.expanded_combat_ref = None;
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn resync(&mut self) {
        classes::sync(&mut self.character, &self.game);
    }

    fn save_current(&mut self) {
        if self.on_roster {
            return;
        }
        let _ = persistence::save_character(&self.character);
        self.settings.last_character = Some(self.character.id);
        let _ = persistence::save_settings(&self.settings);
        self.dirty = false;
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(3)).map(|_| Message::AutosaveTick)
    }

    pub fn theme(&self) -> iced::Theme {
        theme::base_theme()
    }

    pub fn view(&self) -> Element<'_, Message> {
        ui::view(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => {}
            Message::AutosaveTick => {
                if self.dirty && !self.on_roster {
                    self.save_current();
                }
            }
            Message::SaveNow => self.save_current(),
            Message::ShowRoster => {
                self.save_current();
                self.roster = persistence::list_characters();
                self.on_roster = true;
                self.show_settings = false;
            }
            Message::OpenSettings(open) => self.show_settings = open,
            Message::NewCharacter(tag) => {
                let mut character = Character::new(persistence::new_id(), "New Character", tag.clone());
                classes::init_character(&mut character, &tag, &self.game);
                self.adopt_character(character);
                self.on_roster = false;
                self.save_current();
            }
            Message::OpenCharacter(id) => {
                if let Ok(character) = persistence::load_character_by_id(id) {
                    self.adopt_character(character);
                    self.on_roster = false;
                    self.save_current();
                }
            }
            Message::DeleteCharacter(id) => {
                let _ = persistence::delete_character(id);
                self.roster = persistence::list_characters();
            }
            Message::ImportCharacter => {
                if let Ok(Some(character)) = persistence::import_character() {
                    self.adopt_character(character);
                    self.on_roster = false;
                    self.save_current();
                }
            }
            Message::ExportCharacter => {
                let _ = persistence::export_character(&self.character);
            }
            Message::SelectTab(tab) => self.active_tab = tab,
            Message::ChangePortrait => {
                if let Some(path) = persistence::pick_portrait() {
                    self.character.portrait = Some(path.to_string_lossy().to_string());
                    self.mark_dirty();
                }
            }
            Message::ChangeFamiliarPortrait => {
                if let Some(path) = persistence::pick_portrait() {
                    if let Some(fam) = self.character.familiar.as_mut() {
                        fam.portrait = Some(path.to_string_lossy().to_string());
                        self.mark_dirty();
                    }
                }
            }

            Message::SetIdentity(field, value) => {
                match field {
                    IdentityField::Name => self.character.name = value,
                    IdentityField::Player => self.character.player = value,
                    IdentityField::Alignment => self.character.alignment = value,
                    IdentityField::Deity => self.character.deity = value,
                    IdentityField::Gender => self.character.gender = value,
                    IdentityField::Age => self.character.age = value,
                    IdentityField::Height => self.character.height = value,
                    IdentityField::Weight => self.character.weight_desc = value,
                }
                self.mark_dirty();
            }
            Message::SetRace(name) => {
                self.character.race = name.clone();
                if let Some(race) = self.game.compendium.races.iter().find(|r| r.name == name) {
                    self.character.size = Size::from_data(&race.size);
                    for key in crate::model::character::ABILITIES {
                        let racial = race.ability_mods.get(key).copied().unwrap_or(0);
                        self.character.ability_mut(key).racial = racial;
                    }
                }
                self.resync();
                self.mark_dirty();
            }
            Message::SetSize(size) => {
                self.character.size = size;
                self.mark_dirty();
            }
            Message::LevelDelta(index, delta) => {
                if let Some(class) = self.character.classes.get_mut(index) {
                    let next = class.level as i32 + delta;
                    class.level = next.clamp(1, 20) as u32;
                    self.resync();
                    self.mark_dirty();
                }
            }
            Message::AddClass(tag) => {
                if !tag.is_empty() {
                    self.character.classes.push(crate::model::character::ClassLevel {
                        tag,
                        level: 1,
                    });
                    self.resync();
                    self.mark_dirty();
                }
            }
            Message::RemoveClass(index) => {
                if self.character.classes.len() > 1 {
                    self.character.classes.remove(index);
                    self.resync();
                    self.mark_dirty();
                }
            }

            Message::SetAbility(key, part, value) => {
                let n = ui::widgets::parse_int(&value);
                let score = self.character.ability_mut(&key);
                match part {
                    AbilityPart::Base => score.base = n,
                    AbilityPart::Racial => score.racial = n,
                    AbilityPart::Enhancement => score.enhancement = n,
                    AbilityPart::Temp => score.temp = n,
                }
                self.resync();
                self.mark_dirty();
            }

            Message::SetHpCurrent(v) => {
                self.character.hp_current = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::SetHpTemp(v) => {
                self.character.hp_temp = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::SetNonlethal(v) => {
                self.character.nonlethal = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::SetRolledHp(v) => {
                self.character.hp_rolled = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::HpDelta(delta) => {
                self.character.hp_current += delta;
                self.mark_dirty();
            }

            Message::SetBonus(field, value) => {
                let n = ui::widgets::parse_int(&value);
                let b = &mut self.character.bonuses;
                match field {
                    BonusField::NaturalArmor => b.natural_armor = n,
                    BonusField::Deflection => b.deflection = n,
                    BonusField::Dodge => b.dodge = n,
                    BonusField::MiscAc => b.misc_ac = n,
                    BonusField::Init => b.init_misc = n,
                    BonusField::Bab => b.bab_misc = n,
                    BonusField::Attack => b.attack_misc = n,
                    BonusField::Cmb => b.cmb_misc = n,
                    BonusField::Cmd => b.cmd_misc = n,
                    BonusField::Fort => b.fort_misc = n,
                    BonusField::Ref => b.ref_misc = n,
                    BonusField::Will => b.will_misc = n,
                    BonusField::HpMisc => b.hp_misc = n,
                    BonusField::Speed => self.character.base_speed = n,
                }
                self.mark_dirty();
            }

            Message::SetSkillRanks(id, v) => {
                self.character.skill_mut(&id).ranks = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::SetSkillMisc(id, v) => {
                self.character.skill_mut(&id).misc = ui::widgets::parse_int(&v);
                self.mark_dirty();
            }
            Message::ToggleClassSkill(id) => {
                let entry = self.character.skill_mut(&id);
                entry.class_skill_override = !entry.class_skill_override;
                self.mark_dirty();
            }

            Message::SpellSearch(v) => self.spell_search = v,
            Message::SpellSetFilter(v) => self.spell_filter = v,
            Message::SpellSelect(v) => self.selected_spell = v,
            Message::SpellToggleLearned(id) => {
                let book = &mut self.character.spellbook;
                if let Some(pos) = book.learned.iter().position(|s| s == &id) {
                    book.learned.remove(pos);
                    book.prepared.retain(|p| p.spell_id != id);
                } else {
                    book.learned.push(id);
                }
                self.mark_dirty();
            }
            Message::SpellPrepare(id) => {
                let uid = self.character.alloc_uid();
                self.character.spellbook.prepared.push(
                    crate::model::character::PreparedSpell {
                        uid,
                        spell_id: id,
                        used: false,
                    },
                );
                self.mark_dirty();
            }
            Message::SpellUnprepare(uid) => {
                self.character.spellbook.prepared.retain(|p| p.uid != uid);
                self.mark_dirty();
            }
            Message::SpellToggleUsed(uid) => {
                if let Some(p) = self
                    .character
                    .spellbook
                    .prepared
                    .iter_mut()
                    .find(|p| p.uid == uid)
                {
                    p.used = !p.used;
                    self.mark_dirty();
                }
            }
            Message::SpellRest => {
                for p in self.character.spellbook.prepared.iter_mut() {
                    p.used = false;
                }
                self.character.spellbook.slots_used.clear();
                self.mark_dirty();
            }

            Message::HexSearch(v) => self.hex_search = v,
            Message::HexCategoryFilter(v) => self.hex_category = v,
            Message::HexSelect(v) => self.selected_hex = v,
            Message::HexToggle(id) => {
                toggle_vec(&mut self.character.selected_abilities, id);
                self.mark_dirty();
            }
            Message::SetPatron(name) => {
                self.character.patron = name;
                self.selected_hex = None;
                self.mark_dirty();
            }

            Message::TrickToggle(id) => {
                toggle_vec(&mut self.character.selected_abilities, id);
                self.mark_dirty();
            }
            Message::KiDelta(delta) => {
                if let Some(pool) = self.character.resources.get_mut("ki") {
                    pool.current = (pool.current + delta).clamp(0, pool.max.max(0));
                    self.mark_dirty();
                }
            }
            Message::KiRest => {
                if let Some(pool) = self.character.resources.get_mut("ki") {
                    pool.current = pool.max;
                    self.mark_dirty();
                }
            }

            Message::ShopKind(kind) => {
                self.shop_kind = kind;
                self.selected_item = None;
            }
            Message::ShopSearch(v) => self.shop_search = v,
            Message::ShopSelect(v) => self.selected_item = v,
            Message::ShopBuy(id) => self.add_item(&id, true),
            Message::ShopAdd(id) => self.add_item(&id, false),
            Message::InvAdd => {
                let uid = self.character.alloc_uid();
                self.character.inventory.push(crate::model::character::InventoryItem {
                    uid,
                    source_id: None,
                    name: "New Item".to_string(),
                    kind: ItemKind::Gear,
                    quantity: 1,
                    weight: 0.0,
                    price: 0.0,
                    equipped: false,
                    ac_bonus: 0,
                    max_dex: None,
                    armor_check_penalty: 0,
                    notes: String::new(),
                });
                self.mark_dirty();
            }
            Message::InvRemove(uid) => {
                self.character.inventory.retain(|i| i.uid != uid);
                if self.expanded_item == Some(uid) {
                    self.expanded_item = None;
                }
                self.mark_dirty();
            }
            Message::InvExpand(uid) => {
                self.expanded_item = if self.expanded_item == Some(uid) {
                    None
                } else {
                    Some(uid)
                };
            }
            Message::InvToggleEquip(uid) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.equipped = !item.equipped;
                    self.mark_dirty();
                }
            }
            Message::InvSetQty(uid, v) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.quantity = ui::widgets::parse_u32(&v);
                    self.mark_dirty();
                }
            }
            Message::InvSetName(uid, v) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.name = v;
                    self.mark_dirty();
                }
            }
            Message::InvSetWeight(uid, v) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.weight = ui::widgets::parse_f64(&v);
                    self.mark_dirty();
                }
            }
            Message::InvSetPrice(uid, v) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.price = ui::widgets::parse_f64(&v);
                    self.mark_dirty();
                }
            }

            Message::CustomAdd(which) => {
                let uid = self.character.alloc_uid();
                self.custom_list_mut(which).push(crate::model::character::CustomEntry {
                    uid,
                    name: "New Entry".to_string(),
                    level: 0,
                    description: String::new(),
                });
                self.mark_dirty();
            }
            Message::CustomRemove(which, uid) => {
                self.custom_list_mut(which).retain(|e| e.uid != uid);
                self.mark_dirty();
            }
            Message::CustomSetName(which, uid, v) => {
                if let Some(entry) = self.custom_list_mut(which).iter_mut().find(|e| e.uid == uid) {
                    entry.name = v;
                    self.mark_dirty();
                }
            }
            Message::CustomSetLevel(which, uid, v) => {
                let level = ui::widgets::parse_int(&v).clamp(0, 9);
                if let Some(entry) = self.custom_list_mut(which).iter_mut().find(|e| e.uid == uid) {
                    entry.level = level;
                    self.mark_dirty();
                }
            }
            Message::CustomSetDesc(which, uid, v) => {
                if let Some(entry) = self.custom_list_mut(which).iter_mut().find(|e| e.uid == uid) {
                    entry.description = v;
                    self.mark_dirty();
                }
            }
            Message::SetCoins(field, v) => {
                let n = ui::widgets::parse_int(&v) as i64;
                match field {
                    CoinField::Pp => self.character.coins.pp = n,
                    CoinField::Gp => self.character.coins.gp = n,
                    CoinField::Sp => self.character.coins.sp = n,
                    CoinField::Cp => self.character.coins.cp = n,
                }
                self.mark_dirty();
            }

            Message::FamiliarToggle => {
                if self.character.familiar.is_some() {
                    self.character.familiar = None;
                } else {
                    self.character.familiar = Some(crate::model::character::Familiar::default());
                }
                self.mark_dirty();
            }
            Message::SetFamiliar(field, value) => {
                if let Some(fam) = self.character.familiar.as_mut() {
                    match field {
                        FamiliarField::Name => fam.name = value,
                        FamiliarField::Species => fam.species = value,
                        FamiliarField::HpCurrent => fam.hp_current = ui::widgets::parse_int(&value),
                        FamiliarField::HpMax => fam.hp_max = ui::widgets::parse_int(&value),
                        FamiliarField::NaturalArmor => {
                            fam.natural_armor = ui::widgets::parse_int(&value)
                        }
                        FamiliarField::Deflection => fam.deflection = ui::widgets::parse_int(&value),
                        FamiliarField::Bab => fam.bab = ui::widgets::parse_int(&value),
                        FamiliarField::HitDice => fam.hit_dice = value,
                        FamiliarField::FortBase => fam.fort_base = ui::widgets::parse_int(&value),
                        FamiliarField::RefBase => fam.ref_base = ui::widgets::parse_int(&value),
                        FamiliarField::WillBase => fam.will_base = ui::widgets::parse_int(&value),
                        FamiliarField::Speed => fam.speed = value,
                        FamiliarField::Senses => fam.senses = value,
                        FamiliarField::Attacks => fam.attacks = value,
                        FamiliarField::Granted => fam.granted_ability = value,
                        FamiliarField::Special => fam.special = value,
                        FamiliarField::Notes => fam.notes = value,
                    }
                    self.mark_dirty();
                }
            }
            Message::SetFamiliarSize(size) => {
                if let Some(fam) = self.character.familiar.as_mut() {
                    fam.size = size;
                    self.mark_dirty();
                }
            }
            Message::SetFamiliarAbility(key, value) => {
                let n = ui::widgets::parse_int(&value);
                if let Some(fam) = self.character.familiar.as_mut() {
                    fam.abilities.insert(key, n);
                    self.mark_dirty();
                }
            }
            Message::FamiliarSkillRanks(id, v) => {
                if let Some(fam) = self.character.familiar.as_mut() {
                    fam.skills.entry(id).or_default().ranks = ui::widgets::parse_int(&v);
                    self.mark_dirty();
                }
            }
            Message::FamiliarSkillMisc(id, v) => {
                if let Some(fam) = self.character.familiar.as_mut() {
                    fam.skills.entry(id).or_default().misc = ui::widgets::parse_int(&v);
                    self.mark_dirty();
                }
            }

            Message::CustomSkillAdd(scope) => {
                let uid = self.character.alloc_uid();
                if let Some(list) = self.custom_skills_mut(scope) {
                    list.push(crate::model::character::CustomSkill {
                        uid,
                        name: "New Skill".to_string(),
                        ability: "int".to_string(),
                        ranks: 0,
                        misc: 0,
                        class_skill: false,
                        trained_only: false,
                    });
                    self.mark_dirty();
                }
            }
            Message::CustomSkillRemove(scope, uid) => {
                if let Some(list) = self.custom_skills_mut(scope) {
                    list.retain(|s| s.uid != uid);
                    self.mark_dirty();
                }
            }
            Message::CustomSkillSet(scope, uid, field, value) => {
                if let Some(list) = self.custom_skills_mut(scope) {
                    if let Some(skill) = list.iter_mut().find(|s| s.uid == uid) {
                        match field {
                            CustomSkillField::Name => skill.name = value,
                            CustomSkillField::Ability => skill.ability = value,
                            CustomSkillField::Ranks => {
                                skill.ranks = ui::widgets::parse_int(&value)
                            }
                            CustomSkillField::Misc => skill.misc = ui::widgets::parse_int(&value),
                        }
                        self.mark_dirty();
                    }
                }
            }
            Message::CustomSkillToggle(scope, uid, flag) => {
                if let Some(list) = self.custom_skills_mut(scope) {
                    if let Some(skill) = list.iter_mut().find(|s| s.uid == uid) {
                        match flag {
                            CustomSkillFlag::Class => skill.class_skill = !skill.class_skill,
                            CustomSkillFlag::Trained => skill.trained_only = !skill.trained_only,
                        }
                        self.mark_dirty();
                    }
                }
            }

            Message::GalleryAdd => {
                let paths = persistence::pick_images();
                if !paths.is_empty() {
                    for path in paths {
                        self.character.gallery.push(path.to_string_lossy().to_string());
                    }
                    self.mark_dirty();
                }
            }
            Message::GalleryRemove(index) => {
                if index < self.character.gallery.len() {
                    self.character.gallery.remove(index);
                    self.mark_dirty();
                }
            }
            Message::FeatureExpand(key) => {
                self.expanded_feature = if self.expanded_feature.as_deref() == Some(key.as_str()) {
                    None
                } else {
                    Some(key)
                };
            }
            Message::CombatRefExpand(key) => {
                self.expanded_combat_ref =
                    if self.expanded_combat_ref.as_deref() == Some(key.as_str()) {
                        None
                    } else {
                        Some(key)
                    };
            }

            Message::FeatSearch(v) => self.feat_search = v,
            Message::FeatSelect(v) => self.selected_feat = v,
            Message::FeatAdd(id) => {
                if !self.character.feats.contains(&id) {
                    self.character.feats.push(id);
                    self.mark_dirty();
                }
            }
            Message::FeatRemove(id) => {
                self.character.feats.retain(|f| f != &id);
                self.mark_dirty();
            }

            Message::ToggleBook(book) => {
                self.settings.toggle_book(&book);
                let _ = persistence::save_settings(&self.settings);
            }
            Message::ToggleUnlabeled => {
                self.settings.exclude_unlabeled = !self.settings.exclude_unlabeled;
                let _ = persistence::save_settings(&self.settings);
            }

            Message::NotesAction(action) => {
                let is_edit = action.is_edit();
                self.notes_content.perform(action);
                if is_edit {
                    self.character.notes = self.notes_content.text();
                    self.mark_dirty();
                }
            }
            Message::EditorAction(target, action) => {
                let is_edit = action.is_edit();
                let text = {
                    let content = self
                        .editors
                        .entry(target)
                        .or_insert_with(iced::widget::text_editor::Content::new);
                    content.perform(action);
                    content.text()
                };
                if is_edit {
                    let trimmed = text.strip_suffix('\n').unwrap_or(&text).to_string();
                    self.apply_editor_text(target, trimmed);
                    self.mark_dirty();
                }
            }
            Message::ToggleCondition(name) => {
                toggle_vec(&mut self.character.conditions, name);
                self.mark_dirty();
            }
        }
        self.sync_editors();
        Task::none()
    }

    /// Add a compendium item to inventory. When `charge` is true the item's
    /// price is deducted from the purse and the add aborts if unaffordable;
    /// otherwise it is granted for free.
    fn add_item(&mut self, id: &str, charge: bool) {
        let template = match self.game.item(id) {
            Some(item) => item.clone(),
            None => return,
        };
        if charge && !self.character.coins.spend_gp(template.price) {
            return;
        }
        let uid = self.character.alloc_uid();
        self.character.inventory.push(crate::model::character::InventoryItem {
            uid,
            source_id: Some(template.id),
            name: template.name,
            kind: template.kind,
            quantity: 1,
            weight: template.weight,
            price: template.price,
            equipped: false,
            ac_bonus: template.ac_bonus,
            max_dex: template.max_dex,
            armor_check_penalty: template.armor_check_penalty,
            notes: String::new(),
        });
        self.mark_dirty();
    }

    /// Derived statistics for the active character.
    pub fn derived(&self) -> rules::derived::DerivedStats {
        rules::derived::compute(&self.character)
    }

    fn custom_skills_mut(
        &mut self,
        scope: SkillScope,
    ) -> Option<&mut Vec<crate::model::character::CustomSkill>> {
        match scope {
            SkillScope::Character => Some(&mut self.character.custom_skills),
            SkillScope::Familiar => self.character.familiar.as_mut().map(|f| &mut f.custom_skills),
        }
    }

    fn custom_list_mut(
        &mut self,
        which: CustomList,
    ) -> &mut Vec<crate::model::character::CustomEntry> {
        match which {
            CustomList::Feat => &mut self.character.custom_feats,
            CustomList::Ability => &mut self.character.custom_abilities,
            CustomList::Spell => &mut self.character.custom_spells,
            CustomList::RacialTrait => &mut self.character.racial_traits,
        }
    }

    fn custom_list_ref(&self, which: CustomList) -> &Vec<crate::model::character::CustomEntry> {
        match which {
            CustomList::Feat => &self.character.custom_feats,
            CustomList::Ability => &self.character.custom_abilities,
            CustomList::Spell => &self.character.custom_spells,
            CustomList::RacialTrait => &self.character.racial_traits,
        }
    }

    /// The current model text backing an editor field.
    fn editor_seed_text(&self, target: EditorTarget) -> String {
        match target {
            EditorTarget::FamiliarGranted => self
                .character
                .familiar
                .as_ref()
                .map(|f| f.granted_ability.clone())
                .unwrap_or_default(),
            EditorTarget::FamiliarSpecial => self
                .character
                .familiar
                .as_ref()
                .map(|f| f.special.clone())
                .unwrap_or_default(),
            EditorTarget::FamiliarNotes => self
                .character
                .familiar
                .as_ref()
                .map(|f| f.notes.clone())
                .unwrap_or_default(),
            EditorTarget::CustomDesc(list, uid) => self
                .custom_list_ref(list)
                .iter()
                .find(|e| e.uid == uid)
                .map(|e| e.description.clone())
                .unwrap_or_default(),
            EditorTarget::InventoryNotes(uid) => self
                .character
                .inventory
                .iter()
                .find(|i| i.uid == uid)
                .map(|i| i.notes.clone())
                .unwrap_or_default(),
        }
    }

    /// Write an editor's text back into the model field it edits.
    fn apply_editor_text(&mut self, target: EditorTarget, text: String) {
        match target {
            EditorTarget::FamiliarGranted => {
                if let Some(f) = self.character.familiar.as_mut() {
                    f.granted_ability = text;
                }
            }
            EditorTarget::FamiliarSpecial => {
                if let Some(f) = self.character.familiar.as_mut() {
                    f.special = text;
                }
            }
            EditorTarget::FamiliarNotes => {
                if let Some(f) = self.character.familiar.as_mut() {
                    f.notes = text;
                }
            }
            EditorTarget::CustomDesc(list, uid) => {
                if let Some(entry) = self.custom_list_mut(list).iter_mut().find(|e| e.uid == uid) {
                    entry.description = text;
                }
            }
            EditorTarget::InventoryNotes(uid) => {
                if let Some(item) = self.character.inventory.iter_mut().find(|i| i.uid == uid) {
                    item.notes = text;
                }
            }
        }
    }

    /// Reconcile the editor map with the character: create buffers for new
    /// fields, drop buffers for removed ones, and leave live buffers untouched
    /// so in-progress edits keep their cursor and selection.
    fn sync_editors(&mut self) {
        let mut wanted: Vec<EditorTarget> = Vec::new();
        if self.character.familiar.is_some() {
            wanted.push(EditorTarget::FamiliarGranted);
            wanted.push(EditorTarget::FamiliarSpecial);
            wanted.push(EditorTarget::FamiliarNotes);
        }
        for list in [
            CustomList::Feat,
            CustomList::Ability,
            CustomList::Spell,
            CustomList::RacialTrait,
        ] {
            for entry in self.custom_list_ref(list) {
                wanted.push(EditorTarget::CustomDesc(list, entry.uid));
            }
        }
        for item in &self.character.inventory {
            wanted.push(EditorTarget::InventoryNotes(item.uid));
        }

        self.editors.retain(|target, _| wanted.contains(target));
        for target in wanted {
            if !self.editors.contains_key(&target) {
                let text = self.editor_seed_text(target);
                self.editors
                    .insert(target, iced::widget::text_editor::Content::with_text(&text));
            }
        }
    }
}

fn toggle_vec(vec: &mut Vec<String>, value: String) {
    if let Some(pos) = vec.iter().position(|v| v == &value) {
        vec.remove(pos);
    } else {
        vec.push(value);
    }
}
