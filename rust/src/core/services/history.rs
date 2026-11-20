//! Gecmis (history) verisi ve islemleri.
//!
//! Python surumundeki `core/services/history_manager.py` karsiligi.

use crate::core::models::HistoryEntry;
use crate::core::{constants, storage};
use chrono::Local;

/// Gecmis kayitlarini yonetir.
#[derive(Debug, Default)]
pub struct HistoryManager {
    history: Vec<HistoryEntry>,
}

impl HistoryManager {
    /// En fazla tutulacak kayit sayisi.
    pub const MAX_ENTRIES: usize = 500;

    /// Kayitli gecmisi diskten yukler.
    pub fn load() -> Self {
        let history: Vec<HistoryEntry> = storage::load(&constants::hist_file());
        Self { history }
    }

    /// Gecmis kayitlarinin salt-okunur listesi.
    pub fn history(&self) -> &[HistoryEntry] {
        &self.history
    }

    /// Yeni bir gecmis kaydi ekler (en basa).
    pub fn add(&mut self, title: &str, url: &str) {
        if url.is_empty() || url.starts_with("file://") || url.starts_with("about:blank") {
            return;
        }
        let entry = HistoryEntry {
            title: if title.trim().is_empty() {
                url.to_string()
            } else {
                title.trim().to_string()
            },
            url: url.to_string(),
            time: Local::now().format("%d.%m.%Y %H:%M").to_string(),
        };
        self.history.insert(0, entry);
        if self.history.len() > Self::MAX_ENTRIES {
            self.history.truncate(Self::MAX_ENTRIES);
        }
        self.save();
    }

    /// Tum gecmisi temizler.
    pub fn clear(&mut self) {
        self.history.clear();
        self.save();
    }

    fn save(&self) {
        storage::save(&constants::hist_file(), &self.history);
    }
}
