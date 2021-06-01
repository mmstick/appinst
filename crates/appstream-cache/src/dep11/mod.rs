pub mod appstream;
pub mod codec;

use anyhow::Context;
use crate::PackageEvent;
use flate2::read::GzDecoder;
use futures_codec::FramedRead;
use futures_lite::prelude::*;
use os_str_bytes::OsStrBytes;
use self::codec::Dep11Splitter;
use smol::channel::Sender;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const LISTS: &str = "/var/lib/apt/lists";

pub fn fetch<'a>(executor: &smol::LocalExecutor<'a>, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    let lists = Path::new(LISTS);

    if !lists.exists() {
        return Ok(());
    }

    // Fetches all DEP11 package lists in the system
    let dep11_entries = fs::read_dir(lists)
        .context("failed to read apt lists dir")?
        .filter_map(Result::ok)
        .filter(|entry| contains_slice(&entry.file_name().to_raw_bytes(), b"dep11_Components"));

    for package_entry in dep11_entries {
        executor.spawn(read_components(package_entry.path(), tx.clone())).detach();
    }

    // Fetches all DEP11 icons
    let dep11_entries = fs::read_dir(lists)
        .context("failed to read apt lists dir")?
        .filter_map(Result::ok)
        .filter(|entry| contains_slice(&entry.file_name().to_raw_bytes(), b"dep11_icons"));

    for package_entry in dep11_entries {
        executor.spawn(read_icons(package_entry.path(), tx.clone())).detach();
    }

    Ok(())
}

async fn read_icons(path: PathBuf, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    let filename = match path.file_name() {
        Some(filename) => filename.to_raw_bytes(),
        None => return Ok(())
    };

    let size = if contains_slice(&filename, b"48x48") {
        "48x48"
    } else if contains_slice(&filename, b"64x64") {
        "64x64"
    } else if contains_slice(&filename, b"128x128") {
        "128x128"
    } else {
        return Ok(())
    };

    smol::unblock(move || {
        let mut archive = File::open(&path)
            .map(GzDecoder::new)
            .map(tar::Archive::new)
            .expect("failed to open file");

        if let Ok(entries) = archive.entries() {
            for mut file in entries.filter_map(Result::ok) {
                if let Ok(path) = file.path() {
                    if let Some(name) = path.file_name().and_then(OsStr::to_str).map(String::from) {
                        use std::io::Read;
                        let mut buffer = Vec::new();
                        if file.read_to_end(&mut buffer).is_ok() {
                            let tx = tx.clone();

                            futures_lite::future::block_on(async move {
                                let _ = tx.send(PackageEvent::Dep11Icon { size, buffer, name }).await;
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }).await
}

async fn read_components(path: PathBuf, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    let decoder = File::open(&path)
        .map(GzDecoder::new)
        .expect("failed to open file");

    let decoder = smol::Unblock::new(decoder);

    let mut stream = FramedRead::new(decoder, Dep11Splitter::default());

    if let Some(result) = stream.next().await {
        let info = result.unwrap();
        if let Some(header) = stream.decoder_mut().header.as_ref() {
            let origin = header.origin.clone();

            if let Some(base_url) = header.media_base_url.clone() {
                let _ = tx.send(PackageEvent::MediaUrl { origin: origin.clone(), base_url }).await;
            }

            let _ = tx.send(PackageEvent::Dep11 { origin: origin.clone(), info }).await;

            while let Some(event) = stream.next().await {
                if let Ok(info) = event {
                    let _ = tx.send(PackageEvent::Dep11 { origin: origin.clone(), info }).await;
                }
            }
        }
    }

    Ok(())
}

fn contains_slice(slice: &[u8], pattern: &[u8]) -> bool {
    slice.windows(pattern.len()).any(|v| v == pattern)
}
