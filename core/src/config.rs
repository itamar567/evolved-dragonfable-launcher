use directories::ProjectDirs;
use std::sync::LazyLock;

pub const LOCAL_SERVER_ADDR: &str = "127.0.0.1:39260";
pub const FLASH_PLAYER_LAUNCH_URL: &str = "http://127.0.0.1:39260";
pub const REMOTE_SERVER_ADDR: &str = "play.dragonfable.com";
pub const REMOTE_SERVER_URL: &str = "https://play.dragonfable.com";

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("com", "itmr", "Evolved-DragonFable-Launcher").unwrap());
