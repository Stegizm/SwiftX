//! Veri modelleri — diskte JSON olarak saklanan yapilar.

use serde::{Deserialize, Serialize};

/// Bir yer imi kaydi.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

impl Bookmark {
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
        }
    }
}

/// Bir gecmis kaydi.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub time: String,
}

/// Bir indirme kaydi.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DownloadRecord {
    pub name: String,
    pub path: String,
    pub time: String,
}

/// Kayitli bir eklenti.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Extension {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Varsayilan eklenti listesi.
pub fn default_extensions() -> Vec<Extension> {
    let raw: &[(&str, &str, &str, &str, bool)] = &[
        (
            "dark_reader",
            "Dark Reader",
            "Web sayfalarina dark mode uygula",
            "🌙",
            true,
        ),
        ("no_ads", "No Ads", "Reklam ve izleme engelle", "🚫", true),
        (
            "speedup",
            "Page Speed",
            "Sayfa yuklemesini hizlandir",
            "⚡",
            true,
        ),
        (
            "privacy",
            "Privacy Guard",
            "Izleme ve profil olusturmayi engelle",
            "🔒",
            true,
        ),
        (
            "readmode",
            "Read Mode",
            "Makale modunda okumayi etkinlestir",
            "📖",
            false,
        ),
    ];
    raw.iter()
        .map(|(id, name, desc, icon, enabled)| Extension {
            id: (*id).to_string(),
            name: (*name).to_string(),
            desc: (*desc).to_string(),
            icon: (*icon).to_string(),
            enabled: *enabled,
            version: "1.0.0".to_string(),
        })
        .collect()
}

/// Uygulama ayar bayraklari.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub ad_blocker_enabled: bool,
    #[serde(default = "default_true")]
    pub smooth_scroll: bool,
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default = "default_true")]
    pub restore_session: bool,
    #[serde(default = "default_true")]
    pub tab_hibernate: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ad_blocker_enabled: true,
            smooth_scroll: true,
            dark_mode: true,
            restore_session: true,
            tab_hibernate: true,
        }
    }
}

/// Kaydedilen oturum bilgisi.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Session {
    #[serde(default)]
    pub tabs: Vec<SessionTab>,
    #[serde(default)]
    pub active: usize,
}

/// Oturumdaki tek bir sekme.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SessionTab {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub title: String,
}
