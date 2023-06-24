pub mod params;

use params::Params;
use std::fmt;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use log::{debug};

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

            let program_name = split[0];

            let cmd = Command::new(&program_name)
                .args(&split[1..])
                .arg(operation)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

            debug!("{}", cmd);

            let process = cmd.spawn()
                .map_err(|err| err.into());



            process
        })
}

pub fn run(helper: &str, operation: &str, params: Params) -> Result<Params> {
    let mut process = spawn(helper, operation)?;
    let mut stdin = process.stdin.take().unwrap();
    params.write_to(&mut stdin)?;
    drop(stdin);
    process.wait()?;

    let mut stdout = process.stdout.take().unwrap();

    let output_params = params::from_stream(&mut stdout)?;
    Ok(output_params)
}
