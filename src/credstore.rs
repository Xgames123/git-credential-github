/*
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

pub struct CredStore {
    entry: keyring::Entry,
}

impl CredStore {
    pub fn new() -> Result<CredStore, CredsError> {
        Ok(CredStore {
            entry: keyring::Entry::new("gh-login", "github.com")?,
        })
    }

    pub fn store(self, username: &str, password: &str) -> Result<(), CredsError> {
        let mut creds = String::new();

        creds.push_str(username);
        creds.push('/');
        creds.push_str(password);

        self.entry.set_password(&creds)?;

        Ok(())
    }

    pub fn delete(self) -> Result<(), CredsError> {
        self.entry.delete_password()?;
        Ok(())
    }

    pub fn get<'a>(self) -> Result<(&'a str, &'a str), CredsError> {
        let creds: &'a String = self.entry.get_password()?;
        return match parse_creds(creds) {
            None => Err(InvalidData),
            Some((username, token)) => Ok((username, token)),
        };
    }
}

fn parse_creds(data: &str) -> Option<(&str, &str)> {
    for (i, &char) in data.as_bytes().iter().enumerate() {
        if char == b'/' {
            return Some((&data[..i], &data[(i + 1)..]));
        }
    }
    None
}

pub fn get() -> Result<(String, String), CredsError> {
    let credstore = CredStore::new()?;
    let (username, passwd) = credstore.get()?;

    Ok((username.to_string(), passwd.to_string()))
}
*/
