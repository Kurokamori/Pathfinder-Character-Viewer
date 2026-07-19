//! Disk persistence: characters, settings, autosave, and import/export.

use crate::model::character::Character;
use crate::model::settings::Settings;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Application data root under the OS's per-user data directory.
pub fn app_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PathfinderViewer")
}

pub fn characters_dir() -> PathBuf {
    app_dir().join("characters")
}

fn settings_path() -> PathBuf {
    app_dir().join("settings.json")
}

fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(characters_dir())
}

/// A lightweight listing entry for the character picker.
#[derive(Debug, Clone)]
pub struct CharacterSummary {
    pub id: u64,
    pub name: String,
    pub class_line: String,
    pub path: PathBuf,
}

/// Allocate a fresh, unique character id.
pub fn new_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
}

/// List all saved characters, newest first.
pub fn list_characters() -> Vec<CharacterSummary> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(characters_dir()) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(character) = load_character(&path) {
            let class_line = character
                .classes
                .iter()
                .map(|c| format!("{} {}", title_case(&c.tag), c.level))
                .collect::<Vec<_>>()
                .join(" / ");
            out.push(CharacterSummary {
                id: character.id,
                name: character.name,
                class_line,
                path,
            });
        }
    }
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out
}

pub fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn character_path(id: u64) -> PathBuf {
    characters_dir().join(format!("{id}.json"))
}

/// Write a character to its canonical location.
pub fn save_character(character: &Character) -> std::io::Result<()> {
    ensure_dirs()?;
    let text = serde_json::to_string_pretty(character)?;
    std::fs::write(character_path(character.id), text)
}

/// Read a character document from an explicit path.
pub fn load_character(path: &Path) -> std::io::Result<Character> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(std::io::Error::from)
}

/// Read a character by its id.
pub fn load_character_by_id(id: u64) -> std::io::Result<Character> {
    load_character(&character_path(id))
}

pub fn delete_character(id: u64) -> std::io::Result<()> {
    let path = character_path(id);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn load_settings() -> Settings {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> std::io::Result<()> {
    ensure_dirs()?;
    let text = serde_json::to_string_pretty(settings)?;
    std::fs::write(settings_path(), text)
}

/// Prompt for a location and export the character as JSON.
pub fn export_character(character: &Character) -> std::io::Result<bool> {
    let default_name = format!("{}.json", sanitize(&character.name));
    let picked = rfd::FileDialog::new()
        .set_title("Export character")
        .set_file_name(&default_name)
        .add_filter("Character JSON", &["json"])
        .save_file();
    match picked {
        Some(path) => {
            let text = serde_json::to_string_pretty(character)?;
            std::fs::write(path, text)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Prompt for a file and import a character, assigning it a fresh id.
pub fn import_character() -> std::io::Result<Option<Character>> {
    let picked = rfd::FileDialog::new()
        .set_title("Import character")
        .add_filter("Character JSON", &["json"])
        .pick_file();
    match picked {
        Some(path) => {
            let mut character = load_character(&path)?;
            character.id = new_id();
            Ok(Some(character))
        }
        None => Ok(None),
    }
}

/// Prompt for an image file to use as a portrait.
pub fn pick_portrait() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose portrait")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .pick_file()
}

/// Prompt for one or more images to add to the gallery.
pub fn pick_images() -> Vec<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Add art to gallery")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .pick_files()
        .unwrap_or_default()
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "character".to_string()
    } else {
        trimmed.to_string()
    }
}
