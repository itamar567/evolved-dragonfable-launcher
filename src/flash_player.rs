use crate::config::{FLASH_PLAYER_LAUNCH_URL, PROJECT_DIRS};
use crate::files::download_file;
use futures::executor;
use std::path::PathBuf;
use std::process;
use std::process::Child;

#[cfg(target_os = "linux")]
use {
    tar::Archive,
    flate2::read::GzDecoder,
    std::fs,
    std::fs::File,
};

fn launch(executable: PathBuf) -> Result<Child, ()> {
    process::Command::new(executable)
        .args([format!("{}/game/DFLoader.swf", FLASH_PLAYER_LAUNCH_URL)])
        .spawn().map_err(|_| ())
}

/// Start the Flash Player as an external process
///
/// The Flash Player is downloaded and cached at `<cache_dir>/flash_player`
#[cfg(target_os = "linux")]
pub fn start_flash_player() -> Result<Child, ()> {
    let executable_path = PROJECT_DIRS.cache_dir().join("flash_player");

    if executable_path.exists() {
        return launch(executable_path);
    }

    let archive_path = PROJECT_DIRS.cache_dir().join("_flash_player.tar.gz");
    let unpacked_archive_path = PROJECT_DIRS.cache_dir().join("_flash_player");

    // Download
    if executor::block_on(download_file(&archive_path, "https://fpdownload.macromedia.com/pub/flashplayer/updaters/32/flash_player_sa_linux.x86_64.tar.gz")).is_err() {
        return Err(());
    }

    // Extract
    let mut archive = Archive::new(GzDecoder::new(File::open(&archive_path).unwrap()));
    archive.unpack(&unpacked_archive_path).unwrap();

    // Move
    if fs::rename(unpacked_archive_path.join("flashplayer"), &executable_path).is_err() {
        return Err(());
    }

    // Delete unused files
    if fs::remove_dir_all(unpacked_archive_path).is_err() {
        return Err(());
    }
    if fs::remove_file(archive_path).is_err() {
        return Err(());
    }

    launch(executable_path)
}

#[cfg(target_os = "windows")]
pub fn start_flash_player() -> Result<Child, ()> {
    let executable_path = PROJECT_DIRS.cache_dir().join("flash_player.exe");

    if executable_path.exists() {
        return launch(executable_path);
    }

    if executor::block_on(download_file(
        &executable_path,
        "https://fpdownload.macromedia.com/pub/flashplayer/updaters/32/flashplayer_32_sa.exe",
    ))
    .is_err()
    {
        return Err(());
    }

    launch(executable_path)
}
