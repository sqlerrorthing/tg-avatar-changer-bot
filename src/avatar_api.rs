pub mod mock;

use image::DynamicImage;

#[derive(Debug)]
pub struct FetchError;

pub trait AvatarProvider {
    type FetchAvatarFuture<'a>: Future<Output = Result<DynamicImage, FetchError>> + Send
    where
        Self: 'a;

    fn fetch_avatar<'a>(&'a self) -> Self::FetchAvatarFuture<'a>;
}
