//! Chrome (arayuz webview) ile Rust cekirdegi arasindaki mesaj protokolu.

use serde::Deserialize;
use std::path::PathBuf;
use wry::PermissionKind;

/// Chrome arayuzunden gelen komutlar.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum ChromeMsg {
    /// Arayuz hazir oldu.
    Ready,
    /// Yeni sekme ac.
    NewTab {
        #[serde(default)]
        url: Option<String>,
    },
    /// Sekmeyi kapat.
    CloseTab { index: usize },
    /// Aktif sekmeyi kapat.
    CloseCurrent,
    /// Tam ekrani ac/kapat.
    ToggleFullscreen,
    /// Sekmeye gec.
    SwitchTab { index: usize },
    /// Adres cubugundan git / ara.
    Navigate { text: String },
    /// Geri.
    Back,
    /// Ileri.
    Forward,
    /// Yenile.
    Reload,
    /// Ana sayfa.
    Home,
    /// Kenar cubugunu ac/kapat.
    ToggleSidebar,
    /// Adres cubuguna odaklan (kenar cubugu "Ara" butonu).
    FocusUrl,
    /// Mevcut sayfayi yer imine ekle.
    AddBookmark,
    /// Yer imini kaldir.
    RemoveBookmark { title: String },
    /// Gecmisi goster.
    ToggleHistory,
    /// Indirmeleri goster.
    ToggleDownloads,
    /// Ayarlari goster.
    ToggleSettings,
    /// Eklenti merkezini goster.
    OpenExtensions,
    /// Paneli kapat.
    ClosePanel,
    /// Paneli temizle.
    ClearPanel,
    /// Gecmisi tamamen temizle.
    ClearHistory,
    /// URL ac.
    OpenUrl { url: String },
    /// Arama yap.
    Search { text: String },
    /// Sekme sag-tik menu eylemi.
    TabContext {
        index: usize,
        action: String,
        #[serde(default)]
        color: Option<String>,
    },
    /// Ayar ac/kapat.
    ToggleSetting { name: String },
    /// Eklenti degisikliklerini kaydet.
    SaveExtensions { changes: Vec<ExtensionChange> },
    /// Yazdir.
    Print,
    /// Yakinlastir.
    ZoomIn,
    /// Uzaklastir.
    ZoomOut,
    /// Yakinlastirmayi sifirla.
    ZoomReset,
    /// Sayfa icinde arama cubugunu ac.
    FindOpen,
    /// Sayfada metin ara.
    Find { text: String },
    /// Sonraki eslesme.
    FindNext,
    /// Onceki eslesme.
    FindPrev,
    /// Aramayi kapat.
    FindClose,
    /// Gelistirici araclarini ac/kapat.
    ToggleDevtools,
    /// Yer imlerini disa aktar.
    ExportBookmarks,
    /// Yer imlerini ice aktar.
    ImportBookmarks,
}

/// Eklenti degisikligi.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionChange {
    pub id: String,
    pub enabled: bool,
}

/// Olay dongusune gonderilen kullanici olaylari.
#[derive(Debug)]
pub enum UserEvent {
    /// Chrome IPC mesaji.
    Chrome(ChromeMsg),
    /// Sekme basligi degisti.
    Title { tab: u64, title: String },
    /// Sekme URL'i degisti.
    Url { tab: u64, url: String },
    /// Sayfa yuklenmeye basladi.
    LoadStarted { tab: u64 },
    /// Sayfa yuklendi.
    LoadFinished { tab: u64 },
    /// Indirme basladi (url, hedef dosya).
    DownloadStarted { url: String, path: PathBuf },
    /// Indirme tamamlandi.
    DownloadFinished {
        url: String,
        path: Option<PathBuf>,
        ok: bool,
    },
    /// Yeni pencere istegi (yeni sekmede acilir).
    NewWindow { tab: u64, url: String },
    /// Izin istendi.
    Permission { kind: PermissionKind },
    /// Tam ekran istendi/cikildi.
    Fullscreen(bool),
    /// Uyutma denetimi icin zamanlayici tetiklendi.
    HibernateTick,
    /// Pencere yeniden boyutlandirildi.
    Resized,
    /// Indirme cubugunu gizle (zamanli).
    HideDownloadBar,
    /// Ilerleme cubugunu gizle (zamanli).
    HideProgress,
    /// Durum cubugunda gecici mesaj goster.
    Status(String),
    /// Tema/ayar durumunu chrome'a yeniden gonder.
    RefreshState,
}
