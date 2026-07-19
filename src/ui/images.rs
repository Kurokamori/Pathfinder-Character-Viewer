//! Lazy, content-sniffed image handle cache.
//!
//! iced's [`Handle::from_path`] decodes an image by trusting its file
//! extension. Art saved from the web is frequently a WebP (or PNG) carrying a
//! misleading `.jpg` extension, so extension-based decoding picks the wrong
//! codec, fails, and the widget renders blank. Reading the bytes ourselves and
//! building the handle from them makes iced sniff the true format via
//! `image::load_from_memory` instead.
//!
//! [`Handle::from_bytes`] mints a fresh id on every call, so a naive
//! `from_bytes` inside a view would force the renderer to re-decode and
//! re-upload the texture on every redraw. This cache loads each path exactly
//! once and hands back the same handle thereafter, letting the renderer keep
//! its uploaded texture.

use iced::widget::image::Handle;
use std::cell::RefCell;
use std::collections::HashMap;

/// Path-keyed store of decoded image handles. `None` records a path that could
/// not be read so it is not retried on every redraw.
#[derive(Default)]
pub struct ImageCache {
    entries: RefCell<HashMap<String, Option<Handle>>>,
}

impl ImageCache {
    /// The cached handle for `path`, loading and caching it on first request.
    /// Returns `None` when the file cannot be read.
    pub fn handle(&self, path: &str) -> Option<Handle> {
        if let Some(existing) = self.entries.borrow().get(path) {
            return existing.clone();
        }
        let loaded = std::fs::read(path).ok().map(Handle::from_bytes);
        self.entries
            .borrow_mut()
            .insert(path.to_string(), loaded.clone());
        loaded
    }

    /// Drop every cached handle, e.g. when switching characters.
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
    }
}
