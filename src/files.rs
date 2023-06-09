use reqwest::IntoUrl;
use std::fs;
use std::path::PathBuf;

/// Write to a file using an absolute path
pub fn write_file<C: AsRef<[u8]>>(dest: &PathBuf, data: C) -> Result<(), ()> {
    if let Some(dir) = dest.parent() {
        // if !dir.exists() && fs::create_dir_all(dir).is_err() {
        //     dbg!("e1");
        //     return Err(());
        // }
        if !dir.exists() {
            if let Err(err) = fs::create_dir_all(dir) {
                dbg!(err);
                dbg!(dest);
                return Err(());
            }
        }
    }

    if fs::write(dest, data).is_err() {
        return Err(());
    }

    Ok(())
}

pub async fn download_file<T: IntoUrl>(dest: &PathBuf, url: T) -> Result<(), ()> {
    match reqwest::get(url).await {
        Ok(response) => {
            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => return Err(()),
            };

            if write_file(dest, bytes).is_err() {
                return Err(());
            }

            Ok(())
        }
        Err(_) => Err(()),
    }
}
