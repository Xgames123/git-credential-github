mod credhelper;
mod ghauth;

use crate::ghauth::AccessTokenPollError;
use clap::ArgAction::SetTrue;
use clap::{crate_authors, crate_name, crate_version, Parser, Subcommand};
use reqwest::Client;
use std::io::{Read, Write};

use std::string::String;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short = 'b', long)]
    //The backing credentails helper. The credentails will be stored here.
    backing_credhelper: String,

    #[arg(short = 'p', long)]
    //If set disables the startup prompt
    no_prompt: bool,

    #[command(subcommand)]
    operation: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    //Stores the credentials in the backing helper
    Store,
    //Deletes the credentials from the backing helper
    Erase,
    //Gets the stored credentials
    Get,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.operation {
        Store => {
            todo!()
        }

        Erase => {
            todo!()
        }

        Get => {
            if !cli.no_prompt {
                eprintln!("*******************************************************");
                eprintln!("*                       gh-login                      *");
                eprintln!("*     A simple git credentials helper for github      *");
                eprintln!("*******************************************************");
                eprintln!("NOTE: use --no-prompt to disable this message");
            }

            let process = credhelper::spawn(&cli.backing_credhelper, "get").unwrap();
            let mut stdout = process.stdout.unwrap();
            let mut stdin = process.stdin.unwrap();

            std::io::copy(&mut std::io::stdin(), &mut stdin)
                .expect("Error sending data to backing helper");

            let mut returnedParams = credhelper::params::from_stream(&mut stdout)
                .expect("Invalid data returned by backing helper");

            if !returnedParams.contains(String::from("password")){
                let client = reqwest::Client::new();
                let access_token = get_access_token_via_device_code(&client).await;

                returnedParams.add(String::from("password"), access_token.access_token);
            }

            returnedParams.write_to_sdtout();
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

fn get_repo_uri(params: &credhelper::params::Params) -> Result<String, ParamNotFoundError> {
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
