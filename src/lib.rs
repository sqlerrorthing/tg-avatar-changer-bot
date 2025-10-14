use log::{error, info, warn};
use regex::Regex;
use std::time::Duration;
use std::time::Instant;
use tempfile::NamedTempFile;

use derive_new::new;
use tokio::time::sleep;

use tdlib_rs::{
    enums::{InputChatPhoto, InputFile},
    functions,
    types::{Error, InputChatPhotoStatic, InputFileLocal},
};

use crate::avatar_provider::AvatarProvider;
use crate::avatar_provider::FetchError;

pub mod avatar_provider;

const MAX_AVATAR_FETCH_ATTEMPTS: u8 = 5;

#[derive(Debug, new)]
pub struct AvatarChanger<P> {
    provider: P,
    client_id: i32,
    change_duration: Duration,
}

macro_rules! measure {
    ($func:expr) => {{
        let start = Instant::now();
        let result = $func;
        let elapsed = start.elapsed();
        (result, elapsed.as_secs_f64())
    }};
}

impl<P: AvatarProvider> AvatarChanger<P> {
    async fn fetch_and_write_avatar_to_tmpfile(&self) -> Result<NamedTempFile, FetchError> {
        let (avatar, elapsed) = {
            let mut attempt = 1;
            loop {
                if attempt == 1 {
                    info!("Fetching new avatar");
                } else {
                    info!(
                        "Fetching new avatar ({}/{MAX_AVATAR_FETCH_ATTEMPTS})",
                        attempt
                    );
                }

                let (res, elapsed) = measure!(self.provider.fetch_avatar().await);

                match res {
                    Ok(avatar) => break (avatar, elapsed),
                    Err(FetchError::NoMoreAvatars) => {
                        info!("No more avatars");
                        return Err(FetchError::NoMoreAvatars);
                    }
                    Err(err) => {
                        error!("Error fetching new avatar: {err:?}");

                        attempt += 1;
                        if attempt > MAX_AVATAR_FETCH_ATTEMPTS {
                            return Err(err);
                        }
                    }
                }
            }
        };

        info!(
            "Fetched new avatar in {elapsed:.2} seconds{}",
            self.provider
                .how_much_is_left()
                .await
                .map(|left| format!(", {left} avatars left"))
                .unwrap_or_default()
        );

        let file = unsafe { tempfile::NamedTempFile::new().unwrap_unchecked() };
        avatar
            .write_to(&mut file.as_file(), image::ImageFormat::Jpeg)?;

        info!("Saved temporary as {}", file.path().display());

        Ok(file)
    }

    pub async fn run_loop(&self) {
        loop {
            let avatar = match self.fetch_and_write_avatar_to_tmpfile().await {
                Ok(avatar) => avatar,
                Err(FetchError::NoMoreAvatars) => {
                    break;
                }
                Err(err) => {
                    error!("Unexpected error: {err:#?}");
                    continue;
                }
            };

            let local = InputFileLocal {
                path: avatar.path().display().to_string(),
            };

            let (result, set_time) =
                measure!(try_set_profile_photo(InputFile::Local(local), self.client_id).await);

            drop(avatar); // delete temp file

            match result {
                Ok(_) => info!("Avatar has been set in {set_time:.2} seconds"),
                Err(err) => error!("Failed to set profile photo: {err:?}"),
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
                warn!("Rate limited, retrying after {retry_after}s");
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
