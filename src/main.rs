pub mod credhelper;
pub mod ghauth;

use crate::ghauth::AccessTokenPollError;
use clap::{Parser, Subcommand};
use reqwest::Client;

use log::*;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::string::String;

#[derive(Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
#[command(about = "A simple git credentials helper for github", long_about = None)]
struct Cli {
    ///The backing credentials helper. The credentials will be stored here.
    #[arg(short = 'b', long, default_value = "")]
    backing_helper: String,

    ///If set disables the startup prompt
    #[arg(short = 'p', long)]
    no_prompt: bool,

    /// Verbosity of the logging
    #[arg(short = 'v', long, default_value = "0")]
    verbose: usize,

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

    stderrlog::new()
        .module(module_path!())
        .verbosity(cli.verbose)
        .init()
        .unwrap();

    let mut backing_helper: String = cli.backing_helper;
    if backing_helper == "" {
        backing_helper = match std::env::var("GHLOGIN_BACKINGHELPER").ok() {
            Some(helper) => String::from(helper),
            None => String::new(),
        }
    }
    if backing_helper == "" {
        error!("No backing helper set.\nUse the -b option or the GHLOGIN_BACKINGHELPER enviroment varible to set one");
    }

    match cli.operation {
        Commands::Store => {
            debug!("Storing credentials");

            let params = credhelper::params::from_stdin().expect("Failed to read data from stdin");

            debug!("Input params: '{}'", params);
            debug!("Running backing helper '{}'", &backing_helper);

            let output = credhelper::run(&backing_helper, "store", params)
                .expect("Failed to run backing helper");

            debug!("Done. Writing credentials to stdout");
            debug!("Output params: '{}'", output);

            output.write_to_sdtout().expect("Failed to write to stdout");
        }

        Commands::Erase => {
            debug!("Erasing credentials");

            let params = credhelper::params::from_stdin().expect("Failed to read data from stdin");

            debug!("Input params: '{}'", params);
            debug!("Running backing helper '{}'", &backing_helper);

            let output = credhelper::run(&backing_helper, "erase", params)
                .expect("Failed to run backing helper");

            debug!("Done. Writing credentials to stdout");
            debug!("Output params: '{}'", output);

            output.write_to_sdtout().expect("Failed to write to stdout");
        }

        Commands::Get => {
            if !cli.no_prompt {
                eprintln!("*******************************************************");
                eprintln!("*                       gh-login                      *");
                eprintln!("*     A simple git credentials helper for github      *");
                eprintln!("*******************************************************");
                eprintln!("NOTE: use --no-prompt to disable this message");
            }

            debug!("Reading parameters from stdin");

            let params = credhelper::params::from_stdin().expect("Failed to read data from stdin");

            debug!("Input params: '{}'", params);
            debug!("Running backing helper '{}'", &backing_helper);

            let mut output = credhelper::run(&backing_helper, "get", params)
                .expect("Failed to run backing helper");

            debug!(
                "Done running credentials helper '{}' output: {}",
                &backing_helper, output
            );

            if !output.contains("password") {
                debug!("No password returned by helper. Fetching credentials..");

                let client = Client::new();
                let access_token = get_access_token_via_device_code(&client).await;

                output.add(String::from("password"), access_token.access_token);
            }

            debug!("Output params: '{}'", output);
            debug!("Done. Writing credentials to stdout");

            output.write_to_sdtout().expect("Failed to write to stdout");
        }
    }
}

fn copy_clipboard(data: String) -> Result<(), String> {
    let mut command = Command::new("wl-copy");
    command.stdin(Stdio::piped());

    let mut process = command.spawn().map_err(|err| err.to_string())?;
    let mut stdin = process.stdin.take().unwrap();
    stdin.write(data.as_bytes()).unwrap();

    let status = process.wait().map_err(|err| err.to_string())?;
    if !status.success() {
        return Err(format!(
            "wl-copy exited with status code: {}",
            status.code().unwrap()
        ));
    }

    Ok(())
}

async fn get_access_token_via_device_code(client: &Client) -> ghauth::AccessToken {
    let device_code = get_device_code(client).await;

    loop {
        break match ghauth::poll_for_access_token(client, &device_code).await {
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
    }
}

async fn get_device_code(client: &Client) -> ghauth::DeviceCode {
    eprintln!("Getting login code...");
    let device_code = ghauth::get_device_code(client).await.unwrap_or_else(|err| {
        panic!("Failed to get device code \n Err: {}", err);
    });

    eprintln!("gh-login: Go to the link below and enter in the device code");
    eprintln!("{}", device_code.verification_uri);
    eprintln!("device code: {}", device_code.user_code);
    if let Some(error) = copy_clipboard(device_code.user_code.to_string()).err() {
        warn!("Could not copy to clipboard: {}", error);
    }
    device_code
}

#[derive(Debug, Clone)]
pub struct ParamNotFoundError {
    pub param: String,
}
