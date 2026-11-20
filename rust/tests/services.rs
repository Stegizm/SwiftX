//! Cekirdek servislerin davranis testleri.

use std::sync::Mutex;
use swiftx::core::services::{BookmarkManager, DownloadManager, HistoryManager, SettingsManager};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_data_dir<F: FnOnce()>(f: F) {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("swiftx-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("SWIFTX_DATA_DIR", &dir);
    f();
    let _ = std::fs::remove_dir_all(&dir);
    std::env::remove_var("SWIFTX_DATA_DIR");
}

#[test]
fn bookmarks_flow() {
    with_temp_data_dir(|| {
        let mut mgr = BookmarkManager::load();
        assert!(mgr.bookmarks().is_empty());
        assert!(mgr.add("https://example.com", "Example").is_ok());
        assert!(mgr.add("https://example.com", "Example").is_err());
        assert!(mgr.add("file:///tmp/x.html", "Local").is_err());
        assert!(mgr.add("about:blank", "Blank").is_err());
        assert_eq!(mgr.bookmarks().len(), 1);
        mgr.remove("Example");
        assert!(mgr.bookmarks().is_empty());
    });
}

#[test]
fn history_flow() {
    with_temp_data_dir(|| {
        let mut mgr = HistoryManager::load();
        mgr.add("A", "https://a.com");
        mgr.add("B", "https://b.com");
        mgr.add("Skip", "file:///tmp/x.html");
        assert_eq!(mgr.history().len(), 2);
        assert_eq!(mgr.history()[0].url, "https://b.com");
        mgr.clear();
        assert!(mgr.history().is_empty());
    });
}

#[test]
fn settings_persistence() {
    with_temp_data_dir(|| {
        {
            let mut mgr = SettingsManager::load();
            assert!(mgr.flag("dark_mode").unwrap());
            assert_eq!(mgr.toggle("dark_mode"), Some(false));
        }
        let reloaded = SettingsManager::load();
        assert!(!reloaded.flag("dark_mode").unwrap());
        assert_eq!(reloaded.total_extension_count(), 5);
        assert_eq!(reloaded.active_extension_count(), 4);
    });
}

#[test]
fn settings_extension_changes() {
    with_temp_data_dir(|| {
        let mut mgr = SettingsManager::load();
        mgr.apply_extension_changes(&[("readmode".to_string(), true)]);
        assert!(mgr.extension_enabled("readmode"));
        assert_eq!(mgr.active_extension_count(), 5);
    });
}

#[test]
fn downloads_flow() {
    with_temp_data_dir(|| {
        let mut mgr = DownloadManager::load();
        mgr.add("file.pdf", "/tmp/file.pdf");
        assert_eq!(mgr.downloads().len(), 1);
        mgr.clear();
        assert!(mgr.downloads().is_empty());
    });
}
