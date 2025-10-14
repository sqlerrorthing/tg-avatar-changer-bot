use derive_new::new;
use reqwest::Client;
use serde::Deserialize;

use crate::avatar_api::AvatarProvider;

#[derive(Debug, new)]
pub struct UnsplashProvider {
    client_id: String,
    query: Option<String>,
    client: Client,
}

impl<'de> Deserialize<'de> for UnsplashProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            client_id: String,
            query: Option<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(UnsplashProvider::new(
            helper.client_id,
            helper.query,
            Client::new(),
        ))
    }
}

#[derive(Deserialize, Debug)]
struct UnsplashApiResponse {
    urls: Urls,
}

#[derive(Deserialize, Debug)]
struct Urls {
    regular: String,
}

impl AvatarProvider for UnsplashProvider {
    async fn fetch_avatar<'a>(&'a self) -> Result<image::DynamicImage, super::FetchError> {
        let mut params = Vec::with_capacity(3);
        params.extend_from_slice(&[
            ("client_id", self.client_id.as_str()),
            ("orientation", "squarish"),
        ]);

        if let Some(query) = &self.query {
            params.push(("query", query.as_str()));
        }

        let random_image_url = self
            .client
            .get("https://api.unsplash.com/photos/random")
            .query(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<UnsplashApiResponse>()
            .await?
            .urls
            .regular;

        let bytes = self
            .client
            .get(random_image_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        Ok(image::load_from_memory(&bytes)?)
    }
}
