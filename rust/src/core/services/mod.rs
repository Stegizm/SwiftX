//! Servis katmani — veri yoneticileri.

pub mod bookmarks;
pub mod downloads;
pub mod history;
pub mod settings;

pub use bookmarks::BookmarkManager;
pub use downloads::DownloadManager;
pub use history::HistoryManager;
pub use settings::SettingsManager;
