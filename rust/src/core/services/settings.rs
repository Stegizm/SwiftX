//! Uygulama ayarlarini merkezi olarak yonetir.
//!
//! Python surumundeki `core/services/settings_manager.py` karsiligi.
//! Ek olarak, orijinalde eksik olan **ayar kaliciligi** burada saglanir
//! (`settings.json`).

use crate::core::models::{default_extensions, Extension, Settings};
use crate::core::{constants, storage};

/// Ayarlari ve eklentileri tutar; her degisiklikte diske kaydeder.
#[derive(Debug)]
pub struct SettingsManager {
    settings: Settings,
    extensions: Vec<Extension>,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            extensions: default_extensions(),
        }
    }
}

impl SettingsManager {
    /// Ayarlari ve eklentileri diskten yukler.
    pub fn load() -> Self {
        let settings: Settings = storage::load(&constants::settings_file());
        let ext_path = constants::extensions_file();
        let extensions: Vec<Extension> = if ext_path.exists() {
            storage::load(&ext_path)
        } else {
            let defaults = default_extensions();
            storage::save(&ext_path, &defaults);
            defaults
        };
        Self {
            settings,
            extensions,
        }
    }

    /// Ayar bayraklarinin salt-okunur hali.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Ada gore ayar degerini dondurur.
    pub fn flag(&self, name: &str) -> Option<bool> {
        match name {
            "ad_blocker_enabled" => Some(self.settings.ad_blocker_enabled),
            "smooth_scroll" => Some(self.settings.smooth_scroll),
            "dark_mode" => Some(self.settings.dark_mode),
            "restore_session" => Some(self.settings.restore_session),
            "tab_hibernate" => Some(self.settings.tab_hibernate),
            _ => None,
        }
    }

    /// Verilen ayari tersine cevirir, yeni degeri dondurur ve kaydeder.
    pub fn toggle(&mut self, name: &str) -> Option<bool> {
        let new_value = match name {
            "ad_blocker_enabled" => {
                self.settings.ad_blocker_enabled = !self.settings.ad_blocker_enabled;
                self.settings.ad_blocker_enabled
            }
            "smooth_scroll" => {
                self.settings.smooth_scroll = !self.settings.smooth_scroll;
                self.settings.smooth_scroll
            }
            "dark_mode" => {
                self.settings.dark_mode = !self.settings.dark_mode;
                self.settings.dark_mode
            }
            "restore_session" => {
                self.settings.restore_session = !self.settings.restore_session;
                self.settings.restore_session
            }
            "tab_hibernate" => {
                self.settings.tab_hibernate = !self.settings.tab_hibernate;
                self.settings.tab_hibernate
            }
            _ => return None,
        };
        self.save_settings();
        Some(new_value)
    }

    /// Bir ayari acikca ayarlar.
    pub fn set_flag(&mut self, name: &str, value: bool) {
        match name {
            "ad_blocker_enabled" => self.settings.ad_blocker_enabled = value,
            "smooth_scroll" => self.settings.smooth_scroll = value,
            "dark_mode" => self.settings.dark_mode = value,
            "restore_session" => self.settings.restore_session = value,
            "tab_hibernate" => self.settings.tab_hibernate = value,
            _ => return,
        }
        self.save_settings();
    }

    /// Eklenti listesinin salt-okunur hali.
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Verilen eklenti etkin mi?
    pub fn extension_enabled(&self, id: &str) -> bool {
        self.extensions
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.enabled)
            .unwrap_or(false)
    }

    /// Eklenti degisikliklerini uygular ve kaydeder.
    pub fn apply_extension_changes(&mut self, changes: &[(String, bool)]) {
        for (id, enabled) in changes {
            if let Some(ext) = self.extensions.iter_mut().find(|e| &e.id == id) {
                ext.enabled = *enabled;
            }
        }
        self.save_extensions();
    }

    /// Etkin eklenti sayisi.
    pub fn active_extension_count(&self) -> usize {
        self.extensions.iter().filter(|e| e.enabled).count()
    }

    /// Toplam eklenti sayisi.
    pub fn total_extension_count(&self) -> usize {
        self.extensions.len()
    }

    fn save_settings(&self) {
        storage::save(&constants::settings_file(), &self.settings);
    }

    fn save_extensions(&self) {
        storage::save(&constants::extensions_file(), &self.extensions);
    }
}
