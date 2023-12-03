use crate::ghauth::AccessTokenPollError;
use clap::{crate_name, crate_version, Parser, Subcommand};
use log::*;
use reqwest::Client;
use std::fmt::Display;
use std::io::{self, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::string::String;

mod credhelper;
mod ghauth;
mod paramparsing;
mod utils;
mod verbosity;

const PROMPT_SIZE: usize = 55;
const GHLOGIN_BACKINGHELPER: &str = "GHLOGIN_BACKINGHELPER";
const GCG_BACKINGHELPER: &str = "GCG_BACKINGHELPER";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    ///The backing credentials helper. The credentials will be stored here.
    #[arg(short = 'b', long)]
    backing_helper: Option<String>,

    ///Disables the startup prompt
    #[arg(short = 'p', long)]
    no_prompt: bool,

    #[command(flatten)]
    verbosity: verbosity::Verbosity,

    ///Disables opening the verification url in a browser
    #[arg(long)]
    no_open_url: bool,

    ///Don't copy the device code to the clipboard
    #[arg(long)]
    no_clip: bool,

    /// Go through the authentication process even if not needed. (only for get operation)
    #[arg(long)]
    auth: bool,

    /// Don't authenticate when the credential helper returns a non 0 exit code
    #[arg(long)]
    no_auth_on_fail: bool,

    #[command(subcommand)]
    operation: Operation,
}

#[derive(Subcommand, Copy, Clone)]
pub enum Operation {
    ///Stores the credentials in the backing helper
    Store,
    ///Deletes the credentials from the backing helper
    Erase,
    ///Gets the stored credentials
    Get,
}
impl Display for Operation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => fmt.write_str("get"),
            Self::Erase => fmt.write_str("erase"),
            Self::Store => fmt.write_str("store"),
        }
    }
}
impl Operation {
    pub fn is_get(&self) -> bool {
        match self {
            Operation::Get => true,
            _ => false,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(cli.verbosity.log_level())
        .init()
        .unwrap();

    let backing_helper = cli.backing_helper
        .or_else(|| std::env::var(GHLOGIN_BACKINGHELPER).ok())
        .or_else(|| std::env::var(GCG_BACKINGHELPER).ok()).unwrap_or_else(||{
            die!("No backing helper set use the -b option or the {GCG_BACKINGHELPER} environment variable");
        });

    let params = paramparsing::parse_from_stdin().unwrap_or_else(|err| {
        die!("Failed to read data from stdin\n{}", err);
    });

    debug!("backing_helper={}", &backing_helper);
    debug!("params={:?}", params);
    debug!("operation={}", cli.operation);

    if cli.operation.is_get() && !cli.no_prompt && !cli.verbosity.is_quied() {
        print_prompt();
    }

    let mut output =
        credhelper::run(&backing_helper, cli.operation, &params).unwrap_or_else(|err| match err {
            credhelper::CredHelperError::Io(err) => {
                die!(
                    "Failed to run credential helper '{}'\n{}",
                    backing_helper,
                    err
                );
            }
            credhelper::CredHelperError::Non0ExitCode(code, output) => {
                error!(
                    "Credential helper '{}' exited with code: {}",
                    backing_helper, code
                );
                if cli.no_auth_on_fail {
                    die!("See Last error");
                }

                output.unwrap_or_default()
            }
        });

    if cli.operation.is_get() && (!output.contains_key("password") || cli.auth) {
        debug!("Fetching credentials..");
        let client = Client::new();
        let device_code = ghauth::get_device_code(&client).await.unwrap();

        if !cli.verbosity.is_quied() {
            eprintln!("Go to the link below and enter in the device code");
            eprintln!("{}", device_code.verification_uri.to_string());
        }
        eprintln!("device code: {}", device_code.user_code.to_string());
        if !cli.no_clip {
            copy_clipboard(&device_code.user_code);
        }
        if !cli.no_open_url {
            xdg_open(&device_code.verification_uri);
        }

        let access_token = loop {
            break match ghauth::poll_for_access_token(&client, &device_code).await {
                Ok(token) => token,
                Err(err) => match err {
                    AccessTokenPollError::DeviceCodeExpired => {
                        info!("Device code expired");
                        continue;
                    }
                    AccessTokenPollError::Reqwest(err) => {
                        panic!("{}", err);
                    }
                },
            };
        };

        output.insert(String::from("password"), access_token.access_token);
    }

    debug!("Done. Writing credentials to stdout");
    debug!("Output params: '{:?}'", output);

    paramparsing::write_to_stdout(&output);
}

fn print_prompt() {
    print_prompt_line();
    print_prompt_text(format!("{} v{}", crate_name!(), crate_version!()).as_str());
    print_prompt_text("A simple git credentials helper for GitHub");
    print_prompt_line();
    eprintln!("NOTE: use --no-prompt to disable this message");
}
fn print_prompt_line() {
    eprintln!("*{:*^1$}*", "", PROMPT_SIZE);
}
fn print_prompt_text(text: &str) {
    eprintln!("*{: ^1$}*", text, PROMPT_SIZE);
}

fn run_clip_prog(program: &str, data: &str) -> io::Result<()> {
    let mut command = Command::new(program);
    command.stdin(Stdio::piped());

    let mut process = command.spawn()?;
    let mut stdin = process.stdin.take().unwrap();
    stdin.write_all(data.as_bytes()).unwrap();
    drop(stdin);
    let status = process.wait().unwrap();
    debug!("{}", status);
    Ok(())
}

fn copy_clipboard(data: &str) {
    run_clip_prog("wl-copy", data).unwrap_or_else(|err| {
        if err.kind() != ErrorKind::NotFound {
            warn!("Could not copy to clipboard (wl-copy)\n{}", err);
        }
        run_clip_prog("xclip", data).unwrap_or_else(|err| {
            warn!("Could not copy to clipboard (xclip)\n{}", err);
        })
    })
}

fn xdg_open(url: &str) {
    debug!("xdg_open {}", url);
    let _ = Command::new("xdg-open").arg(url).spawn().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            return;
        }
        warn!("Could not start xdg-open\n{}", err);
    });
}
