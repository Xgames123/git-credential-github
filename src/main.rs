use clap::{crate_name, crate_version, Parser, Subcommand};
use log::*;
use std::{collections::HashMap, fmt::Display, string::String};

mod credhelper;
mod flow;
mod ghauth;
mod params;
mod verbosity;

const PROMPT_SIZE: usize = 55;
const GHLOGIN_BACKINGHELPER: &str = "GHLOGIN_BACKINGHELPER";
const GCG_BACKINGHELPER: &str = "GCG_BACKINGHELPER";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    ///DEPRECATED: The backing credentials helper. The credentials will be stored here.
    #[arg(hide = true, short = 'b', long)]
    backing_helper: Option<String>,

    ///DEPRECATED: Always go through the authentication process even if not needed. (only for get operation)
    #[arg(hide = true, long)]
    auth: bool,

    ///DEPRECATED: Don't authenticate when the credential helper returns a non 0 exit code
    #[arg(hide = true, long)]
    no_auth_on_fail: bool,

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

    #[command(subcommand)]
    operation: Operation,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Operation {
    ///Stores the credentials in the backing helper
    Store,
    ///Deletes the credentials from the backing helper
    Erase,
    ///Gets the stored credentials
    Get,
}
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Get => "get",
            Self::Erase => "erase",
            Self::Store => "store",
        })
    }
}

macro_rules! die {
    ($($arg:tt)+) => {log::error!("FATAL: {}", format!($($arg)+)); std::process::exit(1);}
}

fn main() {
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

    let params = params::parse_from_stdin().unwrap_or_else(|err| {
        die!("Failed to read parameters from stdin\n{}", err);
    });

    let start_path = std::env::args().nth(0);
    if let Some(start_path) = &start_path {
        if start_path.ends_with("git-credential-gh-login") {
            warn!("gh-login is deprecated. Change gh-login to github in your .gitconfig");
        }
    }

    debug!("running_from={:?}", start_path);
    debug!("params={:?}", params);
    debug!("operation={:?}", cli.operation);

    if cli.operation == Operation::Get && !cli.no_prompt && !cli.verbosity.is_quied() {
        print_prompt();
    }

    if cli.auth {
        warn!("--auth is deprecated.");
    }
    if cli.no_auth_on_fail {
        warn!("--no-auth-on-fail is deprecated.");
    }

    let output = if let Some(backing_helper) = backing_helper {
        legacy(
            cli.operation,
            cli.no_auth_on_fail,
            cli.auth,
            cli.no_clip,
            cli.no_open_url,
            cli.verbosity.is_quied(),
            &backing_helper,
            &params,
        )
    } else {
        let mut output = HashMap::new();
        if cli.operation == Operation::Get {
            flow::fetch_creds(
                &mut output,
                cli.no_clip,
                cli.no_open_url,
                cli.verbosity.is_quied(),
            );
        }

        output
    };

    debug!("Done. Writing credentials to stdout");
    debug!("Output params: '{:?}'", output);

    params::write_to_stdout(&output);
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

fn legacy(
    op: Operation,
    no_auth_on_fail: bool,
    auth: bool,
    no_clip: bool,
    no_open_url: bool,
    quiet: bool,
    backing_helper: &str,
    params: &HashMap<String, String>,
) -> HashMap<String, String> {
    warn!("--backing-helper, GCG_BACKINGHELPER and GHLOGIN_BACKINGHELPER are deprecated.\nAdd 'github' to the END of your .gitconfig credentials helper list.\n\nSee readme for more info: https://github.com/Xgames123/git-credential-github\n");
    debug!("backing_helper={}", &backing_helper);
    let mut output =
        credhelper::run(&backing_helper, op, &params).unwrap_or_else(|err| match err {
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
                if no_auth_on_fail {
                    die!("See Last error");
                }

                output.unwrap_or_default()
            }
        });

    if op == Operation::Get && (!output.contains_key("password") || auth) {
        flow::fetch_creds(&mut output, no_clip, no_open_url, quiet);
    };

    output
}
