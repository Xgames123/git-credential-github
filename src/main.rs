use crate::ghauth::AccessTokenPollError;
use clap::{crate_name, crate_version, Parser, Subcommand};
use log::*;
use reqwest::Client;
use std::collections::HashMap;
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
    ///DEPRECATED: The backing credentials helper. The credentials will be stored here.
    #[arg(short = 'b', long)]
    backing_helper: Option<String>,

    /// Disables the startup prompt
    #[arg(short = 'p', long)]
    no_prompt: bool,

    #[command(flatten)]
    verbosity: verbosity::Verbosity,

    /// Disables opening the verification url in a browser
    #[arg(long)]
    no_open_url: bool,

    /// Don't copy the device code to the clipboard
    #[arg(long)]
    no_clip: bool,

    ///DEPRECATED: Always go through the authentication process even if not needed. (only for get operation)
    #[arg(long)]
    auth: bool,

    ///DEPRECATED: Don't authenticate when the credential helper returns a non 0 exit code
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

    let backing_helper = cli
        .backing_helper
        .or_else(|| std::env::var(GHLOGIN_BACKINGHELPER).ok())
        .or_else(|| std::env::var(GCG_BACKINGHELPER).ok());

    let params = paramparsing::parse_from_stdin().unwrap_or_else(|err| {
        die!("Failed to read data from stdin\n{}", err);
    });

    if let Some(start_path) = std::env::args().nth(0) {
        if start_path.ends_with("git-credential-gh-login") {
            warn!("gh-login is deprecated. Change gh-login to github in your .gitconfig");
        }
    }

    debug!(
        "running_from={}",
        std::env::args().nth(0).unwrap_or(String::new())
    );
    debug!("params={:?}", params);
    debug!("operation={}", cli.operation);

    if cli.operation.is_get() && !cli.no_prompt && !cli.verbosity.is_quied() {
        print_prompt();
    }

    if cli.auth {
        warn!("--auth is deprecated.");
    }
    if cli.no_auth_on_fail {
        warn!("--no-auth-on-fail is deprecated.");
    }

    let output = if let Some(backing_helper) = backing_helper {
        debug!("backing_helper={}", &backing_helper);
        warn!("--backing-helper, GCG_BACKINGHELPER and GHLOGIN_BACKINGHELPER are deprecated.\nAdd 'github' to the END of your .gitconfig credentials helper list.\n\nSee readme for more info: https://github.com/Xgames123/git-credential-github\n");
        let mut output = credhelper::run(&backing_helper, cli.operation, &params).unwrap_or_else(
            |err| match err {
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
            },
        );

        if cli.operation.is_get() && (!output.contains_key("password") || cli.auth) {
            fetch_creds(
                &mut output,
                cli.no_clip,
                cli.no_open_url,
                cli.verbosity.is_quied(),
            )
            .await;
        };

        output
    } else {
        let mut output = HashMap::new();
        if cli.operation.is_get() {
            fetch_creds(
                &mut output,
                cli.no_clip,
                cli.no_open_url,
                cli.verbosity.is_quied(),
            )
            .await;
        }

        output
    };

    debug!("Done. Writing credentials to stdout");
    debug!("Output params: '{:?}'", output);

    paramparsing::write_to_stdout(&output);
}

async fn fetch_creds(
    output: &mut HashMap<String, String>,
    no_clip: bool,
    no_open_url: bool,
    quied: bool,
) {
    debug!("Fetching credentials..");
    let client = Client::new();
    let device_code = ghauth::get_device_code(&client).await.unwrap();

    if !quied {
        eprintln!("Go to the link below and enter in the device code");
        eprintln!("{}", device_code.verification_uri.to_string());
    }
    eprintln!("device code: {}", device_code.user_code.to_string());
    if !no_clip {
        copy_clipboard(&device_code.user_code);
    }
    if !no_open_url {
        open_url(&device_code.verification_uri);
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

fn open_url(url: &str) {
    debug!("open");
    open::that(url).unwrap_or_else(|err| {
        warn!("Could not start open url\n{}", err);
    });
}
