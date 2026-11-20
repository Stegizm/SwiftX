//! JSON tabanli kalici veri okuma/yazma yardimcilari.
//!
//! Python surumundeki `core/storage.py` dosyasinin karsiligi.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;

use super::constants;

/// Veri klasorunun var oldugunu garantiler.
pub fn ensure_data_dir() {
    constants::ensure_data_dir();
}

/// JSON dosyasini yukler; herhangi bir hata durumunda `default` degerini dondurur.
pub fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// Veriyi JSON dosyasina yazar (okunabilir, 2 bosluk girintili).
pub fn save<T: Serialize>(path: &Path, data: &T) {
    ensure_data_dir();
    if let Ok(text) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(path, text);
    }
}
