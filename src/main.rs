mod credstore;
mod ghauth;
mod params;

use crate::ghauth::AccessTokenPollError;
use clap::ArgAction::SetTrue;
use clap::{crate_authors, crate_name, crate_version};
use params::Params;
use reqwest::Client;
use std::string::String;

#[tokio::main]
async fn main() {
    let args = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about("A simple git credentials manager for github")
        .arg(clap::Arg::new("no-prompt").short('p').action(SetTrue))
        .arg(
            clap::Arg::new("no-store")
                .help("If set don't write or read from the credentials store")
                .short('c')
                .action(SetTrue),
        )
        .subcommand(clap::Command::new("get").about("Gets the stored credentials"))
        .subcommand(
            clap::Command::new("store").about("Stores the credentials into the backing store"),
        )
        .subcommand(clap::Command::new("erase").about("Deletes all the stored credentials"))
        .get_matches();

    let mut params = Params::from_stdin().expect("Failed to read parameters returned by git {}");

    match params.get("host".to_string()) {
        Some(host) => {
            if host != "github.com" {
                eprintln!("host '{}' is not github.com. Exiting...", host);
                return;
            }
        }
        None => {
            eprintln!("No host given by git. Exiting...");
            return;
        }
    };

    match params.get("protocol".to_string()) {
        Some(protocol) => {
            if protocol != "https" {
                eprintln!("Unsupported protocol '{}'. Exiting...", protocol);
                return;
            }
        }
        None => {
            eprintln!("No protocol given by git. Exiting...");
            return;
        }
    };

    let use_credstore: bool = !args.get_flag("no-store");
    let credstore;
    if use_credstore{
        match credstore::CredStore::new() {
            Ok(store) => {
                credstore = store;
            }
            Err(err) => {
                use_credstore = false;
                eprintln!("{}", err);
            }
        }
    }

    match args.subcommand() {
        Some(("store", _)) => {
            if args.get_flag("no-store") {
                return;
            }

            eprintln!("gh-login: saving credentials");

            let username = match params.get(String::from("username")) {
                None => {
                    eprintln!("no username returned by git");
                    return;
                }
                Some(username) => username,
            };

            let password = match params.get(String::from("password")) {
                None => {
                    eprintln!("no password returned by git");
                    return;
                }
                Some(password) => password,
            };

            if use_credstore {
                credstore.store(username, password).unwrap();
            }
        }

        Some(("erase", _)) => {
            if args.get_flag("no-store") {
                return;
            }

            eprintln!("gh-login: deleting credentials");

            if use_credstore {
                credstore.delete().expect("Failed to delete access key from credentials store");
            }

        }

        Some(("get", _)) => {
            if !args.get_flag("no-prompt") {
                eprintln!("*******************************************************");
                eprintln!("*                       gh-login                      *");
                eprintln!("*     A simple git credentials manager for github     *");
                eprintln!("*******************************************************");
                eprintln!("NOTE: use --no-prompt to disable this message");
            }

            //let path = params.get(String::from("path")).expect("path variable not given by git");

            //let username = get_username_from_repo_path(path);

            if use_credstore {
                match credstore.get() {
                    Ok((username, password)) => {
                        params.add(String::from("username"), username.to_string());
                        params.add(String::from("password"), password.to_string());
                    }
                    Err(err) => {
                        eprintln!("Failed to get password from credentials store. {}", err);
                    }
                }
            }

            let client = reqwest::Client::new();
            let access_token = get_access_token_via_device_code(&client).await;

            //params.add(String::from("username"), username.to_string());
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
                    eprintln!("Device code expired");
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
    eprintln!("Getting login code...");
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
        Some(path) => Ok("https://github.com/".to_owned() + path),
        None => Err(ParamNotFoundError {
            param: "path".to_string(),
        }),
    };
}

fn get_username_from_repo_path(path: &str) -> &str {
    for (i, &character) in path.as_bytes().iter().enumerate() {
        if character == b'/' {
            return &path[..i];
        }
    }

    return path;
}

#[derive(Debug, Clone)]
struct ParamNotFoundError {
    pub param: String,
}
