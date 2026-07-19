//! Loads the consolidated compendium bundles produced by `build_data`.

use crate::model::compendium::*;
use std::path::{Path, PathBuf};

/// Locate the `data/` directory next to the executable or in the project root.
pub fn resolve_data_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("data"));
            if let Some(up) = dir.parent() {
                candidates.push(up.join("data"));
                if let Some(up2) = up.parent() {
                    candidates.push(up2.join("data"));
                }
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("data"));
    }
    candidates.into_iter().find(|p| p.join("spells.json").exists())
}

fn read_bundle<T: serde::de::DeserializeOwned>(dir: &Path, file: &str) -> Result<T, String> {
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("reading {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parsing {file}: {e}"))
}

/// Load the full compendium from the resolved data directory.
pub fn load() -> Result<Compendium, String> {
    let dir = resolve_data_dir()
        .ok_or_else(|| "Could not find the data/ directory. Run `cargo run --bin build_data` first.".to_string())?;
    Ok(Compendium {
        spells: read_bundle(&dir, "spells.json")?,
        abilities: read_bundle(&dir, "abilities.json")?,
        feats: read_bundle(&dir, "feats.json")?,
        races: read_bundle(&dir, "races.json")?,
        items: read_bundle(&dir, "items.json")?,
        classes: read_bundle(&dir, "classes.json")?,
    })
}
