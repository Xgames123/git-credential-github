static OAUTH_CLIENT_ID: &str = "71c898ad634b388e6614";
static OAUTH_SCOPE: &str = "repo";

use log::{debug, error};
use serde::Deserialize;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

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

#[allow(unused)]
#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

pub fn epoch_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards????");

    since_the_epoch.as_secs()
}

#[derive(Debug, Error)]
pub enum AccessTokenPollError {
    #[error("The device code has expired")]
    DeviceCodeExpired,
    #[error("Polling github failed. {0}")]
    Ureq(#[from] ureq::Error),
}

pub fn poll_for_access_token(
    agent: &ureq::Agent,
    device_code: &DeviceCode,
) -> Result<AccessToken, AccessTokenPollError> {
    let form_params = [
        ("client_id", OAUTH_CLIENT_ID),
        ("scope", OAUTH_SCOPE),
        ("device_code", &device_code.device_code),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];

    Ok(loop {
        if device_code.expired() {
            return Err(AccessTokenPollError::DeviceCodeExpired);
        }

        sleep(Duration::from_secs(device_code.interval));

        let mut response = agent
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .send_form(form_params)?;

        if response.status() == 404 {
            continue;
        }
        match response.body_mut().read_json() {
            Ok(token) => break token,
            Err(e) => {
                let string_response = response.body_mut().read_to_string()?;
                error!(
                    "Invalid response received from github.\n{}",
                    string_response
                );
                return Err(e.into());
            }
        }
    })
}

pub fn get_device_code(agent: &ureq::Agent) -> Result<DeviceCode, ureq::Error> {
    debug!("get_device_code");
    let form_params = [("client_id", OAUTH_CLIENT_ID), ("scope", OAUTH_SCOPE)];

    let mut response = agent
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .send_form(form_params)?;

    let mut device_code: DeviceCode = response.body_mut().read_json()?;

    device_code.time = epoch_time();
    debug!("device_code: {:?}", device_code);
    Ok(device_code)
}
