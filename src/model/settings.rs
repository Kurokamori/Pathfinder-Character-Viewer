//! Persisted application settings (independent of any single character).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Source books excluded from browsing/selection.
    pub excluded_books: BTreeSet<String>,
    /// Also hide entries with no known source when true.
    pub exclude_unlabeled: bool,
    /// Id of the character to reopen on launch.
    pub last_character: Option<u64>,
}

impl Settings {
    /// Whether an entry from the given source should be shown.
    pub fn allows(&self, source: &str) -> bool {
        if source.is_empty() {
            !self.exclude_unlabeled
        } else {
            !self.excluded_books.contains(source)
        }
    }

    pub fn toggle_book(&mut self, book: &str) {
        if self.excluded_books.contains(book) {
            self.excluded_books.remove(book);
        } else {
            self.excluded_books.insert(book.to_string());
        }
    }
}
