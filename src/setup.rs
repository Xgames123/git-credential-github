use itertools::Itertools;
use log::error;
use std::process::Command;

pub fn setup() -> Result<(), ()> {
    let output = Command::new("git")
        .arg("config")
        .arg("list")
        .arg("--show-origin")
        .arg("--null")
        .output()
        .map_err(|e| error!("Failed to run 'git config list': {}", e))?;

    if !output.status.success() {
        error!(
            "Git returned non 0 exit code:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(());
    }
    let output = String::from_utf8_lossy(&output.stdout);
    for (config_path, config) in output.split('\0').tuples() {
        let Some(helper) = parse_credential_config_key(config) else {
            continue;
        };

        // skip non github helpers
        if helper
            .url
            .map(|url| !url.contains("github.com"))
            .unwrap_or(false)
        {
            continue;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct CredHelper<'a> {
    url: Option<&'a str>,
    helper_name: &'a str,
    helper_args: &'a str,
}

fn parse_credential_config_key<'a>(config: &'a str) -> Option<CredHelper<'a>> {
    let (key, value) = config.split_once('\n')?;
    let (helper_name, helper_args) = value.split_once(' ').unwrap_or((value, ""));

    let mut split_key = key.split(".");
    if split_key.next() != Some("credential") {
        return None;
    }
    let next = split_key.next()?;
    if next == "helper" {
        Some(CredHelper {
            url: None,
            helper_name,
            helper_args,
        })
    } else if split_key.next()? == "helper" {
        Some(CredHelper {
            url: Some(next),
            helper_name,
            helper_args,
        })
    } else {
        None
    }
}
