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
                .arg(operation)
                .args(&split[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|err| err.into());

            cmd
        })
}
