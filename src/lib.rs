use log::{error, info, warn};
use regex::Regex;
use std::time::Duration;

use derive_new::new;
use tokio::time::sleep;

use tdlib_rs::{
    enums::{InputChatPhoto, InputFile},
    functions,
    types::{Error, InputChatPhotoStatic, InputFileLocal},
};

use crate::avatar_api::AvatarProvider;

pub mod avatar_api;

#[derive(Debug, new)]
pub struct AvatarChanger<P> {
    provider: P,
    client_id: i32,
    change_duration: Duration,
}

impl<P: AvatarProvider> AvatarChanger<P> {
    pub async fn run_loop(&self) {
        loop {
            let avatar = self.provider.fetch_avatar().await.unwrap();
            info!("Setting avatar: {}x{}", avatar.width(), avatar.height());

            let file = tempfile::NamedTempFile::new().unwrap();
            avatar
                .write_to(&mut file.as_file(), image::ImageFormat::Jpeg)
                .unwrap();

            let local = InputFileLocal {
                path: file.path().display().to_string(),
            };

            if let Err(err) = try_set_profile_photo(InputFile::Local(local), self.client_id).await {
                error!("Failed to set profile photo: {err:?}");
            } else {
                info!("Avatar has been set");
            }

            info!("Waiting {:.2} seconds", self.change_duration.as_secs_f32());

            sleep(self.change_duration).await;
        }
    }
}

async fn try_set_profile_photo(photo: InputFile, client_id: i32) -> Result<(), Error> {
    loop {
        let Err(err) = functions::set_profile_photo(
            InputChatPhoto::Static(InputChatPhotoStatic {
                photo: photo.clone(),
            }),
            false,
            client_id,
        )
        .await
        else {
            return Ok(());
        };

        match err.code {
            429 => {
                let retry_after = parse_retry_time(&err.message).ok_or(err)? + 1;
                warn!("Ratelimited, retrying after {retry_after}s");
                sleep(Duration::from_secs(retry_after)).await;
            }
            _ => return Err(err),
        }
    }
}

fn parse_retry_time(text: &str) -> Option<u64> {
    let re = unsafe { Regex::new(r"retry after (\d+)").unwrap_unchecked() };

    re.captures(text)
        .and_then(|caps| caps.get(1)?.as_str().parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use crate::parse_retry_time;

    #[test]
    fn prase_retry_time() {
        let message = "Too Many Requests: retry after 294";
        assert_eq!(parse_retry_time(message), Some(294))
    }
}
