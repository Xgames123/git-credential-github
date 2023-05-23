mod clientextentions;
mod params;

use clap::crate_name;
use clap::Command;
use params::Params;


static OAUTH_CLIENT_ID: &str = "0120e057bd645470c1ed";
static OATUH_CLIENT_SECRET: &str = "18867509d956965542b521a529a79bb883344c90";
static OAUTH_SCOPE: &str = "repo";

fn main() {
    let args = Command::new(crate_name!())
        .author("ldev")
        .version("1.0.0")
        .about("A simple git credentials manager for github")
        .subcommand(Command::new("get").about("Gets the stored credentials"))
        .get_matches();

    match args.subcommand() {
        Some(("get", _)) => {
            let params =
                Params::from_stdin().expect("Failed to read parameters returned by git {}");

            match params.get("host".to_string()) {
                Some(host) => {
                    if host != "github.com" {
                        return;
                    }
                }
                None => {
                    return;
                }
            };

            match params.get("protocol".to_string()) {
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
                .get("username".to_string())
                .expect("No username parameter given by git");

            let repoUri = get_repo_uri(&params).expect("Parameter not given by git");

            auth_via_tty(repoUri, &username);
        }
        _ => {}
    }
}

fn auth_via_tty(repo_uri: String, username: &String) {
    let client = reqwest::Client::new();
    let deviceCode = get_device_code(&client);
}

async fn get_device_code(client: &reqwest::Client) -> Result<String, reqwest::Error> {
    let form_params = [("client_id", OAUTH_CLIENT_ID), ("scope", OAUTH_SCOPE)];
    let request = client
        .post("https://github.com/login/device/code")
        .form(&form_params)
        .send()
        .await?;
    request.error_for_status()?;

    return Ok("".to_string());
}

fn get_repo_uri(params: &Params) -> Result<String, ParamNotFoundError> {
    return match params.get("path".to_string()) {
        Some(path) => Ok("https://github.com/".to_owned()),
        None => Err(ParamNotFoundError {
            param: "path".to_string(),
        }),
    };
}

fn log(s: &str) {
    eprintln!("{0}: {1}", crate_name!(), s);
}

#[derive(Debug, Clone)]
struct ParamNotFoundError {
    pub param: String,
}
