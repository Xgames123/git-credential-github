use crate::credstore::CredsError::InvalidData;
use std::{error::Error, fmt::Display, fmt::Formatter};

#[derive(Debug)]
enum CredsError {
    FailedToAccessStore(Box<dyn std::error::Error + Send + Sync>),
    InvalidData,
    TooManyCredentials,
    NoData,
}
impl Display for CredsError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CredsError::FailedToAccessStore(err) => {
                fmt.write_fmt(format_args!("Failed to access credentials store '{err}'"))
            }
            InvalidData => fmt.write_str("Invalid data"),
            CredsError::NoData => fmt.write_str("No data in credentials store"),
            CredsError::TooManyCredentials => fmt.write_str("2 credentials stored"),
        }
    }
}
impl Error for CredsError {}

impl From<keyring::error::Error> for CredsError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::PlatformFailure(err) => CredsError::FailedToAccessStore(err),
            keyring::Error::NoStorageAccess(err) => CredsError::FailedToAccessStore(err),
            keyring::Error::NoEntry => CredsError::NoData,
            keyring::Error::BadEncoding(_) => InvalidData,
            keyring::Error::TooLong(_, _) => InvalidData,
            keyring::Error::Invalid(_, _) => InvalidData,
            keyring::Error::Ambiguous(_) => CredsError::TooManyCredentials,
            _ => todo!(),
        }
    }
}

pub fn store(username: &str, password: &str) {
    let entry = keyring::Entry::new("gh-login", "github.com")
        .expect("Failed get entry in credentials store");

    let mut creds = String::new();

    creds.push_str(username);
    creds.push('/');
    creds.push_str(password);

    entry
        .set_password(&creds)
        .expect("Failed to store access key");
}

pub fn get<'a>() -> Result<(&'a str, &'a str), CredsError> {
    let entry = keyring::Entry::new("gh-login", "github.com")?;
    let creds : &'a String = entry.get_password()?;
    return match parse_creds(creds) {
        None => Err(InvalidData),
        Some((username, token)) => Ok((username, token)),
    };
}

pub fn parse_creds(data: &str) -> Option<(&str, &str)> {
    for (i, &char) in data.as_bytes().iter().enumerate() {
        if char == b'/' {
            return Some((&data[..i], &data[(i + 1)..]));
        }
    }
    None
}
