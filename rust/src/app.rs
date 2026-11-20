//! Koordinator: pencere, chrome arayuzu, sekmeler ve servisleri bir arada tutar.
//!
//! Python surumundeki `windows/main_window.py` karsiligi.

use crate::core::constants;
use crate::core::models::{Session, SessionTab};
use crate::core::services::{BookmarkManager, DownloadManager, HistoryManager, SettingsManager};
use crate::core::updater::{self, UpdateInfo};
use crate::engine::{self, WebConfig};
use crate::ipc::{ChromeMsg, ExtensionChange, UserEvent};
use crate::ui;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::event_loop::EventLoopProxy;
use tao::window::{Fullscreen, Window};
use wry::{WebContext, WebView, WebViewBuilder};

/// Sekmelerin uyutulmasi icin gereken hareketsizlik suresi (saniye).
const HIBERNATE_IDLE_SECONDS: u64 = 300;

/// Acik sag panel turu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelKind {
    History,
    Downloads,
    Settings,
    Extensions,
}

/// Tek bir sekme.
struct Tab {
    id: u64,
    webview: WebView,
    title: String,
    url: String,
    saved_url: String,
    hibernated: bool,
    color: String,
    last_active: Instant,
    zoom: f64,
}

/// Uygulama durumu ve olay isleyicisi.
pub struct App {
    window: Window,
    chrome: WebView,
    context: WebContext,
    proxy: EventLoopProxy<UserEvent>,
    blocker: Arc<AtomicBool>,
    tabs: Vec<Tab>,
    active: usize,
    next_id: u64,
    bookmarks: BookmarkManager,
    history: HistoryManager,
    downloads: DownloadManager,
    settings: SettingsManager,
    sidebar_open: bool,
    panel: Option<PanelKind>,
    dl_visible: bool,
    dl_text: String,
    progress_visible: bool,
    progress_value: f64,
    /// Fiziksel pencere boyutu (device pixels) ve olcek carpani.
    win_phys: (f64, f64),
    scale: f64,
    fullscreen: bool,
    quitting: bool,
    last_find: String,
    debug: bool,
    calibrated: bool,
    last_state: String,
    update: Option<UpdateInfo>,
}

impl App {
    /// Uygulamayi olusturur: chrome arayuzu + ilk sekme/oturum.
    pub fn new(window: Window, proxy: EventLoopProxy<UserEvent>) -> Self {
        let mut context = WebContext::new(Some(constants::data_dir().join("webview")));
        let size = window.inner_size();
        let win_phys = (size.width as f64, size.height as f64);
        if std::env::var("SWIFTX_DEBUG").is_ok() {
            eprintln!(
                "[SwiftX scale] inner_size={}x{} window_scale={}",
                size.width,
                size.height,
                window.scale_factor()
            );
        }

        let selftest = std::env::var("SWIFTX_SELFTEST").is_ok();
        let debug = std::env::var("SWIFTX_DEBUG").is_ok();

        let mut chrome_builder = WebViewBuilder::new_with_web_context(&mut context)
            .with_html(ui::CHROME_HTML)
            .with_bounds(engine::rect(0.0, 0.0, win_phys.0, win_phys.1))
            .with_background_color((28, 27, 34, 255))
            .with_initialization_script(crate::engine::scripts::HOTKEYS_JS);
        if selftest {
            chrome_builder =
                chrome_builder.with_initialization_script(crate::engine::scripts::SELFTEST_JS);
        }
        let chrome = chrome_builder
            .with_ipc_handler({
                let proxy = proxy.clone();
                move |req: wry::http::Request<String>| {
                    if let Ok(msg) = serde_json::from_str::<ChromeMsg>(req.body()) {
                        let _ = proxy.send_event(UserEvent::Chrome(msg));
                    }
                }
            })
            .build_as_child(&window)
            .expect("chrome arayuzu olusturulamadi");

        // wry, set_bounds sirasinda boyutlari webview'in kendi olcek carpaniyla
        // "logical" birime cevirir. Bu yuzden yerlesim hesaplarinda tao penceresinin
        // degil, web gorunumunun olcek carpanini kullanmaliyiz.
        let scale = webview_scale(&chrome, &window);
        if std::env::var("SWIFTX_DEBUG").is_ok() {
            eprintln!(
                "[SwiftX scale] webview_scale={scale} css_viewport={}x{}",
                win_phys.0 / scale,
                win_phys.1 / scale
            );
        }

        let settings = SettingsManager::load();
        let blocker = Arc::new(AtomicBool::new(settings.settings().ad_blocker_enabled));

        let mut app = Self {
            window,
            chrome,
            context,
            proxy: proxy.clone(),
            blocker,
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
            bookmarks: BookmarkManager::load(),
            history: HistoryManager::load(),
            downloads: DownloadManager::load(),
            settings,
            sidebar_open: true,
            panel: None,
            dl_visible: false,
            dl_text: String::new(),
            progress_visible: false,
            progress_value: 0.0,
            win_phys,
            scale,
            fullscreen: false,
            quitting: false,
            last_find: String::new(),
            calibrated: false,
            last_state: String::new(),
            update: None,
            debug,
        };

        // Uyutma zamanlayicisi (30 sn).
        let tick_proxy = proxy.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(30));
            if tick_proxy.send_event(UserEvent::HibernateTick).is_err() {
                break;
            }
        });

        // Acilista guncelleme kontrolu (arka planda, sessiz).
        let update_proxy = proxy;
        std::thread::spawn(move || match updater::check() {
            Ok(Some(info)) => {
                let _ = update_proxy.send_event(UserEvent::UpdateAvailable(Box::new(info)));
            }
            Ok(None) => {}
            Err(_) => {}
        });

        app.load_session_or_home();
        app.send_state();
        app
    }

    /// Guncellemeyi arka planda kontrol eder ve sonucu bildirir.
    fn check_updates_async(&self) {
        let proxy = self.proxy.clone();
        std::thread::spawn(move || match updater::check() {
            Ok(Some(info)) => {
                let _ = proxy.send_event(UserEvent::UpdateAvailable(Box::new(info)));
            }
            Ok(None) => {
                let _ = proxy.send_event(UserEvent::UpdateNone);
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::UpdateError(e));
            }
        });
    }

    /// Bulunan guncellemeyi indirir, uygular ve cikar.
    fn install_update(&mut self) {
        let Some(info) = self.update.clone() else {
            self.notify("Guncelleme bulunamadi.".to_string());
            return;
        };
        self.notify(format!("⬇ Guncelleme indiriliyor: v{}", info.version));
        self.send_state();
        let proxy = self.proxy.clone();
        std::thread::spawn(move || match updater::download_and_apply(&info) {
            Ok(()) => {
                let _ = proxy.send_event(UserEvent::UpdateReady);
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::UpdateError(e));
            }
        });
    }

    fn config(&self) -> WebConfig {
        let s = self.settings.settings();
        WebConfig {
            smooth_scroll: s.smooth_scroll,
            dark_mode: s.dark_mode || self.settings.extension_enabled("dark_reader"),
            ad_block: s.ad_blocker_enabled || self.settings.extension_enabled("no_ads"),
            privacy: self.settings.extension_enabled("privacy"),
            speedup: self.settings.extension_enabled("speedup"),
            read_mode: self.settings.extension_enabled("readmode"),
        }
    }

    fn content_bounds(&self) -> wry::Rect {
        let (w, h) = self.win_phys;
        let s = self.scale;
        let top = ui::TOPBAR_H * s;
        let left = if self.sidebar_open {
            constants::SIDEBAR_W as f64 * s
        } else {
            0.0
        };
        let right = if self.panel.is_some() {
            ui::PANEL_W * s
        } else {
            0.0
        };
        let bottom = if self.dl_visible {
            ui::DLBAR_H * s
        } else {
            0.0
        };
        engine::rect(
            left,
            top,
            (w - left - right).max(1.0),
            (h - top - bottom).max(1.0),
        )
    }

    /// Yerlesimi gunceller (pencere boyutu / panel / cubuk degisimlerinde).
    pub fn layout(&self) {
        let _ = self
            .chrome
            .set_bounds(engine::rect(0.0, 0.0, self.win_phys.0, self.win_phys.1));
        let bounds = self.content_bounds();
        for (i, tab) in self.tabs.iter().enumerate() {
            let _ = tab.webview.set_bounds(bounds);
            let _ = tab.webview.set_visible(i == self.active && !tab.hibernated);
        }
    }

    fn current(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    fn load_session_or_home(&mut self) {
        if self.settings.settings().restore_session {
            let session: Session = crate::core::storage::load(&constants::session_file());
            if !session.tabs.is_empty() {
                for st in &session.tabs {
                    let url = if st.url.is_empty() {
                        engine::home_url()
                    } else {
                        st.url.clone()
                    };
                    // Geri yuklenen sekmeler tembel baslatilir: yalnizca aktif
                    // sekme gercek sayfayi yukler, digerleri uyutulmus baslar.
                    self.new_tab_at(Some(url), false, true);
                }
                let active = session.active.min(self.tabs.len().saturating_sub(1));
                self.switch_tab(active);
                return;
            }
        }
        self.new_tab_at(None, false, false);
    }

    fn new_tab_at(&mut self, url: Option<String>, switch: bool, lazy: bool) {
        let id = self.next_id;
        self.next_id += 1;
        let url = url.unwrap_or_else(engine::home_url);
        let cfg = self.config();
        let bounds = self.content_bounds();
        let build_url = if lazy {
            "about:blank".to_string()
        } else {
            url.clone()
        };
        let t0 = Instant::now();
        match engine::build_tab(
            &self.window,
            &mut self.context,
            &cfg,
            id,
            &build_url,
            bounds,
            self.proxy.clone(),
            self.blocker.clone(),
        ) {
            Ok(webview) => {
                let _ = webview.set_visible(false);
                if lazy {
                    let _ = webview.load_html(crate::engine::scripts::HIBERNATE_HTML);
                }
                let title = if lazy {
                    "Yeni Sekme".to_string()
                } else if url.starts_with("file://") {
                    "Ana Sayfa".to_string()
                } else {
                    "Yeni Sekme".to_string()
                };
                self.tabs.push(Tab {
                    id,
                    webview,
                    title,
                    url: url.clone(),
                    saved_url: url,
                    hibernated: lazy,
                    color: String::new(),
                    last_active: Instant::now(),
                    zoom: 1.0,
                });
                if switch || self.tabs.len() == 1 {
                    self.switch_tab(self.tabs.len() - 1);
                }
                if self.debug {
                    eprintln!(
                        "[SwiftX timing] sekme{} webview olusturma: {:?}",
                        id,
                        t0.elapsed()
                    );
                }
            }
            Err(e) => self.notify(format!("Sekme acilamadi: {e}")),
        }
    }

    fn switch_tab(&mut self, index: usize) {
        if self.tabs.is_empty() {
            return;
        }
        let index = index.min(self.tabs.len() - 1);
        self.active = index;
        if self.tabs[index].hibernated {
            self.wake_tab(index);
        }
        for (i, tab) in self.tabs.iter().enumerate() {
            let _ = tab.webview.set_visible(i == index);
        }
        let _ = self.tabs[index].webview.focus();
        self.tabs[index].last_active = Instant::now();
    }

    fn close_tab(&mut self, index: usize) {
        if self.tabs.is_empty() {
            return;
        }
        let index = index.min(self.tabs.len() - 1);
        let tab = self.tabs.remove(index);
        let _ = tab.webview.set_visible(false);
        drop(tab.webview);
        if self.tabs.is_empty() {
            self.active = 0;
            self.new_tab_at(None, true, false);
            return;
        }
        if index < self.active {
            self.active -= 1;
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.switch_tab(self.active);
    }

    fn navigate(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let url = normalize_url(text);
        self.open_url(&url);
    }

    fn open_url(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            let _ = tab.webview.load_url(url);
        }
    }

    fn go_back(&self) {
        if let Some(tab) = self.current() {
            let _ = tab.webview.evaluate_script("window.history.back();");
        }
    }

    fn go_forward(&self) {
        if let Some(tab) = self.current() {
            let _ = tab.webview.evaluate_script("window.history.forward();");
        }
    }

    fn reload(&self) {
        if let Some(tab) = self.current() {
            let _ = tab.webview.reload();
        }
    }

    fn go_home(&mut self) {
        let home = engine::home_url();
        self.open_url(&home);
    }

    fn add_bookmark(&mut self) {
        let Some(tab) = self.current() else { return };
        let url = tab.url.clone();
        let title = tab.title.clone();
        match self.bookmarks.add(&url, &title) {
            Ok(()) => self.notify(format!("'{title}' yer imlerine eklendi.")),
            Err(e) => self.notify(e),
        }
    }

    fn remove_bookmark(&mut self, title: &str) {
        self.bookmarks.remove(title);
        self.notify("Yer imi kaldirildi.".to_string());
    }

    fn notify(&mut self, text: String) {
        self.dl_text = format!("ℹ  {text}");
        self.dl_visible = true;
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(3));
            let _ = proxy.send_event(UserEvent::HideDownloadBar);
        });
    }

    fn toggle_setting(&mut self, name: &str) {
        let Some(value) = self.settings.toggle(name) else {
            return;
        };
        if name == "ad_blocker_enabled" {
            self.blocker.store(value, Ordering::Relaxed);
        }
        let durum = if value { "Acik" } else { "Kapali" };
        let label = match name {
            "ad_blocker_enabled" => "Reklam Bloker",
            "smooth_scroll" => "Smooth Scroll",
            "dark_mode" => "Auto Dark Mode",
            "restore_session" => "Session Recovery",
            "tab_hibernate" => "Sekme Uyutma",
            _ => name,
        };
        self.notify(format!("{label} {durum}"));
    }

    fn apply_extensions(&mut self, changes: &[ExtensionChange]) {
        let mapped: Vec<(String, bool)> =
            changes.iter().map(|c| (c.id.clone(), c.enabled)).collect();
        self.settings.apply_extension_changes(&mapped);
        let msg = format!(
            "{} eklenti aktif, {} devre disi",
            self.settings.active_extension_count(),
            self.settings.total_extension_count() - self.settings.active_extension_count()
        );
        self.notify(msg);
    }

    fn hibernate_tab(&mut self, index: usize) {
        if index == self.active || index >= self.tabs.len() {
            return;
        }
        if self.tabs[index].hibernated {
            return;
        }
        self.tabs[index].saved_url = self.tabs[index].url.clone();
        let _ = self.tabs[index]
            .webview
            .load_html(crate::engine::scripts::HIBERNATE_HTML);
        self.tabs[index].hibernated = true;
        let _ = self.tabs[index].webview.set_visible(false);
    }

    fn wake_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || !self.tabs[index].hibernated {
            return;
        }
        let url = self.tabs[index].saved_url.clone();
        let _ = self.tabs[index].webview.load_url(&url);
        self.tabs[index].url = url;
        self.tabs[index].hibernated = false;
    }

    fn check_hibernation(&mut self) {
        if !self.settings.settings().tab_hibernate || self.tabs.len() <= 1 {
            return;
        }
        let now = Instant::now();
        let mut targets = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            if i != self.active
                && !tab.hibernated
                && now.duration_since(tab.last_active).as_secs() >= HIBERNATE_IDLE_SECONDS
            {
                targets.push(i);
            }
        }
        for i in targets {
            self.hibernate_tab(i);
        }
    }

    fn set_zoom(&mut self, delta: f64) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            let z = (tab.zoom + delta).clamp(0.4, 3.0);
            tab.zoom = z;
            let _ = tab.webview.zoom(z);
        }
    }

    fn reset_zoom(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.zoom = 1.0;
            let _ = tab.webview.zoom(1.0);
        }
    }

    fn eval_find(&self, text: &str, forward: bool) {
        if let Some(tab) = self.current() {
            let arg = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
            let js = format!("window.__swiftx_find && window.__swiftx_find({arg}, {forward});");
            let _ = tab.webview.evaluate_script(&js);
        }
    }

    fn find_open(&mut self) {
        let _ = self.chrome.focus();
        let _ = self.chrome.evaluate_script("window.swiftx.openFind();");
    }

    fn find_close(&mut self) {
        self.last_find.clear();
        let _ = self.chrome.evaluate_script("window.swiftx.closeFind();");
        if let Some(tab) = self.current() {
            let _ = tab
                .webview
                .evaluate_script("window.__swiftx_find_clear && window.__swiftx_find_clear();");
        }
    }

    fn toggle_devtools(&mut self) {
        if let Some(tab) = self.current() {
            if tab.webview.is_devtools_open() {
                tab.webview.close_devtools();
            } else {
                tab.webview.open_devtools();
            }
        }
    }

    fn export_bookmarks(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Yer imlerini disa aktar")
            .set_file_name("swiftx-bookmarks.json")
            .add_filter("JSON", &["json"])
            .save_file()
        else {
            return;
        };
        crate::core::storage::save(&path, &self.bookmarks.bookmarks().to_vec());
        self.notify(format!("Yer imleri disa aktarildi: {}", path.display()));
    }

    fn import_bookmarks(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Yer imlerini ice aktar")
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        let imported: Vec<crate::core::models::Bookmark> = crate::core::storage::load(&path);
        let mut count = 0;
        for b in imported {
            if self.bookmarks.add(&b.url, &b.title).is_ok() {
                count += 1;
            }
        }
        self.notify(format!("{count} yer imi ice aktarildi"));
    }

    fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        if self.fullscreen {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
            self.notify("Tam ekran modu (F11 ile cik)".to_string());
        } else {
            self.window.set_fullscreen(None);
        }
    }

    fn set_panel(&mut self, kind: Option<PanelKind>) {
        self.panel = kind;
        self.layout();
    }

    /// Oturumu diske kaydeder.
    pub fn save_session(&mut self) {
        if self.quitting {
            return;
        }
        self.quitting = true;
        let tabs: Vec<SessionTab> = self
            .tabs
            .iter()
            .map(|t| SessionTab {
                url: if t.hibernated {
                    t.saved_url.clone()
                } else {
                    t.url.clone()
                },
                title: t.title.clone(),
            })
            .collect();
        let session = Session {
            tabs,
            active: self.active,
        };
        crate::core::storage::save(&constants::session_file(), &session);
    }

    /// Pencere yeniden boyutlandirildi.
    pub fn resized(&mut self) {
        let size = self.window.inner_size();
        self.win_phys = (size.width as f64, size.height as f64);
        // Olcek carpani baslangicta kalibre edilir; yeniden boyutlandirmada
        // yeniden olcmeye gerek yoktur.
        self.layout();
        self.send_state();
    }

    /// Chrome arayuzune guncel durumu iter.
    pub fn send_state(&mut self) {
        let state = self.build_state();
        let json = serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string());
        // Durum degismediyse chrome'a yeniden gonderp gereksiz DOM yenilemeyi onle.
        if json == self.last_state {
            return;
        }
        self.last_state = json.clone();
        let js = format!("window.swiftx && window.swiftx.setState({json});");
        let _ = self.chrome.evaluate_script(&js);
    }

    fn build_state(&self) -> serde_json::Value {
        let tabs: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                json!({
                    "title": t.title,
                    "active": i == self.active,
                    "color": t.color,
                    "hibernated": t.hibernated,
                })
            })
            .collect();

        let url = self.current().map(|t| t.url.clone()).unwrap_or_default();

        let bookmarks: Vec<_> = self
            .bookmarks
            .bookmarks()
            .iter()
            .map(|b| json!({ "title": b.title, "url": b.url }))
            .collect();

        let (panel_title, items) = match self.panel {
            Some(PanelKind::History) => {
                let items: Vec<_> = self
                    .history
                    .history()
                    .iter()
                    .map(|h| {
                        json!({
                            "label": format!("{}  —  {}", h.time, h.title),
                            "meta": h.url,
                            "url": h.url,
                        })
                    })
                    .collect();
                ("Gecmis".to_string(), items)
            }
            Some(PanelKind::Downloads) => {
                let items: Vec<_> = self
                    .downloads
                    .downloads()
                    .iter()
                    .map(|d| {
                        json!({
                            "label": d.name,
                            "meta": format!("{}  —  {}", d.time, d.path),
                            "url": "",
                        })
                    })
                    .collect();
                ("Indirilenler".to_string(), items)
            }
            Some(PanelKind::Settings) => ("Ayarlar".to_string(), Vec::new()),
            Some(PanelKind::Extensions) => ("Eklenti Merkezi".to_string(), Vec::new()),
            None => (String::new(), Vec::new()),
        };

        let s = self.settings.settings();
        let extensions: Vec<_> = self
            .settings
            .extensions()
            .iter()
            .map(|e| {
                json!({
                    "id": e.id, "name": e.name, "desc": e.desc,
                    "icon": e.icon, "enabled": e.enabled, "version": e.version,
                })
            })
            .collect();

        json!({
            "tabs": tabs,
            "url": url,
            "bookmarks": bookmarks,
            "panel": {
                "open": self.panel.is_some(),
                "kind": self.panel.map(panel_kind_str).unwrap_or(""),
                "title": panel_title,
                "items": items,
            },
            "settings": {
                "ad_blocker_enabled": s.ad_blocker_enabled,
                "smooth_scroll": s.smooth_scroll,
                "dark_mode": s.dark_mode,
                "restore_session": s.restore_session,
                "tab_hibernate": s.tab_hibernate,
            },
            "extensions": extensions,
            "sidebar_open": self.sidebar_open,
            "progress": { "visible": self.progress_visible, "value": self.progress_value },
            "dlbar": { "visible": self.dl_visible, "text": self.dl_text },
            "version": constants::VERSION,
            "update": {
                "available": self.update.is_some(),
                "version": self.update.as_ref().map(|u| u.version.clone()).unwrap_or_default(),
            },
        })
    }

    /// Kullanici olayini isler.
    pub fn handle(&mut self, event: UserEvent) {
        match event {
            UserEvent::Chrome(msg) => self.handle_chrome(msg),
            UserEvent::Title { tab, title } => {
                if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab) {
                    if !t.hibernated {
                        t.title = if title.trim().is_empty() {
                            "Yeni Sekme".to_string()
                        } else {
                            title
                        };
                    }
                }
            }
            UserEvent::Url { tab, url } => {
                if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab) {
                    if !t.hibernated {
                        t.url = url;
                    }
                }
            }
            UserEvent::LoadStarted { tab } => {
                if self.current().map(|t| t.id) == Some(tab) {
                    self.progress_visible = true;
                    self.progress_value = 45.0;
                }
            }
            UserEvent::LoadFinished { tab } => {
                if self.current().map(|t| t.id) == Some(tab) {
                    self.progress_value = 100.0;
                }
                if let Some(t) = self.tabs.iter().find(|t| t.id == tab) {
                    if !t.hibernated {
                        let (title, url) = (t.title.clone(), t.url.clone());
                        self.history.add(&title, &url);
                        if self.debug {
                            eprintln!("[SwiftX timing] yuklendi tab={tab} url={url}");
                        }
                    }
                }
                let proxy = self.proxy.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(400));
                    let _ = proxy.send_event(UserEvent::HideProgress);
                });
            }
            UserEvent::DownloadStarted { url, path } => {
                let name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "indirme".to_string());
                let _ = url;
                self.dl_text = format!("⬇  Indiriliyor: {name}");
                self.dl_visible = true;
                self.layout();
            }
            UserEvent::DownloadFinished { path, ok, .. } => {
                let name = path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "indirme".to_string());
                if ok {
                    if let Some(p) = path.as_ref() {
                        self.downloads.add(&name, &p.to_string_lossy());
                    }
                    self.dl_text = format!("✓  Indirildi: {name}");
                } else {
                    self.dl_text = format!("✕  Indirme basarisiz: {name}");
                }
                let proxy = self.proxy.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(4));
                    let _ = proxy.send_event(UserEvent::HideDownloadBar);
                });
            }
            UserEvent::NewWindow { url, .. } => {
                self.new_tab_at(Some(url), true, false);
            }
            UserEvent::Permission { kind } => {
                self.notify(format!("✓ {} izni verildi", permission_str(kind)));
            }
            UserEvent::Fullscreen(active) => {
                if active != self.fullscreen {
                    self.toggle_fullscreen();
                }
            }
            UserEvent::HibernateTick => {
                self.check_hibernation();
            }
            UserEvent::Resized => self.resized(),
            UserEvent::ChromeMetrics { css_w, css_h } => {
                if css_w > 1.0 {
                    let new_scale = self.win_phys.0 / css_w;
                    if (new_scale - self.scale).abs() > 0.001 {
                        if self.debug {
                            eprintln!(
                                "[SwiftX scale] kalibre edildi: css={}x{} scale={}",
                                css_w, css_h, new_scale
                            );
                        }
                        self.scale = new_scale;
                    }
                }
                self.calibrated = true;
                self.layout();
            }
            UserEvent::HideDownloadBar => {
                self.dl_visible = false;
                self.layout();
            }
            UserEvent::HideProgress => {
                self.progress_visible = false;
                self.progress_value = 0.0;
            }
            UserEvent::Status(text) => self.notify(text),
            UserEvent::RefreshState => {}
            UserEvent::UpdateAvailable(info) => {
                let version = info.version.clone();
                self.update = Some(*info);
                self.notify(format!("⬆ Yeni surum mevcut: v{version}"));
            }
            UserEvent::UpdateNone => {
                self.notify("Guncelleme yok; en son surumdesiniz.".to_string());
            }
            UserEvent::UpdateError(e) => {
                self.notify(format!("Guncelleme hatasi: {e}"));
            }
            UserEvent::UpdateReady => {
                self.save_session();
                self.send_state();
                std::process::exit(0);
            }
        }
        self.send_state();
    }

    fn handle_chrome(&mut self, msg: ChromeMsg) {
        if self.debug {
            eprintln!("[SwiftX debug] chrome msg: {msg:?}");
        }
        match msg {
            ChromeMsg::Ready => {
                if self.debug {
                    eprintln!("[SwiftX debug] chrome ready");
                }
                if !self.calibrated {
                    let proxy = self.proxy.clone();
                    let _ = self.chrome.evaluate_script_with_callback(
                        "JSON.stringify([window.innerWidth, window.innerHeight])",
                        move |res| {
                            let trimmed = res.trim_matches('"');
                            if let Ok(v) = serde_json::from_str::<Vec<f64>>(trimmed) {
                                if v.len() == 2 && v[0] > 1.0 {
                                    let _ = proxy.send_event(UserEvent::ChromeMetrics {
                                        css_w: v[0],
                                        css_h: v[1],
                                    });
                                }
                            }
                        },
                    );
                }
            }
            ChromeMsg::NewTab { url } => self.new_tab_at(url, true, false),
            ChromeMsg::CloseTab { index } => self.close_tab(index),
            ChromeMsg::CloseCurrent => self.close_tab(self.active),
            ChromeMsg::SwitchTab { index } => self.switch_tab(index),
            ChromeMsg::Navigate { text } => self.navigate(&text),
            ChromeMsg::Back => self.go_back(),
            ChromeMsg::Forward => self.go_forward(),
            ChromeMsg::Reload => self.reload(),
            ChromeMsg::Home => self.go_home(),
            ChromeMsg::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                self.layout();
            }
            ChromeMsg::FocusUrl => {
                let _ = self.chrome.focus();
                let _ = self.chrome.evaluate_script("window.swiftx.focusUrl();");
            }
            ChromeMsg::AddBookmark => self.add_bookmark(),
            ChromeMsg::RemoveBookmark { title } => self.remove_bookmark(&title),
            ChromeMsg::ToggleHistory => self.toggle_panel(PanelKind::History),
            ChromeMsg::ToggleDownloads => self.toggle_panel(PanelKind::Downloads),
            ChromeMsg::ToggleSettings => self.toggle_panel(PanelKind::Settings),
            ChromeMsg::OpenExtensions => self.toggle_panel(PanelKind::Extensions),
            ChromeMsg::ClosePanel => self.set_panel(None),
            ChromeMsg::ClearPanel => self.clear_panel(),
            ChromeMsg::ClearHistory => {
                self.history.clear();
                self.downloads.clear();
                self.notify("Gecmis ve indirilenler temizlendi.".to_string());
            }
            ChromeMsg::OpenUrl { url } => self.open_url(&url),
            ChromeMsg::Search { text } => self.navigate(&text),
            ChromeMsg::TabContext {
                index,
                action,
                color,
            } => self.tab_context(index, &action, color.as_deref()),
            ChromeMsg::ToggleSetting { name } => self.toggle_setting(&name),
            ChromeMsg::SaveExtensions { changes } => self.apply_extensions(&changes),
            ChromeMsg::Print => {
                if let Some(t) = self.current() {
                    let _ = t.webview.print();
                }
            }
            ChromeMsg::ZoomIn => self.set_zoom(0.1),
            ChromeMsg::ZoomOut => self.set_zoom(-0.1),
            ChromeMsg::ZoomReset => self.reset_zoom(),
            ChromeMsg::FindOpen => self.find_open(),
            ChromeMsg::Find { text } => {
                self.last_find = text.clone();
                self.eval_find(&text, true);
            }
            ChromeMsg::FindNext => {
                let text = self.last_find.clone();
                self.eval_find(&text, true);
            }
            ChromeMsg::FindPrev => {
                let text = self.last_find.clone();
                self.eval_find(&text, false);
            }
            ChromeMsg::FindClose => self.find_close(),
            ChromeMsg::ToggleDevtools => self.toggle_devtools(),
            ChromeMsg::ExportBookmarks => self.export_bookmarks(),
            ChromeMsg::ImportBookmarks => self.import_bookmarks(),
            ChromeMsg::SelfTest { data } => {
                eprintln!("[SwiftX selftest] {data}");
            }
            ChromeMsg::Probe { data } => {
                eprintln!("[SwiftX probe] {data}");
            }
            ChromeMsg::CheckUpdates => self.check_updates_async(),
            ChromeMsg::InstallUpdate => self.install_update(),
            ChromeMsg::OpenReleasePage => {
                if let Some(u) = self.update.clone() {
                    self.new_tab_at(Some(u.page_url), true, false);
                }
            }
            ChromeMsg::ToggleFullscreen => self.toggle_fullscreen(),
        }
    }

    fn toggle_panel(&mut self, kind: PanelKind) {
        self.panel = if self.panel == Some(kind) {
            None
        } else {
            Some(kind)
        };
        self.layout();
    }

    fn clear_panel(&mut self) {
        match self.panel {
            Some(PanelKind::History) => self.history.clear(),
            Some(PanelKind::Downloads) => self.downloads.clear(),
            _ => {}
        }
    }

    fn tab_context(&mut self, index: usize, action: &str, color: Option<&str>) {
        if index >= self.tabs.len() {
            return;
        }
        match action {
            "close" => self.close_tab(index),
            "copy" => {
                let url = self.tabs[index].url.clone();
                self.notify(format!("Kopyalandi: {url}"));
            }
            "color" => {
                if let Some(name) = color {
                    let hex = constants::TAB_COLORS
                        .iter()
                        .find(|(n, _, _)| *n == name)
                        .map(|(_, c, _)| c.to_string())
                        .unwrap_or_default();
                    self.tabs[index].color = hex;
                }
            }
            "wake" => self.wake_tab(index),
            "hibernate" => self.hibernate_tab(index),
            _ => {}
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.save_session();
    }
}

fn panel_kind_str(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::History => "history",
        PanelKind::Downloads => "downloads",
        PanelKind::Settings => "settings",
        PanelKind::Extensions => "extensions",
    }
}

fn permission_str(kind: wry::PermissionKind) -> &'static str {
    use wry::PermissionKind::*;
    match kind {
        Microphone => "🎤 Mikrofon",
        Camera => "📷 Kamera",
        Geolocation => "📍 Konum",
        Notifications => "🔔 Bildirim",
        ClipboardRead => "📋 Pano",
        PointerLock => "🖱  Isaretci",
        Autoplay => "▶  Otomatik Oynatma",
        _ => "Izin",
    }
}

/// Kullanici girdisini tam URL'e cevirir (arama veya adres).
pub fn normalize_url(text: &str) -> String {
    let t = text.trim();
    if t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with("file://")
        || t.starts_with("about:")
        || t.starts_with("data:")
    {
        return t.to_string();
    }
    if !t.contains(' ') && t.contains('.') {
        return format!("https://{t}");
    }
    let query = t.replace(' ', "+");
    format!("https://www.google.com/search?q={query}")
}

/// Web gorunumunun gercek olcek carpanini dondurur.
///
/// Linux/WebKitGTK'da bu deger tao penceresinin scale_factor'undan farkli
/// olabildigi icin yerlesim hesaplarinda bunu kullanmak gerekir.
fn webview_scale(webview: &WebView, window: &Window) -> f64 {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        use gtk::prelude::WidgetExt;
        use wry::WebViewExtUnix;
        let _ = window;
        let s = webview.webview().scale_factor() as f64;
        if s > 0.0 {
            s
        } else {
            1.0
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    )))]
    {
        let _ = webview;
        window.scale_factor()
    }
}
