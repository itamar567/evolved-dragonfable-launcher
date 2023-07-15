use crate::config::PROJECT_DIRS;
use base64::alphabet::STANDARD;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::Engine;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

static BASE64_ENGINE: LazyLock<GeneralPurpose> =
    LazyLock::new(|| GeneralPurpose::new(&STANDARD, GeneralPurposeConfig::default()));

/// Write to a file using an absolute path
pub fn write_file<C: AsRef<[u8]>>(
    dest: &PathBuf,
    data: C,
    encode: bool,
    clear_cache_if_changed: bool,
) -> Result<(), ()> {
    if let Some(dir) = dest.parent() {
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            return Err(());
        }
    }

    let encoded_data_string = BASE64_ENGINE.encode(&data);
    let data = if encode {
        encoded_data_string.as_bytes()
    } else {
        data.as_ref()
    };

    if clear_cache_if_changed {
        if let Ok(old_data) = fs::read(dest) {
            if old_data != data {
                // The file was changed, we should clear the cache
                if fs::remove_dir_all(PROJECT_DIRS.cache_dir()).is_err() {
                    return Err(());
                }
            }
        }
    }

    if fs::write(dest, data).is_err() {
        return Err(());
    }

    Ok(())
}

/// Reads from a file using an absolute path
pub fn read_file(path: &PathBuf, encoded: bool) -> Option<Vec<u8>> {
    let data = fs::read(path).ok()?;

    if encoded {
        Some(BASE64_ENGINE.decode(data).ok()?)
    } else {
        Some(data)
    }
}
