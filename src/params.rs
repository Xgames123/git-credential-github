use std::collections::HashMap;
use std::io::{self, BufRead};

#[derive(Debug, Clone)]
pub struct ParamParserError {
    data: String,
}

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
                self.add((&s[..i]).to_string(), (&s[(i + 1)..]).to_string());
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
}
