//! Yer imleri (bookmark) verisi ve islemleri.
//!
//! Python surumundeki `core/services/bookmark_manager.py` karsiligi.

use crate::core::models::Bookmark;
use crate::core::{constants, storage};

/// Yer imlerini yonetir; degisiklikte diske kaydeder.
#[derive(Debug, Default)]
pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
}

impl BookmarkManager {
    /// Kayitli yer imlerini diskten yukler.
    pub fn load() -> Self {
        let bookmarks: Vec<Bookmark> = storage::load(&constants::bm_file());
        Self { bookmarks }
    }

    /// Yer imlerinin salt-okunur listesi.
    pub fn bookmarks(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    /// Yeni yer imi ekler.
    ///
    /// `Err` ile Turkce hata mesaji dondurur (kullaniciya gosterilir).
    pub fn add(&mut self, url: &str, title: &str) -> Result<(), String> {
        if url.is_empty() || url.starts_with("file://") || url.starts_with("about:blank") {
            return Err("Bu sayfa yer imine eklenemez.".to_string());
        }
        if self.bookmarks.iter().any(|b| b.url == url) {
            return Err("Bu sayfa zaten yer imlerinde.".to_string());
        }
        let title = if title.trim().is_empty() {
            url.to_string()
        } else {
            title.trim().to_string()
        };
        self.bookmarks.push(Bookmark::new(title, url));
        self.save();
        Ok(())
    }

    /// Basliga gore yer imini siler.
    pub fn remove(&mut self, title: &str) {
        self.bookmarks.retain(|b| b.title != title);
        self.save();
    }

    /// URL'e gore yer imini siler.
    pub fn remove_by_url(&mut self, url: &str) {
        self.bookmarks.retain(|b| b.url != url);
        self.save();
    }

    /// Tum yer imlerini siler.
    pub fn clear_all(&mut self) {
        self.bookmarks.clear();
        self.save();
    }

    fn save(&self) {
        storage::save(&constants::bm_file(), &self.bookmarks);
    }
}
