pub mod loader;

use crate::model::compendium::*;
use std::collections::HashMap;

/// Indexed, queryable view over the loaded compendium.
pub struct GameData {
    pub compendium: Compendium,
    spell_index: HashMap<String, usize>,
    ability_index: HashMap<String, usize>,
    item_index: HashMap<String, usize>,
    feat_index: HashMap<String, usize>,
    race_index: HashMap<String, usize>,
    sources: Vec<String>,
}

impl GameData {
    pub fn new(compendium: Compendium) -> Self {
        let spell_index = compendium
            .spells
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
        let ability_index = compendium
            .abilities
            .iter()
            .enumerate()
            .map(|(i, a)| (a.id.clone(), i))
            .collect();
        let item_index = compendium
            .items
            .iter()
            .enumerate()
            .map(|(i, it)| (it.id.clone(), i))
            .collect();
        let feat_index = compendium
            .feats
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id.clone(), i))
            .collect();
        let race_index = compendium
            .races
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id.clone(), i))
            .collect();
        let sources = compendium.sources();
        Self {
            compendium,
            spell_index,
            ability_index,
            item_index,
            feat_index,
            race_index,
            sources,
        }
    }

    pub fn spell(&self, id: &str) -> Option<&Spell> {
        self.spell_index.get(id).map(|&i| &self.compendium.spells[i])
    }

    pub fn ability(&self, id: &str) -> Option<&Ability> {
        self.ability_index
            .get(id)
            .map(|&i| &self.compendium.abilities[i])
    }

    pub fn item(&self, id: &str) -> Option<&Item> {
        self.item_index.get(id).map(|&i| &self.compendium.items[i])
    }

    pub fn feat(&self, id: &str) -> Option<&Feat> {
        self.feat_index.get(id).map(|&i| &self.compendium.feats[i])
    }

    pub fn race(&self, id: &str) -> Option<&Race> {
        self.race_index.get(id).map(|&i| &self.compendium.races[i])
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Spells learnable by a class tag, filtered by the settings' book rules.
    pub fn spells_for_class<'a>(
        &'a self,
        class_tag: &'a str,
        settings: &'a crate::model::settings::Settings,
    ) -> impl Iterator<Item = (&'a Spell, u8)> {
        self.compendium.spells.iter().filter_map(move |s| {
            if !settings.allows(&s.source) {
                return None;
            }
            s.level_for(class_tag).map(|lvl| (s, lvl))
        })
    }
}
