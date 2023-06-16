use std::fs;
use std::path::PathBuf;

/// Write to a file using an absolute path
pub fn write_file<C: AsRef<[u8]>>(dest: &PathBuf, data: C) -> Result<(), ()> {
    if let Some(dir) = dest.parent() {
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            return Err(());
        }
    }

    if fs::write(dest, data).is_err() {
        return Err(());
    }

    Ok(())
}
