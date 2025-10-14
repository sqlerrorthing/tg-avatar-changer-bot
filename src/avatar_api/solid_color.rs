use derive_new::new;
use image::{DynamicImage, Rgba, RgbaImage};
use rand::Rng;

use crate::avatar_api::{AvatarProvider, FetchError};

#[derive(Debug, Clone, new)]
pub struct SolidColorProvider {
    size: u32,
    color: Option<Rgba<u8>>,
}

impl Default for SolidColorProvider {
    fn default() -> Self {
        Self {
            size: 512,
            color: None,
        }
    }
}

impl AvatarProvider for SolidColorProvider {
    async fn fetch_avatar<'a>(&'a self) -> Result<DynamicImage, FetchError> {
        let color = self.color.unwrap_or({
            let mut rng = rand::rng();
            Rgba([rng.random(), rng.random(), rng.random(), 255])
        });

        let img = RgbaImage::from_pixel(self.size, self.size, color);
        Ok(DynamicImage::ImageRgba8(img))
    }
}
