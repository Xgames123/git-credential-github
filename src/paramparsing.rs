use std::{
    collections::HashMap,
    io::{self, stdin, BufRead, BufReader},
};

pub fn write_to_stdout(params: &HashMap<String, String>) {
    for (key, value) in params.iter() {
        if key != "" && value != "" {
            println!("{}={}", key, value);
        }
    }
    println!("");
}

pub fn write_to(params: &HashMap<String, String>, stream: &mut impl io::Write) -> io::Result<()> {
    for (key, value) in params.iter() {
        stream.write_fmt(format_args!("{0}={1}\n", key, value))?
    }
    Ok(stream.write_all(b"\n\n")?)
}
pub fn parse_from(stream: &mut impl io::Read) -> io::Result<HashMap<String, String>> {
    let mut params = HashMap::<String, String>::new();
    let mut reader = BufReader::new(stream);
    loop {
        let mut buf = String::new();

        reader.read_line(&mut buf)?;
        let buf = buf.trim();
        if buf.is_empty() {
            return Ok(params);
        }
        if let Some((key, value)) = buf.split_once('=') {
            params.insert(key.to_string(), value.to_string());
        }
    }
}

pub fn parse_from_stdin() -> io::Result<HashMap<String, String>> {
    let mut stdin = stdin().lock();
    parse_from(&mut stdin)
}
