use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::{fmt, io};

use log::debug;

use crate::{paramparsing, Operation};

#[derive(Debug, Clone)]
struct InvalidHelper;

impl fmt::Display for InvalidHelper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid credential helper name")
    }
}
impl std::error::Error for InvalidHelper {}

fn spawn_helper(helper: &str, operation: Operation) -> io::Result<Child> {
    let helpercmd = if helper.starts_with('/') {
        String::from(helper)
    } else {
        let mut helpercmd = String::from("git credential-");
        helpercmd.push_str(helper);
        helper.to_string()
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

pub fn run(
    helper: &str,
    operation: Operation,
    params: &HashMap<String, String>,
) -> io::Result<HashMap<String, String>> {
    let mut process = spawn_helper(helper, operation)?;
    let mut stdin = process.stdin.take().unwrap();
    paramparsing::write_to(&params, &mut stdin)?;
    drop(stdin);

    let mut stdout = process.stdout.take().unwrap();
    let output_params = paramparsing::parse_from(&mut stdout)?;
    drop(stdout);
    process.wait()?;
    Ok(output_params)
}
