use std::collections::HashMap;
use std::io;
use std::process::{Child, Command, Stdio};
use thiserror::Error;

use log::{debug, error};

use crate::{paramparsing, Operation};

fn spawn_helper(helper: &str, operation: Operation) -> io::Result<Child> {
    let helpercmd = helper.trim();
    let helpercmd = if helpercmd.starts_with('/') {
        String::from(helpercmd)
    } else {
        format!("git credential-{}", helpercmd)
    };

    debug!("Running credential helper '{}'", helpercmd);
    let split = shlex::split(&helpercmd).unwrap();
    let program_name = &split[0];

    let mut cmd = Command::new(program_name);
    cmd.args(&split[1..])
        .arg(operation.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    Ok(cmd.spawn()?)
}

#[derive(Error, Debug)]
pub enum CredHelperError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("credential helper exited with code {0}")]
    Non0ExitCode(i32, Option<HashMap<String, String>>),
}

pub fn run(
    helper: &str,
    operation: Operation,
    params: &HashMap<String, String>,
) -> Result<HashMap<String, String>, CredHelperError> {
    let mut process = spawn_helper(helper, operation)?;
    let mut stdin = process.stdin.take().unwrap();
    paramparsing::write_to(&params, &mut stdin)?;
    drop(stdin);

    let mut stdout = process.stdout.take().unwrap();
    let output_params = paramparsing::parse_from(&mut stdout)?;
    drop(stdout);
    let output = process.wait()?;

    if !output.success() {
        Err(CredHelperError::Non0ExitCode(
            output.code().unwrap_or_default(),
            Some(output_params),
        ))
    } else {
        Ok(output_params)
    }
}
