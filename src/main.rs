pub mod credhelper;
pub mod ghauth;

use crate::ghauth::AccessTokenPollError;
use clap::{crate_name, crate_version, Args, Parser, Subcommand};
use reqwest::Client;

use core::fmt;
use log::*;
use std::io::{self, ErrorKind, Write};
use std::process::{Command, ExitCode, Stdio};
use std::string::String;
use stderrlog::LogLevelNum;

const PROMPT_SIZE: usize = 55;
const GHLOGIN_BACKINGHELPER: &str = "GHLOGIN_BACKINGHELPER";

#[derive(Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
#[command(about = "A simple git credentials helper for GitHub", long_about = None)]
struct Cli {
    ///The backing credentials helper. The credentials will be stored here.
    #[arg(short = 'b', long, default_value = "")]
    backing_helper: String,

    ///Disables the startup prompt
    #[arg(short = 'p', long)]
    no_prompt: bool,

    #[command(flatten)]
    verbosity: Verbosity,

    ///Disables opening the verification url in a browser
    #[arg(long)]
    no_open_url: bool,

    ///Don't copy the device code to the clipboard
    #[arg(long)]
    no_clip: bool,

    #[command(subcommand)]
    operation: Commands,
}
#[derive(Args)]
struct Verbosity {
    ///Suppress most output
    #[arg(long, global = true, short = 'q')]
    quiet: bool,

    ///More verbose logging
    #[arg(long, action=clap::ArgAction::Count, global = true, short = 'v')]
    verbose: u8,
}
impl Verbosity {
    fn log_level(&self) -> LogLevelNum {
        if self.quiet {
            return LogLevelNum::Off;
        }
        if self.verbose == 1 {
            return LogLevelNum::Debug;
        }
        if self.verbose > 1 {
            return LogLevelNum::Trace;
        }

        LogLevelNum::Info
    }

    fn is_quied(&self) -> bool {
        self.quiet
    }
}
impl fmt::Display for Verbosity {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self.log_level() {
            LogLevelNum::Off => "QUIET",
            LogLevelNum::Error => "ERROR",
            LogLevelNum::Warn => "WARN",
            LogLevelNum::Info => "INFO",
            LogLevelNum::Debug => "DEBUG",
            LogLevelNum::Trace => "TRACE",
        })
    }
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
async fn main() -> ExitCode {
    let cli = Cli::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(cli.verbosity.log_level())
        .init()
        .unwrap();

    let mut backing_helper: String = cli.backing_helper;
    if backing_helper == "" {
        backing_helper = std::env::var(GHLOGIN_BACKINGHELPER).unwrap_or(String::new());
    }

    if backing_helper == "" {
        error!("No backing helper set use the -b option or the {GHLOGIN_BACKINGHELPER} environment variable");
        return ExitCode::FAILURE;
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
            ExitCode::SUCCESS
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
            ExitCode::SUCCESS
        }

        Commands::Get => {
            if !cli.no_prompt {
                print_prompt(&cli.verbosity, &backing_helper);
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
                let device_code = ghauth::get_device_code(&client).await.unwrap();
                print_device_code(
                    &device_code,
                    &cli.verbosity,
                    !&cli.no_clip,
                    !&cli.no_open_url,
                );
                let access_token = loop {
                    break match ghauth::poll_for_access_token(&client, &device_code).await {
                        Ok(token) => token,
                        Err(err) => match err {
                            AccessTokenPollError::DeviceCodeExpired => {
                                info!("Device code expired");
                                continue;
                            }
                            AccessTokenPollError::Reqwest(error) => {
                                panic!("{}", error);
                            }
                        },
                    };
                };

                output.add(String::from("password"), access_token.access_token);
            }

            debug!("Output params: '{}'", output);
            debug!("Done. Writing credentials to stdout");

            output.write_to_sdtout().expect("Failed to write to stdout");
            ExitCode::SUCCESS
        }
    }
}

fn print_prompt(verboseity: &Verbosity, backing_helper: &str) {
    if verboseity.is_quied() {
        return;
    }
    print_prompt_line();
    print_prompt_text(format!("{} v{}", crate_name!(), crate_version!()).as_str());
    print_prompt_text("A simple git credentials helper for GitHub");
    print_prompt_line();
    eprintln!("Verbosity: {}", verboseity);
    eprintln!("Backing: {}", backing_helper);
    eprintln!("NOTE: use --no-prompt to disable this message");
}
fn print_prompt_line() {
    eprintln!("*{:*^1$}*", "", PROMPT_SIZE);
}
fn print_prompt_text(text: &str) {
    eprintln!("*{: ^1$}*", text, PROMPT_SIZE);
}

fn copy_clipboard(command: &str, data: &str) -> Result<(), io::Error> {
    debug!("copy_clipboard(\"{}\")", data);
    let mut command = Command::new(command);
    command.stdin(Stdio::piped());

    let mut process = command.spawn()?;
    let mut stdin = process.stdin.take().unwrap();
    stdin.write_all(data.as_bytes()).unwrap();
    drop(stdin);
    let status = process.wait().unwrap();
    debug!("{}", status);
    Ok(())
}

fn xdg_open(program: &str) -> Result<(), io::Error> {
    debug!("xdg_open(\"{}\")", program);
    Command::new("xdg-open").arg(program).spawn()?;
    Ok(())
}

fn print_device_code(
    device_code: &ghauth::DeviceCode,
    verboseity: &Verbosity,
    copy_clip: bool,
    open_url: bool,
) {
    if !verboseity.is_quied() {
        eprintln!("Go to the link below and enter in the device code");
        eprintln!("{}", device_code.verification_uri.to_string());
    }
    eprintln!("device code: {}", device_code.user_code.to_string());
    if copy_clip {
        match copy_clipboard("wl-copy", &device_code.user_code) {
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    warn!("Could not copy to clipboard: {}", error);
                }
            }
            Ok(()) => info!("Device code copied to clipboard"),
        }
        match copy_clipboard("xclip", &device_code.user_code) {
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    warn!("Could not copy to clipboard: {}", error);
                }
            }
            Ok(()) => info!("Device code copied to clipboard"),
        }
    }
    if open_url {
        if let Err(error) = xdg_open(&device_code.verification_uri) {
            if error.kind() != ErrorKind::NotFound {
                warn!("Could not start xdg_open: {}", error);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamNotFoundError {
    pub param: String,
}
