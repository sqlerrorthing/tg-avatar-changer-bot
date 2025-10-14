pub mod solid_color;

use image::DynamicImage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network: {0}")]
    NetworkError(#[from] reqwest::Error),
}

pub trait AvatarProvider {
    fn fetch_avatar<'a>(&'a self) -> impl Future<Output = Result<DynamicImage, FetchError>> + Send;
}
