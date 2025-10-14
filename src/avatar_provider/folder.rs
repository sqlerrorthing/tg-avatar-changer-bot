use crate::avatar_provider::{AvatarProvider, FetchError};
use image::DynamicImage;
use log::info;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{fs, task};

pub struct FolderProvider {
    images: Arc<Mutex<Vec<PathBuf>>>,
}

impl<'de> Deserialize<'de> for FolderProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            path: PathBuf,
        }

        let helper = Helper::deserialize(deserializer)?;
        FolderProvider::new(helper.path).map_err(serde::de::Error::custom)
    }
}

impl FolderProvider {
    pub fn new(folder: impl AsRef<Path>) -> Result<Self, FetchError> {
        let mut images = Vec::new();
        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "png" | "jpg" | "jpeg" => images.push(path),
                        _ => {}
                    }
                }
            }
        }

        info!("Loaded {} avatars", images.len());

        Ok(Self {
            images: Arc::new(Mutex::new(images)),
        })
    }
}

impl AvatarProvider for FolderProvider {
    async fn fetch_avatar(&self) -> Result<DynamicImage, FetchError> {
        let path_opt = {
            let mut imgs = self.images.lock().await;
            imgs.pop()
        };

        let path = match path_opt {
            Some(p) => p,
            None => return Err(FetchError::NoMoreAvatars),
        };

        let content = fs::read(&path).await?;

        let image = task::spawn_blocking(move || image::load_from_memory(&content)).await??;

        Ok(image)
    }

    async fn how_much_is_left(&self) -> Option<usize> {
        Some(self.images.lock().await.len())
    }
}
