use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use base64::alphabet::STANDARD;
use base64::Engine;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};

static BASE64_ENGINE: LazyLock<GeneralPurpose> = LazyLock::new(|| GeneralPurpose::new(&STANDARD, GeneralPurposeConfig::default()));

/// Write to a file using an absolute path
pub fn write_file<C: AsRef<[u8]>>(dest: &PathBuf, data: C, encode: bool) -> Result<(), ()> {
    if let Some(dir) = dest.parent() {
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            return Err(());
        }
    }

    if encode {
        if fs::write(dest, BASE64_ENGINE.encode(data)).is_err() {
            return Err(());
        }
    } else if fs::write(dest, data).is_err() {
        return Err(());
    }

    Ok(())
}

/// Reads from a file using an absolute path
pub fn read_file(path: &PathBuf, encoded: bool) -> Option<Vec<u8>> {
    let data = fs::read(path).ok()?;

    return if encoded {
        Some(BASE64_ENGINE.decode(data).ok()?)
    } else {
        Some(data)
    }
}
