//! The baseline file `--baseline` names, ready to import (D21, D52).
//!
//! A path is used as it is. A URL is downloaded to a temporary file, and a
//! gzipped file, whatever its name, is decompressed into another; nothing is
//! cached, a CI job caching it if it wants. The temporary files are removed
//! when the [`BaselineFile`] drops.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use flate2::read::GzDecoder;
use url::Url;

use crate::config::Config;
use crate::fetch::http::host::Hosts;
use crate::fetch::http::{HttpFetcher, Settings};

/// A baseline file on disk, and the temporary files made to get it there.
#[derive(Debug)]
pub struct BaselineFile {
    path: PathBuf,
    temporary: Vec<PathBuf>,
}

impl BaselineFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Makes `spec`, a path or an `http(s)` URL, a database file to import.
    pub fn open(
        spec: &str,
        config: &Config,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<BaselineFile, String> {
        let mut file = BaselineFile {
            path: PathBuf::from(spec),
            temporary: Vec::new(),
        };
        if let Ok(url) = Url::parse(spec)
            && matches!(url.scheme(), "http" | "https")
        {
            let downloaded = file.temporary_file()?;
            let fetcher = HttpFetcher::new(Settings::default(), Hosts::new(config))
                .map_err(|e| e.to_string())?;
            let mut out = File::create(&downloaded).map_err(|e| failed(&downloaded, &e))?;
            runtime
                .block_on(fetcher.download(&url, &mut out))
                .map_err(|e| format!("{spec}: {e}"))?;
            out.flush().map_err(|e| failed(&downloaded, &e))?;
            file.path = downloaded;
        }
        if gzipped(&file.path).map_err(|e| failed(&file.path, &e))? {
            let plain = file.temporary_file()?;
            let mut input =
                GzDecoder::new(File::open(&file.path).map_err(|e| failed(&file.path, &e))?);
            let mut out = File::create(&plain).map_err(|e| failed(&plain, &e))?;
            io::copy(&mut input, &mut out).map_err(|e| failed(&file.path, &e))?;
            file.path = plain;
        }
        Ok(file)
    }

    /// A new, empty file in the temporary directory, removed with `self`.
    fn temporary_file(&mut self) -> Result<PathBuf, String> {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let name = format!(
            "bruce-baseline-{}-{}.duckdb",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| failed(&path, &e))?;
        self.temporary.push(path.clone());
        Ok(path)
    }
}

impl Drop for BaselineFile {
    fn drop(&mut self) {
        for path in &self.temporary {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn failed(path: &Path, e: &io::Error) -> String {
    format!("{}: {e}", path.display())
}

/// Whether the file begins with gzip's magic number.
fn gzipped(path: &Path) -> io::Result<bool> {
    let mut magic = [0; 2];
    let read = File::open(path)?.read(&mut magic)?;
    Ok(read == 2 && magic == [0x1f, 0x8b])
}

#[cfg(test)]
mod tests;
