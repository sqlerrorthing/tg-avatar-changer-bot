use std::time::Duration;

use derive_new::new;
use tokio::time::sleep;

use tdlib_rs::{
    enums::{InputChatPhoto, InputFile},
    functions,
    types::{InputChatPhotoStatic, InputFileLocal},
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
            println!("Setting avatar: {}x{}", avatar.width(), avatar.height());

            let file = tempfile::NamedTempFile::new().unwrap();
            avatar
                .write_to(&mut file.as_file(), image::ImageFormat::Jpeg)
                .unwrap();

            let local = InputFileLocal {
                path: file.path().display().to_string(),
            };

            functions::set_profile_photo(
                InputChatPhoto::Static(InputChatPhotoStatic {
                    photo: InputFile::Local(local),
                }),
                false,
                self.client_id,
            )
            .await
            .unwrap();

            println!(
                "Avatar has been set, waiting {:.2} seconds",
                self.change_duration.as_secs_f32()
            );

            sleep(self.change_duration).await;
        }
    }
}
