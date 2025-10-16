pub mod solid_color;
pub mod unsplash;
pub mod dreamcore;

use image::DynamicImage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Network: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("No more avatars is left")]
    NoMoreAvatars,
}

pub trait AvatarProvider {
    fn fetch_avatar(&self) -> impl Future<Output=Result<DynamicImage, FetchError>> + Send;

    fn how_much_is_left(&self) -> Option<usize> {
        None
    }
}
