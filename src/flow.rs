use crate::ghauth::{self, AccessTokenPollError};
use log::*;
use std::{
    collections::HashMap,
    io::{ErrorKind, Write},
    process::{Command, Stdio},
};
use ureq::Agent;

pub fn fetch_creds(
    output: &mut HashMap<String, String>,
    no_clip: bool,
    no_open_url: bool,
    quied: bool,
) {
    debug!("Fetching credentials...");
    let agent = Agent::new_with_defaults();
    let device_code = ghauth::get_device_code(&agent).unwrap();

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
        break match ghauth::poll_for_access_token(&agent, &device_code) {
            Ok(token) => token,
            Err(err) => match err {
                AccessTokenPollError::DeviceCodeExpired => {
                    info!("Device code expired");
                    continue;
                }
                e => {
                    panic!("Request to Github failed: {}", e);
                }
            },
        };
    };

    output.insert(String::from("password"), access_token.access_token);
}
fn spawn_with_stdin(program: &str, data: &str) -> std::io::Result<()> {
    let mut process = Command::new(program).stdin(Stdio::piped()).spawn()?;

    let mut stdin = process.stdin.take().unwrap();
    stdin.write_all(data.as_bytes())?;
    drop(stdin);
    let status = process.wait()?;
    debug!("{}", status);
    Ok(())
}

/// Returns true when the program executed successfully.
fn try_clip(program: &str, data: &str) -> bool {
    match spawn_with_stdin(program, data) {
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to execute clipboard program '{}': {}", program, e);
            }
            false
        }
        Ok(()) => true,
    }
}

#[cfg(not(target_os = "windows"))]
fn copy_clipboard(data: &str) {
    if try_clip("wl-copy", data) {
        return;
    }
    if try_clip("xclip", data) {
        return;
    }
    error!("Failed to copy device code to clipboard because no clipboard program worked");
}

#[cfg(target_os = "windows")]
fn copy_clipboard(data: &str) {
    clipboard_win::set_clipboard(clipboard_win::formats::Unicode, data)
        .unwrap_or_else(|e| error!("Failed to copy device code to clipboard: ", e));
}

fn open_url(url: &str) {
    debug!("open");
    open::that(url).unwrap_or_else(|err| {
        warn!("Could not start open url\n{}", err);
    });
}
