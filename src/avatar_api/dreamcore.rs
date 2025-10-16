use std::sync::Arc;
use std::time::Duration;
use dreamcore_image_processor::crop_and_resize;
use dreamcore_image_processor::provider::BackgroundProvider;
use dreamcore_image_processor::provider::pinterest::PinterestProvider;
use dreamcore_image_processor::transformation::distortion::Distortion;
use dreamcore_image_processor::transformation::eyes::{Eyeball, Eyeballs};
use dreamcore_image_processor::transformation::{ImageTransformation, Pipeline};
use dreamcore_image_processor::transformation::text::DreamcoreStyledTextTransform;
use image::{DynamicImage, GenericImageView};
use log::{error, info};
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use crate::avatar_api::{AvatarProvider, FetchError};

pub struct DreamcoreProvider {
    pinterest: PinterestProvider,
    pipeline: Pipeline
}

impl Default for DreamcoreProvider {
    fn default() -> Self {
        let pipeline = Pipeline::default()
            + DreamcoreStyledTextTransform::default()
            + Distortion::new(2.0)
            + Eyeballs::new(Eyeball::SimpleEye, 0..=3)
            + Eyeballs::new(Eyeball::EyeWithWings, 0..=2);

        Self {
            pinterest: PinterestProvider::new("dreamcore landscape"),
            pipeline
        }
    }
}

impl AvatarProvider for DreamcoreProvider {
    async fn fetch_avatar(&self) -> Result<DynamicImage, FetchError> {
        let mut img = {
            loop {
                match self.pinterest.fetch_background().await {
                    Ok(img) => break img,
                    Err(err) => {
                        error!("Failed to fetch background image: {err}, sleeping for 1 second");
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        };

        let (w, h) = img.dimensions();
        info!("Resizing image from {w}x{h} to 512x512");

        crop_and_resize(&mut img, 512);

        info!("Transforming image");

        Ok(
            spawn_blocking(move || {
                self.pipeline.transform(&mut img);
                img
            })
            .await
            .unwrap()
        )
    }
}
