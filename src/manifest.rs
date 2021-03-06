use serde::{Deserialize, Serialize};
use std::io::prelude::*;

use crate::error::KyaniteError;
use flate2::Compression;

use log::debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KyaniteManifestItem {
    pub url: String,
    pub file: String,
    pub tags: Vec<String>,
}

impl KyaniteManifestItem {
    pub fn new(url: String, file: String, tags: Vec<String>) -> Self {
        Self { url, file, tags }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KyaniteManifest {
    pub files: Vec<KyaniteManifestItem>,
    pub downloader: String,
}

impl KyaniteManifest {
    pub fn new(downloader: String) -> Self {
        Self {
            files: Vec::new(),
            downloader,
        }
    }

    fn get_path(&self) -> Result<String, KyaniteError> {
        let folder = format!("downloads/{}", &self.downloader);
        if !std::path::Path::new(&folder).exists() {
            debug!(
                "Manifest folder for {} doesn't exist, creating it.",
                &self.downloader
            );
            std::fs::create_dir_all(&folder)?;
        }
        Ok(format!("{}/manifest.json.gz", folder))
    }

    pub fn add(&mut self, item: KyaniteManifestItem) -> Self {
        let mut exists = false;
        for file in &self.files {
            if &file.url == &item.url {
                exists = true;
                break;
            }
        }
        if !exists {
            self.files.push(item);
        } else {
            debug!(
                "Skipped adding {} cause it's already in the manifest.",
                item.url
            );
        }
        self.clone()
    }

    pub fn load(&self) -> Result<Self, KyaniteError> {
        let path = &self.get_path()?;
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer)?;
        let mut contents = String::new();
        flate2::read::GzDecoder::new(buffer.as_slice()).read_to_string(&mut contents)?;
        let manifest: Self;
        if &contents != "" {
            manifest = serde_json::from_str(&contents)?;
        } else {
            manifest = self.to_owned();
        }
        debug!(
            "Manifest for {} loaded with {} items.",
            &self.downloader,
            &self.files.len()
        );
        Ok(manifest)
    }

    pub fn save(&self) -> Result<(), KyaniteError> {
        let path = &self.get_path()?;
        let serialized = serde_json::ser::to_string(self)?;
        let mut gz = flate2::GzBuilder::new()
            .filename(format!("{}_manifest.json", &self.downloader))
            .comment(format!("Kyanite manifest for {}", &self.downloader))
            .write(std::fs::File::create(&path)?, Compression::best());
        let ser_bytes: Vec<u8> = serialized.into_bytes();
        gz.write_all(ser_bytes.as_slice())?;
        gz.finish()?;
        debug!(
            "Manifest for {} saved with {} items.",
            &self.downloader,
            &self.files.len()
        );
        Ok(())
    }
}
