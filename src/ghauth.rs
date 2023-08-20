static OAUTH_CLIENT_ID: &str = "71c898ad634b388e6614";
static OAUTH_SCOPE: &str = "repo";

use log::debug;
use reqwest::StatusCode;
use serde::Deserialize;
use std::fmt::Formatter;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{error::Error, fmt, fmt::Display, thread::sleep};

#[derive(Deserialize, Debug)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,

    #[serde(skip)]
    pub time: u64,
}
impl DeviceCode {
    fn expired(&self) -> bool {
        if (epoch_time() - self.time) >= self.expires_in {
            return true;
        }
        false
    }
}

#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct GithubError {
    pub error: String,
    pub error_description: String,
    pub error_uri: String,
}

pub fn epoch_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?????");

    since_the_epoch.as_secs()
}

#[derive(Debug)]
pub enum AccessTokenPollError {
    DeviceCodeExpired,
    Reqwest(reqwest::Error),
}
impl Display for AccessTokenPollError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            AccessTokenPollError::DeviceCodeExpired => fmt.write_str("The device code has expired"),
            AccessTokenPollError::Reqwest(err) => {
                fmt.write_fmt(format_args!("Polling github failed. {}", err))
            }
        }
    }
}

impl Error for AccessTokenPollError {}

pub async fn poll_for_access_token(
    client: &reqwest::Client,
    device_code: &DeviceCode,
) -> Result<AccessToken, AccessTokenPollError> {
    let form_params = [
        ("client_id", OAUTH_CLIENT_ID.to_string()),
        ("scope", OAUTH_SCOPE.to_string()),
        ("device_code", device_code.device_code.to_owned()),
        (
            "grant_type",
            "urn:ietf:params:oauth:grant-type:device_code".to_string(),
        ),
    ];

    loop {
        if device_code.expired() {
            return Err(AccessTokenPollError::DeviceCodeExpired);
        }

        sleep(Duration::from_secs(device_code.interval));

        let request = client
            .post("https://github.com/login/oauth/access_token")
            .form(&form_params)
            .header("Accept", "application/json");

        break match request.send().await {
            Ok(response) => {
                match response.json().await {
                    Ok(token) => Ok(token),
                    Err(_) => {
                        continue;
                    } //BADCODE: Github doesn't set a status code so we threat all parser errors as 'authorization_pending'
                }
            }
            Err(err) => {
                if err.status().unwrap_or(StatusCode::OK) == 404 {
                    continue;
                }

                Err(AccessTokenPollError::Reqwest(err))
            }
        };
    }
}

pub async fn get_device_code(client: &reqwest::Client) -> Result<DeviceCode, reqwest::Error> {
    debug!("get_device_code");
    let form_params = [("client_id", OAUTH_CLIENT_ID), ("scope", OAUTH_SCOPE)];

    let request = client
        .post("https://github.com/login/device/code")
        .form(&form_params)
        .header("Accept", "application/json");

    let response = request.send().await?;
    let mut device_code: DeviceCode = response.json().await?;

    device_code.time = epoch_time();
    debug!("device_code: {:?}", device_code);
    Ok(device_code)
}
