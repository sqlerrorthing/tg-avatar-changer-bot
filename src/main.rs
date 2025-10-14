use reqwest::Client;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tdlib_rs::{
    enums::{self, AuthorizationState, Update},
    functions,
};

use tg_avatar_changer_bot::{AvatarChanger, avatar_api::solid_color::SolidColorProvider};
use tokio::sync::mpsc::{self, Receiver, Sender};

fn ask_user(string: &str) -> String {
    println!("{string}");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    if let Update::AuthorizationState(update) = update {
        auth_tx.send(update.authorization_state).await.unwrap();
    }
}

async fn handle_authorization_state(
    client_id: i32,
    mut auth_rx: Receiver<AuthorizationState>,
    run_flag: Arc<AtomicBool>,
) -> Receiver<AuthorizationState> {
    while let Some(state) = auth_rx.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                let response = functions::set_tdlib_parameters(
                    false,
                    "session_db".into(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                    false,
                    false,
                    env!("API_ID").parse().unwrap(),
                    env!("API_HASH").into(),
                    "en".into(),
                    "Desktop".into(),
                    String::new(),
                    env!("CARGO_PKG_VERSION").into(),
                    client_id,
                )
                .await;

                if let Err(error) = response {
                    println!("{}", error.message);
                }
            }
            AuthorizationState::WaitPhoneNumber => loop {
                let input = ask_user("Enter your phone number (include the country calling code):");
                let response =
                    functions::set_authentication_phone_number(input, None, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::WaitOtherDeviceConfirmation(x) => {
                println!(
                    "Please confirm this login link on another device: {}",
                    x.link
                );
            }
            AuthorizationState::WaitEmailAddress(_x) => {
                let email_address = ask_user("Please enter email address: ");
                let response =
                    functions::set_authentication_email_address(email_address, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            }
            AuthorizationState::WaitEmailCode(_x) => {
                let code = ask_user("Please enter email authentication code: ");
                let response = functions::check_authentication_email_code(
                    enums::EmailAddressAuthentication::Code(
                        tdlib_rs::types::EmailAddressAuthenticationCode { code },
                    ),
                    client_id,
                )
                .await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            }

            AuthorizationState::WaitCode(_) => loop {
                let input = ask_user("Enter the verification code:");
                let response = functions::check_authentication_code(input, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::WaitRegistration(_x) => {
                let first_name = ask_user("Please enter your first name: ");
                let last_name = ask_user("Please enter your last name: ");
                functions::register_user(first_name, last_name, false, client_id)
                    .await
                    .unwrap();
            }
            AuthorizationState::WaitPassword(_x) => {
                let password = ask_user("Please enter password: ");
                functions::check_authentication_password(password, client_id)
                    .await
                    .unwrap();
            }
            AuthorizationState::Ready => {
                break;
            }
            AuthorizationState::Closed => {
                run_flag.store(false, Ordering::Release);
                break;
            }
            _ => (),
        }
    }

    auth_rx
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let client_id = tdlib_rs::create_client();
    let (auth_tx, auth_rx) = mpsc::channel(5);

    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    let handle = tokio::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            let result = tokio::task::spawn_blocking(tdlib_rs::receive)
                .await
                .unwrap();

            if let Some((update, _client_id)) = result {
                handle_update(update, &auth_tx).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    });

    functions::set_log_verbosity_level(2, client_id)
        .await
        .unwrap();

    let auth_rx = handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    AvatarChanger::new(
        SolidColorProvider::default(),
        client_id,
        Duration::from_mins(2),
    )
    .run_loop()
    .await;

    functions::close(client_id).await.unwrap();

    handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    handle.await.unwrap();
}
