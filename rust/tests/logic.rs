//! Saf mantik fonksiyonlarinin (URL normalizasyonu, reklam engelleme) testleri.

use swiftx::app::normalize_url;
use swiftx::core::updater::is_newer;
use swiftx::engine::ad_blocker::should_block;

#[test]
fn normalize_url_cases() {
    assert_eq!(normalize_url("example.com"), "https://example.com");
    assert_eq!(normalize_url("  https://a.b/c  "), "https://a.b/c");
    assert_eq!(normalize_url("http://x.y"), "http://x.y");
    assert_eq!(
        normalize_url("merhaba dunya"),
        "https://www.google.com/search?q=merhaba+dunya"
    );
    assert_eq!(normalize_url("about:blank"), "about:blank");
    assert_eq!(normalize_url("file:///tmp/x.html"), "file:///tmp/x.html");
}

#[test]
fn ad_blocker_cases() {
    assert!(should_block("https://doubleclick.net/x"));
    assert!(should_block("https://www.google.com/ads/foo"));
    assert!(should_block("https://analytics.google.com/collect"));
    assert!(!should_block("https://www.youtube.com/watch?v=abc"));
    assert!(should_block("https://youtube.com/api/stats?x=1"));
    assert!(!should_block("https://example.com/index.html"));
}

#[test]
fn resources_exist() {
    let home = swiftx::engine::browser::resources_dir().join("home.html");
    assert!(home.exists(), "home.html bulunamadi: {}", home.display());
}

#[test]
fn version_comparison() {
    assert!(is_newer("0.3.1", "0.3.0"));
    assert!(is_newer("0.4.0", "0.3.9"));
    assert!(is_newer("1.0.0", "0.9.9"));
    assert!(is_newer("v0.3.10", "0.3.9"));
    assert!(!is_newer("0.3.0", "0.3.0"));
    assert!(!is_newer("0.2.9", "0.3.0"));
}

#[test]
fn chrome_ui_contains_key_ids() {
    let html = swiftx::ui::CHROME_HTML;
    for id in [
        "urlBar",
        "tabScroll",
        "bookmarks",
        "panel",
        "findbar",
        "newTabBtn",
    ] {
        assert!(html.contains(id), "chrome.html icinde eksik: {id}");
    }
}
