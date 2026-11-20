//! Sekme web gorunumu olusturma ve yonetimi.
//!
//! Python surumundeki `engine/browserpage.py` karsiligi.

use crate::engine::ad_blocker;
use crate::engine::scripts;
use crate::ipc::{ChromeMsg, UserEvent};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{Rect, WebContext, WebView, WebViewBuilder};

/// Web gorunumu icin etkin ozellik bayraklari.
#[derive(Debug, Clone, Copy, Default)]
pub struct WebConfig {
    pub smooth_scroll: bool,
    pub dark_mode: bool,
    pub ad_block: bool,
    pub privacy: bool,
    pub speedup: bool,
    pub read_mode: bool,
}

/// Kaynak (HTML/gorsel) dizinini bulur.
pub fn resources_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SWIFTX_RESOURCES") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("resources");
            if candidate.join("home.html").exists() {
                return candidate;
            }
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/resources"))
}

/// Ana sayfa URL'i (`file://.../home.html`).
pub fn home_url() -> String {
    let path = resources_dir().join("home.html");
    url::Url::from_file_path(&path)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| "about:blank".to_string())
}

/// Fiziksel (device pixel) bir dikdortgen olusturur.
///
/// WebKitGTK alt-web gorunumleri olcegi kendisi uyguladigi icin konum/boyutlar
/// fiziksel piksel cinsinden verilmelidir; aksi halde HiDPI ekranlarda webview
/// ile arayuz (chrome) hizalanmaz.
pub fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: PhysicalPosition::new(x.round() as i32, y.round() as i32).into(),
        size: PhysicalSize::new(
            width.max(1.0).round() as u32,
            height.max(1.0).round() as u32,
        )
        .into(),
    }
}

fn initialization_scripts(config: &WebConfig) -> Vec<String> {
    let mut scripts_vec = Vec::new();
    scripts_vec.push(scripts::HOTKEYS_JS.to_string());
    scripts_vec.push(scripts::FIND_JS.to_string());
    if std::env::var("SWIFTX_DEBUG").is_ok() || std::env::var("SWIFTX_SELFTEST").is_ok() {
        scripts_vec.push(scripts::CONTENT_PROBE_JS.to_string());
    }
    if config.smooth_scroll {
        scripts_vec.push(scripts::SMOOTH_SCROLL_JS.to_string());
        scripts_vec.push(scripts::KEYBOARD_SCROLL_JS.to_string());
    }
    if config.dark_mode {
        scripts_vec.push(scripts::AUTO_DARK_MODE_JS.to_string());
    }
    if config.ad_block {
        scripts_vec.push(ad_blocker::ad_block_js());
    }
    if config.privacy {
        scripts_vec.push(ad_blocker::privacy_js().to_string());
    }
    if config.speedup {
        scripts_vec.push(ad_blocker::speedup_js().to_string());
    }
    if config.read_mode {
        scripts_vec.push(ad_blocker::read_mode_js().to_string());
    }
    scripts_vec
}

/// Yeni bir sekme web gorunumu olusturur.
#[allow(clippy::too_many_arguments)]
pub fn build_tab(
    window: &Window,
    context: &mut WebContext,
    config: &WebConfig,
    tab_id: u64,
    url: &str,
    bounds: Rect,
    proxy: EventLoopProxy<UserEvent>,
    blocker: Arc<AtomicBool>,
) -> wry::Result<WebView> {
    let signal_proxy = proxy.clone();
    let mut builder = WebViewBuilder::new_with_web_context(context)
        .with_bounds(bounds)
        .with_url(url)
        .with_background_color((28, 27, 34, 255))
        .with_devtools(true)
        .with_clipboard(true)
        .with_autoplay(true)
        .with_back_forward_navigation_gestures(true)
        .with_ipc_handler({
            let proxy = proxy.clone();
            move |req: wry::http::Request<String>| {
                if let Ok(msg) = serde_json::from_str::<ChromeMsg>(req.body()) {
                    let _ = proxy.send_event(UserEvent::Chrome(msg));
                }
            }
        })
        .with_navigation_handler({
            let proxy = proxy.clone();
            let blocker = blocker.clone();
            move |nav_url: String| {
                if blocker.load(Ordering::Relaxed) && ad_blocker::should_block(&nav_url) {
                    return false;
                }
                let _ = proxy.send_event(UserEvent::Url {
                    tab: tab_id,
                    url: nav_url,
                });
                true
            }
        })
        .with_on_page_load_handler({
            let proxy = proxy.clone();
            move |event, _url| match event {
                wry::PageLoadEvent::Started => {
                    let _ = proxy.send_event(UserEvent::LoadStarted { tab: tab_id });
                }
                wry::PageLoadEvent::Finished => {
                    let _ = proxy.send_event(UserEvent::LoadFinished { tab: tab_id });
                }
            }
        })
        .with_document_title_changed_handler({
            let proxy = proxy.clone();
            move |title: String| {
                let _ = proxy.send_event(UserEvent::Title { tab: tab_id, title });
            }
        })
        .with_download_started_handler({
            let proxy = proxy.clone();
            move |url: String, path: &mut PathBuf| {
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "download".to_string());
                let target = crate::core::services::DownloadManager::download_dir().join(&filename);
                *path = target.clone();
                let _ = proxy.send_event(UserEvent::DownloadStarted { url, path: target });
                true
            }
        })
        .with_download_completed_handler({
            let proxy = proxy.clone();
            move |url: String, path: Option<PathBuf>, ok: bool| {
                let _ = proxy.send_event(UserEvent::DownloadFinished { url, path, ok });
            }
        })
        .with_new_window_req_handler({
            let proxy = proxy.clone();
            move |url: String, _features| {
                let _ = proxy.send_event(UserEvent::NewWindow { tab: tab_id, url });
                wry::NewWindowResponse::Deny
            }
        })
        .with_permission_handler({
            let proxy = proxy.clone();
            move |kind| {
                let _ = proxy.send_event(UserEvent::Permission { kind });
                match kind {
                    wry::PermissionKind::Microphone
                    | wry::PermissionKind::Camera
                    | wry::PermissionKind::Geolocation
                    | wry::PermissionKind::Notifications
                    | wry::PermissionKind::ClipboardRead
                    | wry::PermissionKind::PointerLock
                    | wry::PermissionKind::Autoplay => wry::PermissionResponse::Allow,
                    _ => wry::PermissionResponse::Default,
                }
            }
        });

    for script in initialization_scripts(config) {
        builder = builder.with_initialization_script(script);
    }

    let webview = builder.build_as_child(window)?;

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        use webkit2gtk::WebViewExt;
        use wry::WebViewExtUnix;
        let inner = webview.webview();
        let p_enter = signal_proxy.clone();
        inner.connect_enter_fullscreen(move |_| {
            let _ = p_enter.send_event(UserEvent::Fullscreen(true));
            true
        });
        let p_leave = signal_proxy.clone();
        inner.connect_leave_fullscreen(move |_| {
            let _ = p_leave.send_event(UserEvent::Fullscreen(false));
            true
        });
    }
    let _ = &signal_proxy;

    Ok(webview)
}
