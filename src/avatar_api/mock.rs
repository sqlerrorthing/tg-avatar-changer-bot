use std::time::Duration;

use derive_new::new;
use image::{DynamicImage, Rgba, RgbaImage};
use rand::Rng;
use tokio::time::sleep;

use crate::avatar_api::{AvatarProvider, FetchError};

#[derive(Debug, Clone, new)]
pub struct MockAvatarProvider(u32);

impl Default for MockAvatarProvider {
    fn default() -> Self {
        Self(512)
    }
}

impl AvatarProvider for MockAvatarProvider {
    async fn fetch_avatar<'a>(&'a self) -> Result<DynamicImage, FetchError> {
        let color = {
            let mut rng = rand::rng();
            Rgba([rng.random(), rng.random(), rng.random(), 255])
        };

        let img = RgbaImage::from_pixel(self.0, self.0, color);
        Ok(DynamicImage::ImageRgba8(img))
    }
}
