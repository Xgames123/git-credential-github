use clap::crate_name;
use clap::Command;
use std::collections::HashMap;
use std::io::{self, BufRead};
use params::Params;

static oauth_client_id: &str = "0120e057bd645470c1ed";
static oatuh_client_secret: &str = "18867509d956965542b521a529a79bb883344c90";
static oauth_scope: &str = "repo";

fn main() {
    let args = Command::new(crate_name!())
        .author("ldev")
        .version("1.0.0")
        .about("A simple git credentials manager for github")
        .subcommand(Command::new("get").about("Gets the stored credentials"))
        .get_matches();

    match args.subcommand() {
        Some(("get", _)) => {
            let mut params = HashMap::new();
            parse_sdtin(&mut params);

            match params.get("host") {
                Some(host) => {
                    if host != "github.com" {
                        return;
                    }
                }
                None => {
                    return;
                }
            };

            match params.get("protocol") {
                Some(protocol) => {
                    if protocol == "http" {
                        log("http is not supported. Use https instead");
                        return;
                    }
                    if protocol != "https" {
                        return;
                    }
                }
                None => {
                    return;
                }
            };

            let username = params
                .get("username")
                .except("No username parameter given by git");

            let repoUri = get_repo_uri(&params).expect("Parameter not given by git");

            auth_via_tty(repoUri, username);
        }
        _ => {}
    }
}

fn auth_via_tty(repo_uri: String, username: String) {
    let client = reqwest::Client::new();
	let deviceCode = get_device_code(&client);
}

fn get_device_code(client: &reqwest::Client) {
    let form_params = [("client_id", oauth_client_id), ("scope", oauth_scope)];
    let mut request = client.post("https://github.com/login/device/code")
		.form(&form_params)
		.send()
		.await?;
	request.error_for_status()?;
	
}


fn get_repo_uri(params: HashMap<String, String>) -> Result<String, ParamNotFoundError> {
    match params.get("path") {
        Some(path) => {
            return "https://github.com/" + path;
        }
        None => {
           Err(ParamNotFoundError { param:"path"});
        }
    }
}

fn log(s: &str) {
    eprintln!("{0}: {1}", crate_name!(), s);
}

fn parse_sdtin(hashmap: &mut HashMap<String, String>) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut buffer = String::new();

        handle.read_line(&mut buffer).unwrap();

        if buffer == "" {
            break;
        }

        let (param_key, param_value) = split_param(&buffer).unwrap_or_else(|_| {
            panic!(
                "Invalid data from stdin. Expected: 'key=value' Got: '{}'",
                buffer
            );
        });
        hashmap.insert(param_key.to_string(), param_value.to_string());
    }
}
#[derive(Debug, Clone)]
struct ParamNotFoundError{
	param : &str
}






fn split_param(param: &str) -> Result<(&str, &str), InvalidParamError> {
    for (i, &character) in param.as_bytes().iter().enumerate() {
        if character == b'=' {
            return Ok((&param[..i], &param[(i + 1)..]));
        }
    }
    return Err(InvalidParamError {param=param});
}
