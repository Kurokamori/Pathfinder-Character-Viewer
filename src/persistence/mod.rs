//! Disk persistence: characters, settings, autosave, and import/export.

use crate::model::character::Character;
use crate::model::settings::Settings;
use std::collections::HashMap;
use std::io::{Read, Write};
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

/// Root for imported image assets, one subdirectory per character id.
pub fn images_dir() -> PathBuf {
    app_dir().join("images")
}

fn character_images_dir(id: u64) -> PathBuf {
    images_dir().join(id.to_string())
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
    let images = character_images_dir(id);
    if images.exists() {
        let _ = std::fs::remove_dir_all(images);
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

/// Prompt for a location and export the character as a self-contained bundle.
///
/// The bundle is a zip archive holding `character.json` alongside every
/// referenced image under `images/`. Image path fields inside the JSON are
/// rewritten to those archive-relative names so the character can be moved
/// between machines. A plain `.json` export (portraits by absolute path) is
/// still offered for callers that only want the sheet data.
pub fn export_character(character: &Character) -> std::io::Result<bool> {
    let default_name = format!("{}.pfchar", sanitize(&character.name));
    let picked = rfd::FileDialog::new()
        .set_title("Export character")
        .set_file_name(&default_name)
        .add_filter("Character bundle", &["pfchar"])
        .add_filter("Character JSON", &["json"])
        .save_file();
    let path = match picked {
        Some(path) => path,
        None => return Ok(false),
    };

    if path.extension().and_then(|s| s.to_str()) == Some("json") {
        let text = serde_json::to_string_pretty(character)?;
        std::fs::write(path, text)?;
        return Ok(true);
    }

    let mut doc = character.clone();
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();

    if let Some(portrait) = doc.portrait.clone() {
        if let Some(internal) = bundle_image(&portrait, "portrait", &mut seen, &mut images) {
            doc.portrait = Some(internal);
        }
    }
    if let Some(familiar) = doc.familiar.as_mut() {
        if let Some(portrait) = familiar.portrait.clone() {
            if let Some(internal) = bundle_image(&portrait, "familiar", &mut seen, &mut images) {
                familiar.portrait = Some(internal);
            }
        }
    }
    for index in 0..doc.gallery.len() {
        let source = doc.gallery[index].clone();
        let base = format!("gallery_{index}");
        if let Some(internal) = bundle_image(&source, &base, &mut seen, &mut images) {
            doc.gallery[index] = internal;
        }
    }

    let json = serde_json::to_string_pretty(&doc)?;
    write_bundle(&path, &json, &images)?;
    Ok(true)
}

/// Prompt for a file and import a character, assigning it a fresh id.
///
/// Accepts either a `.pfchar`/`.zip` bundle (images are extracted into this
/// machine's data directory and path fields rewritten to absolute paths) or a
/// legacy `.json` sheet.
pub fn import_character() -> std::io::Result<Option<Character>> {
    let picked = rfd::FileDialog::new()
        .set_title("Import character")
        .add_filter("Character bundle", &["pfchar", "zip"])
        .add_filter("Character JSON", &["json"])
        .pick_file();
    let path = match picked {
        Some(path) => path,
        None => return Ok(None),
    };

    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());
    if extension.as_deref() == Some("json") {
        let mut character = load_character(&path)?;
        character.id = new_id();
        return Ok(Some(character));
    }

    let character = import_bundle(&path)?;
    Ok(Some(character))
}

/// Read an image from disk and register it for inclusion in the bundle,
/// returning the archive-relative name to store in the character JSON.
///
/// Images that share a source path reuse a single archive entry. If the source
/// cannot be read the original reference is left untouched so no data is lost.
fn bundle_image(
    source: &str,
    base: &str,
    seen: &mut HashMap<String, String>,
    images: &mut Vec<(String, Vec<u8>)>,
) -> Option<String> {
    if let Some(existing) = seen.get(source) {
        return Some(existing.clone());
    }
    let bytes = std::fs::read(source).ok()?;
    let internal = format!("images/{}.{}", base, image_extension(source));
    seen.insert(source.to_string(), internal.clone());
    images.push((internal.clone(), bytes));
    Some(internal)
}

fn image_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| "png".to_string())
}

fn write_bundle(path: &Path, json: &str, images: &[(String, Vec<u8>)]) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("character.json", options).map_err(to_io)?;
    zip.write_all(json.as_bytes())?;
    for (name, bytes) in images {
        zip.start_file(name.as_str(), options).map_err(to_io)?;
        zip.write_all(bytes)?;
    }
    zip.finish().map_err(to_io)?;
    Ok(())
}

fn import_bundle(path: &Path) -> std::io::Result<Character> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(to_io)?;

    let mut json = String::new();
    archive
        .by_name("character.json")
        .map_err(to_io)?
        .read_to_string(&mut json)?;
    let mut character: Character = serde_json::from_str(&json).map_err(std::io::Error::from)?;
    character.id = new_id();

    let dest_dir = character_images_dir(character.id);
    std::fs::create_dir_all(&dest_dir)?;

    let mut placed: HashMap<String, String> = HashMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(to_io)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let relative = match name.strip_prefix("images/") {
            Some(relative) if !relative.is_empty() => relative,
            _ => continue,
        };
        let file_name = match Path::new(relative).file_name().and_then(|s| s.to_str()) {
            Some(file_name) => file_name.to_string(),
            None => continue,
        };
        let out_path = dest_dir.join(&file_name);
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        std::fs::write(&out_path, &bytes)?;
        placed.insert(name, out_path.to_string_lossy().to_string());
    }

    if let Some(portrait) = character.portrait.clone() {
        if let Some(absolute) = placed.get(&portrait) {
            character.portrait = Some(absolute.clone());
        }
    }
    if let Some(familiar) = character.familiar.as_mut() {
        if let Some(portrait) = familiar.portrait.clone() {
            if let Some(absolute) = placed.get(&portrait) {
                familiar.portrait = Some(absolute.clone());
            }
        }
    }
    for slot in character.gallery.iter_mut() {
        if let Some(absolute) = placed.get(slot.as_str()) {
            *slot = absolute.clone();
        }
    }

    Ok(character)
}

fn to_io<E: std::error::Error + Send + Sync + 'static>(error: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, error)
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
