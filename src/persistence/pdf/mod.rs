//! PDF character-sheet export.
//!
//! Renders the character as a real text-based PDF document rather than a screen
//! capture, so the result stays selectable, searchable and sharp in print. The
//! section order follows the viewer's tab order.

mod layout;
mod sections;

use super::{sanitize, to_io};
use crate::data::GameData;
use crate::model::character::Character;
use crate::rules::derived;
use layout::Pdf;

/// Prompt for a destination and write the character sheet as a PDF.
///
/// Returns `Ok(false)` when the user cancels the file dialog.
pub fn export_pdf(character: &Character, game: &GameData) -> std::io::Result<bool> {
    let default_name = format!("{}.pdf", sanitize(&character.name));
    let picked = rfd::FileDialog::new()
        .set_title("Export character sheet as PDF")
        .set_file_name(&default_name)
        .add_filter("PDF document", &["pdf"])
        .save_file();
    let path = match picked {
        Some(path) => path,
        None => return Ok(false),
    };

    let document = build(character, game).map_err(to_io)?;
    document.save(&path).map_err(to_io)?;
    Ok(true)
}

/// Lay out every section of the sheet into a finished document.
fn build(character: &Character, game: &GameData) -> Result<Pdf, printpdf::Error> {
    let stats = derived::compute(character);

    let name = if character.name.trim().is_empty() {
        "Unnamed Character"
    } else {
        character.name.trim()
    };
    let classes = sections::class_line(character);
    let footer = if classes.is_empty() {
        name.to_string()
    } else {
        format!("{name} - {classes}")
    };

    let mut pdf = Pdf::new(&format!("{name} - Character Sheet"), &footer)?;

    sections::identity(&mut pdf, character);
    sections::abilities(&mut pdf, character);
    sections::combat(&mut pdf, character, &stats);
    sections::skills(&mut pdf, character, &stats);
    sections::feats(&mut pdf, character, game);
    sections::racial_traits(&mut pdf, character);
    sections::languages(&mut pdf, character);
    sections::class_abilities(&mut pdf, character, game);
    sections::spells(&mut pdf, character, game);
    sections::familiar(&mut pdf, character);
    sections::inventory(&mut pdf, character, &stats);
    sections::resources(&mut pdf, character);
    sections::narrative(&mut pdf, character);
    sections::notes(&mut pdf, character);

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{Coins, CustomEntry, InventoryItem};
    use crate::model::compendium::{Compendium, ItemKind};

    fn sample() -> Character {
        let mut character = Character::new(1, "Kaelen Voss", "witch");
        character.race = "Human".to_string();
        character.player = "Test".to_string();
        character.alignment = "NG".to_string();
        character.languages = "Common\nAbyssal\nDraconic".to_string();
        character.notes = "Owes a favour to the harbourmaster.".to_string();
        character.backstory = "Raised by the coven in the marshes. ".repeat(60);
        character.conditions = vec!["Fatigued".to_string()];
        character.coins = Coins {
            pp: 0,
            gp: 0,
            sp: 11,
            cp: 0,
        };
        character.skill_mut("spl").ranks = 5;
        character.custom_feats.push(CustomEntry {
            uid: 1,
            name: "Marsh Strider".to_string(),
            level: 0,
            description: "Ignore difficult terrain in swamps. ".repeat(20),
        });
        character.racial_traits.push(CustomEntry {
            uid: 2,
            name: "Skilled".to_string(),
            level: 0,
            description: "One extra skill rank per level.".to_string(),
        });
        character.inventory.push(InventoryItem {
            uid: 3,
            source_id: None,
            name: "Dagger".to_string(),
            kind: ItemKind::Weapon,
            quantity: 2,
            weight: 1.0,
            price: 2.0,
            equipped: true,
            ac_bonus: 0,
            max_dex: None,
            armor_check_penalty: 0,
            notes: String::new(),
            slot: None,
        });
        character
    }

    /// The exporter must produce a structurally valid, multi-page PDF without
    /// touching the file dialog.
    #[test]
    fn builds_a_valid_pdf() {
        let game = GameData::new(Compendium::default());
        let path = std::env::temp_dir().join("pathfinder_viewer_pdf_test.pdf");

        build(&sample(), &game)
            .expect("document builds")
            .save(&path)
            .expect("document saves");

        let bytes = std::fs::read(&path).expect("output readable");
        let _ = std::fs::remove_file(&path);

        assert!(bytes.starts_with(b"%PDF-"), "missing PDF header");
        assert!(bytes.len() > 2000, "suspiciously small output");

        // `/Page` also matches the leading bytes of the `/Pages` tree node, so
        // discount those to count real page objects.
        let page_refs = bytes.windows(5).filter(|w| *w == b"/Page").count();
        let page_trees = bytes.windows(6).filter(|w| *w == b"/Pages").count();
        let pages = page_refs.saturating_sub(page_trees);
        assert!(
            pages > 1,
            "long content should paginate, got {pages} page object(s)"
        );
    }

    /// A brand-new character has almost no data; empty sections must not panic
    /// or emit a zero-page document.
    #[test]
    fn handles_an_empty_character() {
        let game = GameData::new(Compendium::default());
        let path = std::env::temp_dir().join("pathfinder_viewer_pdf_empty.pdf");

        build(&Character::new(2, "", ""), &game)
            .expect("document builds")
            .save(&path)
            .expect("document saves");

        let bytes = std::fs::read(&path).expect("output readable");
        let _ = std::fs::remove_file(&path);
        assert!(bytes.starts_with(b"%PDF-"));
    }

    /// Long unbroken tokens must still be split rather than run off the page.
    #[test]
    fn wraps_overlong_words() {
        let long = "x".repeat(400);
        let lines = layout::wrap(&long, 180.0, 9.0, layout::Style::Regular);
        assert!(lines.len() > 1, "long token was not split");
        for line in lines {
            assert!(layout::text_width(&line, 9.0, layout::Style::Regular) <= 180.5);
        }
    }
}
