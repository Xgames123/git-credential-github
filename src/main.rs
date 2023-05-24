mod ghauth;
mod params;

use crate::ghauth::AccessTokenPollError;
use clap::{crate_authors, crate_name, crate_version};
use clap::ArgAction::SetTrue;
use params::Params;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let args = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about("A simple git credentials manager for github")
        .subcommand(
            clap::Command::new("get")
                .about("Gets the stored credentials")
                .arg(clap::Arg::new("no-prompt").short('n').action(SetTrue)),
        )
        .get_matches();

    match args.subcommand() {
        Some(("get", sub)) => {
            if !sub.get_flag("no-prompt") {
                eprintln!("*******************************************************");
                eprintln!("*                       gh-login                      *");
                eprintln!("*     A simple git credentials manager for github     *");
                eprintln!("*******************************************************");
                eprintln!("NOTE: use --no-prompt to disable this message");
            }

            let mut params =
                Params::from_stdin().expect("Failed to read parameters returned by git {}");

            match params.get("host".to_string()) {
                Some(host) => {
                    if host != "github.com" {
                        return;
                    }
                }
                None => {
                    return;
                }
            };

            match params.get("protocol".to_string()) {
                Some(protocol) => {
                    if protocol == "http" {
                        log("http is not supported. Use https instead");
                        return;
                    }
                    if protocol != "https" {
                        return;
                    }
                }
                None => {
                    return;
                }
            };

            let username = params
                .get("username".to_string())
                .expect("No username parameter given by git");

            let client = reqwest::Client::new();
            let access_token = get_access_token_via_device_code(&client).await;

            params.add(String::from("password"), access_token.access_token);

            params.write_to_sdtout();
        }
        _ => {}
    }
}

async fn get_access_token_via_device_code(client: &reqwest::Client) -> ghauth::AccessToken {
    let device_code = get_device_code(&client).await;

    return loop {
        break match ghauth::poll_for_access_token(&client, &device_code).await {
            Ok(token) => token,
            Err(err) => match err {
                AccessTokenPollError::DeviceCodeExpired => {
                    log("Device code expired");
                    continue;
                }
                AccessTokenPollError::Reqwest(error) => {
                    panic!("{}", error);
                }
            },
        };
    };
}

async fn get_device_code(client: &Client) -> ghauth::DeviceCode {
    log("Getting login code...");
    let device_code = ghauth::get_device_code(&client)
        .await
        .unwrap_or_else(|err| {
            panic!("Failed to get device code \n Err: {}", err);
        });

    eprintln!("gh-login: Go to the link below and type in the code");
    eprintln!("{}", device_code.verification_uri);
    eprintln!("code: {}", device_code.user_code);

    return device_code;
}

fn get_repo_uri(params: &Params) -> Result<String, ParamNotFoundError> {
    return match params.get("path".to_string()) {
        Some(path) => Ok("https://github.com/".to_owned()),
        None => Err(ParamNotFoundError {
            param: "path".to_string(),
        }),
    };
}

fn log(s: &str) {
    eprintln!("{0}: {1}", crate_name!(), s);
}

#[derive(Debug, Clone)]
struct ParamNotFoundError {
    pub param: String,
}
