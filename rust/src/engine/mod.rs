//! Web motoru katmani: sekme web gorunumleri, reklam engelleme, enjekte scriptler.

pub mod ad_blocker;
pub mod browser;
pub mod scripts;

pub use browser::{build_tab, home_url, rect, resources_dir, WebConfig};
