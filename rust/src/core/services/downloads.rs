//! Indirme (download) verisi ve islemleri.
//!
//! Python surumundeki `core/services/download_manager.py` karsiligi.
//! Gercek indirme akisi `engine` katmaninda WebKit sinyalleriyle baglanir;
//! bu modul yalnizca kayit tutar ve indirme klasorunu belirler.

use crate::core::models::DownloadRecord;
use crate::core::{constants, storage};
use chrono::Local;
use std::path::PathBuf;

/// Indirme kayitlarini yonetir.
#[derive(Debug, Default)]
pub struct DownloadManager {
    downloads: Vec<DownloadRecord>,
}

impl DownloadManager {
    /// Kayitli indirmeleri diskten yukler.
    pub fn load() -> Self {
        let downloads: Vec<DownloadRecord> = storage::load(&constants::dl_file());
        Self { downloads }
    }

    /// Indirme kayitlarinin salt-okunur listesi.
    pub fn downloads(&self) -> &[DownloadRecord] {
        &self.downloads
    }

    /// Indirme klasorunu dondurur (`~/Indirilenler` veya `~/Downloads`).
    pub fn download_dir() -> PathBuf {
        if let Some(dir) = dirs::download_dir() {
            if dir.exists() {
                return dir;
            }
        }
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let indirilenler = home.join("Indirilenler");
        if indirilenler.exists() {
            indirilenler
        } else {
            home.join("Downloads")
        }
    }

    /// Yeni bir indirme kaydi ekler (en basa).
    pub fn add(&mut self, name: &str, path: &str) {
        let entry = DownloadRecord {
            name: name.to_string(),
            path: path.to_string(),
            time: Local::now().format("%d.%m.%Y %H:%M").to_string(),
        };
        self.downloads.insert(0, entry);
        self.save();
    }

    /// Tum indirme kayitlarini temizler.
    pub fn clear(&mut self) {
        self.downloads.clear();
        self.save();
    }

    fn save(&self) {
        storage::save(&constants::dl_file(), &self.downloads);
    }
}
