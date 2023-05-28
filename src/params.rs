use std::collections::HashMap;
use std::fmt::Formatter;
use std::{error::Error, fmt::Display, io, io::BufRead};

#[derive(Debug, Clone)]
pub struct ParamParserError {
    data: String,
}
impl Display for ParamParserError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.write_fmt(format_args!(
            "Failed to parse parameter. Data {}",
            self.data
        ))
    }
}
impl Error for ParamParserError {}

pub struct Params {
    hashmap: HashMap<String, String>,
}
impl Params {
    pub fn get(&self, key: String) -> Option<&String> {
        return self.hashmap.get(&key);
    }

    pub fn add(&mut self, name: String, value: String) {
        self.hashmap.insert(name, value);
    }

    pub fn add_from_string(&mut self, s: &String) -> Result<(), ParamParserError> {
        for (i, &character) in s.as_bytes().iter().enumerate() {
            if character == b'=' {
                self.add(
                    (&s[..i]).trim().to_string(),
                    (&s[(i + 1)..]).trim().to_string(),
                );
                return Ok(());
            }
        }
        return Err(ParamParserError {
            data: s.to_string(),
        });
    }

    pub fn from_stdin() -> Result<Params, ParamParserError> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        let mut params = Params {
            hashmap: HashMap::new(),
        };
        loop {
            let mut buffer = String::new();

            handle.read_line(&mut buffer).unwrap();

            if buffer == "" {
                break;
            }
            params.add_from_string(&buffer)?;
        }
        return Ok(params);
    }

    pub fn write_to_sdtout(&self) {
        for (key, value) in self.hashmap.iter() {
            println!("{0}={1}", key, value);
        }
        println!();
    }
}
