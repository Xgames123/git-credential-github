pub mod params;

use params::Params;
use std::fmt;
use std::process::{Child, Command, Stdio};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
struct InvalidHelper;

impl fmt::Display for InvalidHelper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid credential helper name")
    }
}
impl std::error::Error for InvalidHelper {}

pub fn spawn(helper: &str, operation: &str) -> Result<Child> {
    let mut helpercmd;
    if helper.starts_with('/') {
        helpercmd = String::from(helper);
    } else {
        helpercmd = String::from("git credential-");
        helpercmd.push_str(helper);
    }

    shlex::split(&helpercmd)
        .ok_or_else(|| InvalidHelper.into())
        .and_then(|split| {
            let cmd = Command::new(&split[0])
                .args(&split[1..])
                .arg(operation)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|err| err.into());

            cmd
        })
}

pub fn run(helper: &str, operation: &str, params: Params) -> Result<params::Params> {
    let process = spawn(helper, operation)?;
    eprintln!("Opening stdin of helper");
    let mut stdin = process.stdin.unwrap();
    eprintln!("Writing to stdin of helper");
    params.write_to(&mut stdin)?;

    eprintln!("Opening to stdout of helper");
    let stdout = process.stdout.unwrap();
    let output = params::from_stream(stdout)?;
    eprintln!("Parsing stdout of helper");
    Ok(output)
}
