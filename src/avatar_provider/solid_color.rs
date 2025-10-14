use derive_new::new;
use image::{DynamicImage, Rgba, RgbaImage};
use rand::Rng;
use serde::Deserialize;

use crate::avatar_provider::{AvatarProvider, FetchError};

#[derive(Debug, Clone, new, Deserialize)]
pub struct SolidColorProvider {
    size: u32,
    #[serde(deserialize_with = "deserialize_hex_color")]
    color: Option<Rgba<u8>>,
}

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<Option<Rgba<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    if let Some(hex) = opt {
        parse_hex_color(&hex)
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

fn parse_hex_color(hex: &str) -> Result<Rgba<u8>, String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(format!("invalid hex color length: {}", hex));
    }

    u8::from_str_radix(&hex[0..2], 16)
        .and_then(|r| u8::from_str_radix(&hex[2..4], 16).map(|g| (r, g)))
        .and_then(|(r, g)| u8::from_str_radix(&hex[4..6], 16).map(|b| Rgba([r, g, b, 255])))
        .map_err(|_| format!("invalid hex color: {}", hex))
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
