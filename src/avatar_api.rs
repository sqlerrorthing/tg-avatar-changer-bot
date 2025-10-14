pub mod solid_color;
pub mod unsplash;

use image::DynamicImage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

pub trait AvatarProvider {
    fn fetch_avatar<'a>(&'a self) -> impl Future<Output = Result<DynamicImage, FetchError>> + Send;
}
