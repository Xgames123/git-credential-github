mod credhelper;
mod ghauth;

use crate::ghauth::AccessTokenPollError;
use clap::{Parser, Subcommand};
use reqwest::Client;

use std::{io::Read, string::String};

#[derive(Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
#[command(about = "A simple git credentials helper for github", long_about = None)]
struct Cli {
    ///The backing credentails helper. The credentails will be stored here.
    #[arg(short = 'b', long)]
    backing_helper: String,

    ///If set disables the startup prompt
    #[arg(short = 'p', long)]
    no_prompt: bool,

    ///If set logs everything gh-login is doing to stderr
    #[arg(short = 'l', long)]
    log: bool,

    #[command(subcommand)]
    operation: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///Stores the credentials in the backing helper
    Store,
    ///Deletes the credentials from the backing helper
    Erase,
    ///Gets the stored credentials
    Get,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.operation {
        Commands::Store => {
            todo!()
        }

        Commands::Erase => {
            todo!()
        }

        Commands::Get => {
            if !cli.no_prompt {
                eprintln!("*******************************************************");
                eprintln!("*                       gh-login                      *");
                eprintln!("*     A simple git credentials helper for github      *");
                eprintln!("*******************************************************");
                eprintln!("NOTE: use --no-prompt to disable this message");
            }

            if cli.log {
                eprintln!("Reading parameters from stdin");
            }
            let params = credhelper::params::from_stdin().expect("Failled to read data from stdin");

            if cli.log {
                eprintln!("Running backing helper '{}'", &cli.backing_helper);
            }
            let mut output = credhelper::run(&cli.backing_helper, "get", params)
                .expect("Failled to run backing helper");

            if cli.log {
                eprintln!("Done running credentials helper '{}'", &cli.backing_helper);
            }

            if !output.contains(String::from("password")) {
                if cli.log {
                    eprintln!("No password returned by helper. Fetching credentails..");
                }
                let client = reqwest::Client::new();
                let access_token = get_access_token_via_device_code(&client).await;

                output.add(String::from("password"), access_token.access_token);
            }

            if cli.log {
                eprintln!("Done. Writing credentails to stdout");
            }

            output
                .write_to_sdtout()
                .expect("Failled to write to stdout");
        }
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

#[derive(Debug, Clone)]
struct ParamNotFoundError {
    pub param: String,
}
