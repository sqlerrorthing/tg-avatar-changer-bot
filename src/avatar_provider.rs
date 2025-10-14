pub mod folder;
pub mod solid_color;
pub mod unsplash;

use std::future::ready;

use image::DynamicImage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Network: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("{0}")]
    IOError(#[from] std::io::Error),

    #[error("Task Join Error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("No more avatars is left")]
    NoMoreAvatars,
}

pub trait AvatarProvider {
    fn fetch_avatar(&self) -> impl Future<Output = Result<DynamicImage, FetchError>> + Send;

    fn how_much_is_left(&self) -> impl Future<Output = Option<usize>> + Send {
        ready(None)
    }
}
