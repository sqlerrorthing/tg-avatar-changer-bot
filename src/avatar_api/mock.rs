use std::future::{Ready, ready};

use derive_new::new;
use image::{DynamicImage, Rgba, RgbaImage};
use rand::Rng;

use crate::avatar_api::{AvatarProvider, FetchError};

#[derive(Debug, Clone, new)]
pub struct MockAvatarProvider(u32);

impl AvatarProvider for MockAvatarProvider {
    type FetchAvatarFuture<'a> = Ready<Result<DynamicImage, FetchError>>;

    fn fetch_avatar<'a>(&'a self) -> Self::FetchAvatarFuture<'a> {
        let mut rng = rand::rng();
        let color = Rgba([rng.random(), rng.random(), rng.random(), 255]);

        let img = RgbaImage::from_pixel(self.0, self.0, color);
        ready(Ok(DynamicImage::ImageRgba8(img)))
    }
}
