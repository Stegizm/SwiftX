//! Uygulama genelinde kullanilan sabitler ve yapilandirma degerleri.
//!
//! Bu modul, Python surumundeki `core/constants.py` dosyasinin Rust karsiligidir.

use std::path::PathBuf;

/// Uygulama surumu.
pub const VERSION: &str = "0.3.0";

/// Sol kenar cubugunun genisligi (piksel).
pub const SIDEBAR_W: i32 = 48;

/// Sekme rengi secenekleri: (isim, arka plan, on plan).
///
/// Ilk eleman "none" renksiz secenegidir.
pub const TAB_COLORS: &[(&str, &str, &str)] = &[
    ("none", "", ""),
    ("kirmizi", "#e74c3c", "#fff"),
    ("turuncu", "#e67e22", "#fff"),
    ("sari", "#f1c40f", "#222"),
    ("yesil", "#27ae60", "#fff"),
    ("mavi", "#3498db", "#fff"),
    ("mor", "#9b59b6", "#fff"),
];

/// Reklam / izleyici URL kaliplari.
pub const AD_BLOCK_PATTERNS: &[&str] = &[
    "google.com/ads",
    "doubleclick.net",
    "pagead",
    "adsbygoogle",
    "ads.google",
    "googleadservices.com",
    "facebook.com/tr",
    "analytics.google.com",
    "youtube.com/api/stats",
    "amazon-adsystem.com",
    "criteo.com",
    "moatpixel.com",
    "rubiconproject.com",
    "pubmatic.com",
    "gumgum.com",
];

/// Uygulamanin calistirilma turune gore veri dizinini dondurur.
///
/// Oncelik sirasi:
/// 1. `SWIFTX_DATA_DIR` ortam degiskeni (test / tasinabilir kullanim).
/// 2. `XDG_DATA_HOME` veya kullanicinin veri dizini (`~/.local/share`) altinda `swiftx-browser`.
pub fn base_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SWIFTX_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Some(data) = dirs::data_dir() {
        data.join("swiftx-browser")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
            .join("swiftx-browser")
    }
}

/// Kalici veri dizini.
pub fn data_dir() -> PathBuf {
    base_data_dir()
}

/// Veri klasorunun var oldugunu garantiler ve yolunu dondurur.
pub fn ensure_data_dir() -> PathBuf {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Yer imleri dosyasi yolu.
pub fn bm_file() -> PathBuf {
    data_dir().join("bookmarks.json")
}

/// Gecmis dosyasi yolu.
pub fn hist_file() -> PathBuf {
    data_dir().join("history.json")
}

/// Indirmeler dosyasi yolu.
pub fn dl_file() -> PathBuf {
    data_dir().join("downloads.json")
}

/// Oturum dosyasi yolu.
pub fn session_file() -> PathBuf {
    data_dir().join("session.json")
}

/// Eklentiler dosyasi yolu.
pub fn extensions_file() -> PathBuf {
    data_dir().join("extensions.json")
}

/// Ayarlar dosyasi yolu.
pub fn settings_file() -> PathBuf {
    data_dir().join("settings.json")
}
