//! Consolidates the raw Foundry `packs/` data into compact, typed bundles the
//! app loads at runtime. Run via the `build_data` binary.

use crate::model::compendium::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Known source books, matched as substrings against raw entry JSON.
const BOOKS: &[(&str, &str)] = &[
    ("Core Rulebook", "Core Rulebook"),
    ("Advanced Player", "Advanced Player's Guide"),
    ("Advanced Class Guide", "Advanced Class Guide"),
    ("Advanced Race", "Advanced Race Guide"),
    ("Ultimate Magic", "Ultimate Magic"),
    ("Ultimate Combat", "Ultimate Combat"),
    ("Ultimate Equipment", "Ultimate Equipment"),
    ("Ultimate Wilderness", "Ultimate Wilderness"),
    ("Ultimate Intrigue", "Ultimate Intrigue"),
    ("Occult Adventures", "Occult Adventures"),
    ("Horror Adventures", "Horror Adventures"),
    ("Bestiary", "Bestiary"),
    ("Player Companion", "Player Companion"),
];

/// Witch hexes that are Major Hexes (require witch level 10).
const MAJOR_HEXES: &[&str] = &[
    "Agony",
    "Beast Eye",
    "Cook People",
    "Hidden Home",
    "Ice Tomb",
    "Infected Wounds",
    "Major Healing",
    "Nightmares",
    "Regenerative Sinew",
    "Retribution",
    "Speak in Dreams",
    "Waxen Image",
    "Weather Control",
    "Witch's Charge",
];

/// Witch hexes that are Grand Hexes (require witch level 18).
const GRAND_HEXES: &[&str] = &[
    "Death Curse",
    "Dire Prophecy",
    "Eternal Slumber",
    "Forced Reincarnation",
    "Lay to Rest",
    "Life Giver",
    "Natural Disaster",
    "Summon Spirit",
];

/// Ninja tricks that are Master Tricks (require ninja level 10).
const MASTER_TRICKS: &[&str] = &[
    "All the Stars in the Sky",
    "Assassinate",
    "Breath of the Ancestors",
    "Deadly Range",
    "Fractured Mirror",
    "Ghost Walk",
    "Hidden Master",
    "Invisible Blade",
    "Shadow Clone",
    "Shadow Split",
    "Unbound Steps",
    "Uncanny Ki",
];

/// Run the full consolidation, reading from `packs/` and writing to `data/`.
pub fn run() -> std::io::Result<()> {
    let root = std::env::current_dir()?;
    let packs = root.join("packs");
    let out = root.join("data");
    fs::create_dir_all(&out)?;

    let spells = build_spells(&packs.join("spells"));
    write_json(&out.join("spells.json"), &spells)?;
    println!("spells: {}", spells.len());

    let abilities = build_abilities(&packs.join("class-abilities"));
    write_json(&out.join("abilities.json"), &abilities)?;
    println!("abilities: {}", abilities.len());

    let feats = build_feats(&packs.join("feats"));
    write_json(&out.join("feats.json"), &feats)?;
    println!("feats: {}", feats.len());

    let races = build_races(&packs.join("races"));
    write_json(&out.join("races.json"), &races)?;
    println!("races: {}", races.len());

    let mut items = Vec::new();
    items.extend(build_items(&packs.join("items")));
    items.extend(build_weapons(&packs.join("weapons-and-ammo")));
    items.extend(build_armor(&packs.join("armors-and-shields")));
    write_json(&out.join("items.json"), &items)?;
    println!("items: {}", items.len());

    let classes = build_classes(&packs.join("classes"));
    write_json(&out.join("classes.json"), &classes)?;
    println!("classes: {}", classes.len());

    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let text = serde_json::to_string(value).expect("serialize bundle");
    fs::write(path, text)
}

fn read_dir_json(dir: &Path) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                out.push((slug, value));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// --- text helpers -----------------------------------------------------------

/// Strip HTML to readable plain text, preserving paragraph and list structure.
/// Replaces Foundry VTT text enrichers with human-readable text. References
/// such as `@Compendium[pf1.spells.abc123]{Fireball}` or
/// `@UUID[Compendium.pf1.spells.abc123]{Fireball}` are rendered as just their
/// display label (`Fireball`). When an enricher carries no `{label}`, the raw
/// reference is dropped rather than shown as gibberish.
fn resolve_enrichers(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            if let Some(consumed) = parse_enricher(&input[i..], &mut out) {
                i += consumed;
                continue;
            }
        }
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&input[i..(i + ch_len).min(input.len())]);
        i += ch_len;
    }
    out
}

/// Parses a single `@Type[reference]{label}` enricher at the start of `slice`.
/// On success it appends the resolved text to `out` and returns the number of
/// bytes consumed; otherwise it returns `None` so the caller emits the `@`
/// literally.
fn parse_enricher(slice: &str, out: &mut String) -> Option<usize> {
    let bytes = slice.as_bytes();
    let mut i = 1;
    let name_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    if i == name_start || i >= bytes.len() || bytes[i] != b'[' {
        return None;
    }
    let ref_end = slice[i..].find(']').map(|e| i + e)?;
    let mut consumed = ref_end + 1;
    let label = if slice.as_bytes().get(consumed) == Some(&b'{') {
        let label_start = consumed + 1;
        let label_end = slice[label_start..].find('}').map(|e| label_start + e)?;
        let text = slice[label_start..label_end].to_string();
        consumed = label_end + 1;
        Some(text)
    } else {
        None
    };
    if let Some(text) = label {
        out.push_str(&text);
    }
    Some(consumed)
}

fn utf8_len(first_byte: u8) -> usize {
    match first_byte {
        b if b < 0x80 => 1,
        b if b >> 5 == 0b110 => 2,
        b if b >> 4 == 0b1110 => 3,
        b if b >> 3 == 0b11110 => 4,
        _ => 1,
    }
}

pub fn clean_html(input: &str) -> String {
    let resolved = resolve_enrichers(input);
    let mut out = String::with_capacity(resolved.len());
    let bytes = resolved.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = resolved[i..].find('>').map(|e| i + e).unwrap_or(bytes.len());
            let tag = resolved[i + 1..end.min(resolved.len())].to_lowercase();
            let is_closing = tag.starts_with('/');
            let name: String = tag
                .trim_start_matches('/')
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            match name.as_str() {
                "br" | "p" | "div" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    out.push('\n');
                }
                "li" if !is_closing => out.push_str("\n• "),
                "td" | "th" if !is_closing => out.push_str("  "),
                _ => {}
            }
            i = end + 1;
        } else {
            let ch_len = utf8_len(bytes[i]);
            out.push_str(&resolved[i..(i + ch_len).min(resolved.len())]);
            i += ch_len;
        }
    }
    let decoded = decode_entities(&out);
    normalize_whitespace(&decoded)
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&rsquo;", "'")
        .replace("&lsquo;", "'")
        .replace("&rdquo;", "\"")
        .replace("&ldquo;", "\"")
        .replace("&ndash;", "\u{2013}")
        .replace("&mdash;", "\u{2014}")
        .replace("&times;", "\u{00d7}")
        .replace("&hellip;", "\u{2026}")
        .replace("&frac12;", "\u{00bd}")
        .replace("&deg;", "\u{00b0}")
}

fn normalize_whitespace(s: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for raw in s.split('\n') {
        let collapsed: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        lines.push(collapsed);
    }
    let mut out: Vec<String> = Vec::new();
    let mut blanks = 0;
    for line in lines {
        if line.is_empty() {
            blanks += 1;
            if blanks <= 1 {
                out.push(String::new());
            }
        } else {
            blanks = 0;
            if line.starts_with("• ") && out.last().map(|l| l.is_empty()).unwrap_or(false) {
                out.pop();
            }
            out.push(line);
        }
    }
    out.join("\n").trim().to_string()
}

fn detect_book(raw: &Value) -> String {
    let text = raw.to_string();
    for (needle, canonical) in BOOKS {
        if text.contains(needle) {
            return canonical.to_string();
        }
    }
    String::new()
}

fn description_of(data: &Value) -> String {
    let value = data
        .get("description")
        .and_then(|d| d.get("value"))
        .and_then(|v| v.as_str())
        .or_else(|| data.get("shortDescription").and_then(|v| v.as_str()))
        .unwrap_or("");
    clean_html(value)
}

fn str_field(data: &Value, key: &str) -> String {
    data.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

// --- spells ------------------------------------------------------------------

fn build_spells(dir: &Path) -> Vec<Spell> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };

        let mut class_levels = BTreeMap::new();
        if let Some(classes) = data
            .get("learnedAt")
            .and_then(|l| l.get("class"))
            .and_then(|c| c.as_array())
        {
            for pair in classes {
                if let Some(arr) = pair.as_array() {
                    if arr.len() == 2 {
                        let class = arr[0].as_str().unwrap_or("").to_lowercase();
                        let level = arr[1].as_u64().unwrap_or(0) as u8;
                        if !class.is_empty() {
                            class_levels.insert(class, level);
                        }
                    }
                }
            }
        }
        if class_levels.is_empty() {
            continue;
        }

        let components = data.get("components");
        let comp = |k: &str| {
            components
                .and_then(|c| c.get(k))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        };

        let action = data
            .get("actions")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first());
        let casting_time = action
            .and_then(|a| a.get("activation"))
            .map(format_activation)
            .unwrap_or_default();
        let range = action
            .and_then(|a| a.get("range"))
            .map(format_range)
            .unwrap_or_default();
        let target = action
            .and_then(|a| a.get("target"))
            .and_then(|t| t.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let duration = action
            .and_then(|a| a.get("duration"))
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let save = action
            .and_then(|a| a.get("save"))
            .and_then(|s| s.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let descriptors = str_field(data, "types")
            .split(|c| c == ',' || c == ';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let sr = match data.get("sr") {
            Some(Value::Bool(true)) => "Yes".to_string(),
            Some(Value::Bool(false)) => "No".to_string(),
            _ => String::new(),
        };

        out.push(Spell {
            id: slug.clone(),
            name: name.to_string(),
            school: str_field(data, "school"),
            subschool: str_field(data, "subschool"),
            descriptors,
            verbal: comp("verbal"),
            somatic: comp("somatic"),
            material: comp("material"),
            focus: comp("focus"),
            divine_focus: comp("divineFocus"),
            class_levels,
            casting_time,
            range,
            target,
            duration,
            save,
            spell_resistance: sr,
            description: description_of(data),
            source: detect_book(&raw),
        });
    }
    out
}

fn format_activation(v: &Value) -> String {
    let cost = v.get("cost").and_then(|c| c.as_u64()).unwrap_or(1);
    let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match kind {
        "standard" => "1 standard action".to_string(),
        "swift" => "1 swift action".to_string(),
        "immediate" => "1 immediate action".to_string(),
        "move" => "1 move action".to_string(),
        "full" => "1 full-round action".to_string(),
        "round" => format!("{cost} round(s)"),
        "minute" => format!("{cost} minute(s)"),
        "hour" => format!("{cost} hour(s)"),
        other if !other.is_empty() => format!("{cost} {other}"),
        _ => String::new(),
    }
}

fn format_range(v: &Value) -> String {
    let units = v.get("units").and_then(|u| u.as_str()).unwrap_or("");
    let value = v.get("value").and_then(|x| x.as_str()).unwrap_or("");
    match units {
        "touch" => "Touch".to_string(),
        "personal" => "Personal".to_string(),
        "close" => "Close (25 ft. + 5 ft./2 levels)".to_string(),
        "medium" => "Medium (100 ft. + 10 ft./level)".to_string(),
        "long" => "Long (400 ft. + 40 ft./level)".to_string(),
        "unlimited" => "Unlimited".to_string(),
        "ft" => format!("{value} ft."),
        "" => String::new(),
        other => format!("{value} {other}").trim().to_string(),
    }
}

// --- abilities (hexes, tricks, patrons, features) ---------------------------

fn build_abilities(dir: &Path) -> Vec<Ability> {
    let major: BTreeSet<&str> = MAJOR_HEXES.iter().copied().collect();
    let grand: BTreeSet<&str> = GRAND_HEXES.iter().copied().collect();
    let master: BTreeSet<&str> = MASTER_TRICKS.iter().copied().collect();

    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };

        let mut classes: Vec<String> = Vec::new();
        if let Some(list) = data
            .get("associations")
            .and_then(|a| a.get("classes"))
            .and_then(|c| c.as_array())
        {
            for entry in list {
                let class = if let Some(arr) = entry.as_array() {
                    arr.first().and_then(|v| v.as_str())
                } else {
                    entry.as_str()
                };
                if let Some(class) = class {
                    classes.push(class.to_lowercase());
                }
            }
        }
        if classes.is_empty() {
            continue;
        }

        let is_witch = classes.iter().any(|c| c == "witch");
        let is_ninja = classes.iter().any(|c| c == "ninja");
        let mut themes = Vec::new();

        let category = if name.ends_with("Patron") {
            themes.push(name.trim_end_matches("Patron").trim().to_string());
            AbilityCategory::Patron
        } else if is_witch {
            if grand.contains(name) {
                AbilityCategory::GrandHex
            } else if major.contains(name) {
                AbilityCategory::MajorHex
            } else {
                AbilityCategory::Hex
            }
        } else if is_ninja {
            if master.contains(name) {
                AbilityCategory::MasterTrick
            } else {
                AbilityCategory::NinjaTrick
            }
        } else {
            AbilityCategory::Other
        };

        out.push(Ability {
            id: slug.clone(),
            name: name.to_string(),
            category,
            classes,
            ability_type: str_field(data, "abilityType"),
            themes,
            description: description_of(data),
            source: detect_book(&raw),
        });
    }
    out
}

// --- feats -------------------------------------------------------------------

fn build_feats(dir: &Path) -> Vec<Feat> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };
        let mut types = Vec::new();
        if let Some(tags) = data.get("tags").and_then(|t| t.as_array()) {
            for tag in tags {
                if let Some(arr) = tag.as_array() {
                    if let Some(s) = arr.first().and_then(|v| v.as_str()) {
                        types.push(s.to_string());
                    }
                }
            }
        }
        out.push(Feat {
            id: slug.clone(),
            name: name.to_string(),
            types,
            description: description_of(data),
            source: detect_book(&raw),
        });
    }
    out
}

// --- races -------------------------------------------------------------------

fn build_races(dir: &Path) -> Vec<Race> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };

        let mut ability_mods = BTreeMap::new();
        let mut size = String::from("Medium");
        if let Some(changes) = data.get("changes").and_then(|c| c.as_array()) {
            for change in changes {
                let target = change.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let subtarget = change
                    .get("subTarget")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let formula = change
                    .get("formula")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if target == "ability" {
                    if let Ok(amount) = formula.trim().parse::<i32>() {
                        ability_mods.insert(subtarget.to_string(), amount);
                    }
                }
                if target == "size" {
                    size = crate::model::character::Size::from_data(formula).label().to_string();
                }
            }
        }
        if let Some(s) = data.get("size").and_then(|v| v.as_str()) {
            size = crate::model::character::Size::from_data(s).label().to_string();
        }

        out.push(Race {
            id: slug.clone(),
            name: name.to_string(),
            size,
            ability_mods,
            description: description_of(data),
            source: detect_book(&raw),
        });
    }
    out
}

// --- items -------------------------------------------------------------------

fn num_field(data: &Value, key: &str) -> f64 {
    match data.get(key) {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn classify_gear(data: &Value, item_type: &str) -> ItemKind {
    match item_type {
        "consumable" => ItemKind::Consumable,
        _ => {
            let subtype = str_field(data, "equipmentType");
            if subtype.contains("magic") {
                ItemKind::Magic
            } else {
                ItemKind::Gear
            }
        }
    }
}

fn build_items(dir: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let item_type = raw.get("type").and_then(|v| v.as_str()).unwrap_or("loot");
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };
        out.push(Item {
            id: slug.clone(),
            name: name.to_string(),
            kind: classify_gear(data, item_type),
            subtype: str_field(data, "equipmentType"),
            price: num_field(data, "price"),
            weight: num_field(data, "weight"),
            ac_bonus: 0,
            max_dex: None,
            armor_check_penalty: 0,
            description: description_of(data),
            source: detect_book(&raw),
            weapon: None,
        });
    }
    out
}

fn build_weapons(dir: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };
        out.push(Item {
            id: slug.clone(),
            name: name.to_string(),
            kind: ItemKind::Weapon,
            subtype: str_field(data, "weaponType"),
            price: num_field(data, "price"),
            weight: num_field(data, "weight"),
            ac_bonus: 0,
            max_dex: None,
            armor_check_penalty: 0,
            description: description_of(data),
            source: detect_book(&raw),
            weapon: weapon_stats(data),
        });
    }
    out
}

/// Parse combat statistics from a weapon's primary attack action.
fn weapon_stats(data: &Value) -> Option<WeaponStats> {
    let action = data
        .get("actions")
        .and_then(|a| a.as_array())
        .and_then(|arr| arr.first())?;

    let action_type = action
        .get("actionType")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let range = action.get("range");
    let range_units = range
        .and_then(|r| r.get("units"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let melee = action_type == "mwak" || range_units == "melee" || range_units == "reach";

    let ability = action.get("ability");
    let attack_ability = ability
        .and_then(|a| a.get("attack"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(if melee { "str" } else { "dex" })
        .to_string();
    let damage_ability = ability
        .and_then(|a| a.get("damage"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let crit_range = ability
        .and_then(|a| a.get("critRange"))
        .and_then(|v| v.as_i64())
        .unwrap_or(20) as i32;
    let crit_mult = ability
        .and_then(|a| a.get("critMult"))
        .and_then(|v| v.as_i64())
        .unwrap_or(2) as i32;

    let damage_part = action
        .get("damage")
        .and_then(|d| d.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first());
    let formula = damage_part
        .and_then(|part| part.get(0))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let damage = parse_damage_dice(formula);
    let damage_types = damage_part
        .and_then(|part| part.get(1))
        .and_then(|meta| meta.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let range_str = if melee {
        String::new()
    } else {
        match range.and_then(|r| r.get("value")) {
            Some(Value::Number(n)) => format!("{} ft.", n),
            Some(Value::String(s)) if !s.is_empty() => format!("{s} ft."),
            _ => String::new(),
        }
    };

    Some(WeaponStats {
        damage,
        crit_range,
        crit_mult,
        attack_ability,
        damage_ability,
        melee,
        range: range_str,
        damage_types,
    })
}

/// Extract the base dice from a Foundry damage formula.
///
/// Handles `sizeRoll(1, 8, @size)` (Medium base = `1d8`) and a plain `1d8`.
fn parse_damage_dice(formula: &str) -> String {
    if let Some(start) = formula.find("sizeRoll(") {
        let rest = &formula[start + "sizeRoll(".len()..];
        if let Some(end) = rest.find(')') {
            let args: Vec<&str> = rest[..end].split(',').map(|s| s.trim()).collect();
            if args.len() >= 2 {
                if let (Ok(count), Ok(faces)) =
                    (args[0].parse::<i64>(), args[1].parse::<i64>())
                {
                    return format!("{count}d{faces}");
                }
            }
        }
    }
    // Fall back to a bare NdM token if present.
    for token in formula.split(|c: char| !(c.is_ascii_digit() || c == 'd')) {
        if let Some((count, faces)) = token.split_once('d') {
            if !count.is_empty()
                && !faces.is_empty()
                && count.chars().all(|c| c.is_ascii_digit())
                && faces.chars().all(|c| c.is_ascii_digit())
            {
                return format!("{count}d{faces}");
            }
        }
    }
    String::new()
}

fn build_armor(dir: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };
        let subtype = str_field(data, "equipmentSubtype");
        let kind = if subtype.to_lowercase().contains("shield") {
            ItemKind::Shield
        } else {
            ItemKind::Armor
        };
        let armor = data.get("armor");
        let ac_bonus = armor
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let max_dex = armor
            .and_then(|a| a.get("dex"))
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let acp = armor
            .and_then(|a| a.get("acp"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        out.push(Item {
            id: slug.clone(),
            name: name.to_string(),
            kind,
            subtype,
            price: num_field(data, "price"),
            weight: num_field(data, "weight"),
            ac_bonus,
            max_dex,
            armor_check_penalty: acp,
            description: description_of(data),
            source: detect_book(&raw),
            weapon: None,
        });
    }
    out
}

// --- classes -----------------------------------------------------------------

fn build_classes(dir: &Path) -> Vec<ClassEntry> {
    let mut out = Vec::new();
    for (slug, raw) in read_dir_json(dir) {
        let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(&slug);
        let data = match raw.get("data") {
            Some(d) => d,
            None => continue,
        };
        out.push(ClassEntry {
            id: slug.clone(),
            name: name.to_string(),
            description: description_of(data),
        });
    }
    out
}

/// Convenience for callers that want the output directory.
pub fn output_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_compendium_enricher_to_label() {
        let input = "Gains @Compendium[pf1.spells.sdfsdggsdgsdg]{Ill Omen} as a bonus spell.";
        assert_eq!(
            clean_html(input),
            "Gains Ill Omen as a bonus spell."
        );
    }

    #[test]
    fn resolves_multiple_and_uuid_enrichers() {
        let input = "2nd @UUID[Compendium.pf1.spells.abc]{Alter Self}, @Compendium[pf1.spells.def]{Levitate}";
        assert_eq!(clean_html(input), "2nd Alter Self, Levitate");
    }

    #[test]
    fn drops_unlabeled_enricher() {
        let input = "See @Compendium[pf1.spells.xyz] here.";
        assert_eq!(clean_html(input), "See here.");
    }

    #[test]
    fn parses_size_roll_damage() {
        assert_eq!(parse_damage_dice("sizeRoll(1, 8, @size)"), "1d8");
        assert_eq!(
            parse_damage_dice("sizeRoll(1, 6, @size) + min(@abilities.str.mod, 0)[Strength]"),
            "1d6"
        );
        assert_eq!(parse_damage_dice("2d4"), "2d4");
        assert_eq!(parse_damage_dice(""), "");
    }

    #[test]
    fn weapon_stats_from_action() {
        let data = serde_json::json!({
            "actions": [{
                "actionType": "mwak",
                "range": { "value": 0, "units": "melee" },
                "ability": { "attack": "str", "damage": "str", "critRange": 19, "critMult": 2 },
                "damage": { "parts": [["sizeRoll(1, 8, @size)", { "values": ["slashing"] }]] }
            }]
        });
        let stats = weapon_stats(&data).expect("weapon should parse");
        assert_eq!(stats.damage, "1d8");
        assert_eq!(stats.crit_range, 19);
        assert_eq!(stats.crit_mult, 2);
        assert!(stats.melee);
        assert_eq!(stats.crit_label(), "19-20/x2");
        assert_eq!(stats.damage_types, vec!["slashing".to_string()]);
    }

    #[test]
    fn leaves_bare_at_sign_untouched() {
        let input = "email me @ home and pay @5 gold";
        assert_eq!(clean_html(input), "email me @ home and pay @5 gold");
    }

    #[test]
    fn resolves_enricher_nested_in_html() {
        let input = "<p>Bonus: @Compendium[pf1.spells.q]{Charm Person}</p>";
        assert_eq!(clean_html(input), "Bonus: Charm Person");
    }

    #[test]
    fn list_items_do_not_emit_stray_bullets() {
        let input = "<ul>\n<li>@Compendium[pf1.spells.a]{Enlarge Person}&nbsp;(3rd),</li>\n<li>@Compendium[pf1.spells.b]{Tongues}(7th),</li>\n</ul>";
        assert_eq!(clean_html(input), "• Enlarge Person (3rd),\n• Tongues(7th),");
    }

    #[test]
    fn preserves_multibyte_utf8() {
        let input = "<p>2nd — Cat\u{2019}s Grace, na\u{00ef}ve r\u{00e9}sum\u{00e9}</p>";
        assert_eq!(
            clean_html(input),
            "2nd — Cat\u{2019}s Grace, na\u{00ef}ve r\u{00e9}sum\u{00e9}"
        );
    }
}
