#![allow(dead_code)]
mod language_server;
mod parser;

use std::{
    collections::HashMap,
    fs::File,
    io::{stdin, BufRead, BufReader, Read},
};

use anyhow::{bail, Result};

#[tokio::main]
async fn main() -> Result<()> {
    parse_and_log_file();
    // handle_io().await?;
    Ok(())
}

fn parse_and_log_file() {
    let f = File::open("/Users/charliehowe/Projects/rust/proto_ls/test.proto").unwrap();
    let mut sc = parser::scanner::Scanner::new(f);
    let f = parser::scan_file(&mut sc).unwrap();
    println!("{f:#?}");
}

// signal::ctrl_c().await?;
async fn handle_io() -> Result<()> {
    // TODO: Organizing writes back for outputs
    let mut reader = BufReader::new(stdin());
    let mut parsing_headers = true;
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        let buf = buf.trim_end();
        println!("buf: {:?}", buf.as_bytes());

        if buf.is_empty() {
            parsing_headers = !parsing_headers;

            // Clear headers from the previous message
            if parsing_headers {
                headers.clear();
            } else {
                let content_length = &headers["Content-Length"];
                println!("length: {:?}", content_length.as_bytes());
                let content_length: usize = content_length.parse()?;
                let content = read_n(&mut reader, content_length)?;
                println!("content: {content:#?}");
                // TODO: can we pass this into a channel for processing?
                parse_content(content)?;
            }
            continue;
        }

        if parsing_headers {
            let (key, value) = parse_header(buf.to_owned())?;
            headers.insert(key.clone(), value.clone());
            println!("header: {} - {}", key.clone(), value.clone());
            continue;
        }

        // TODO: Ctrl+C
        if buf == "quit" {
            break;
        }
    }
    Ok(())
}

fn read_n<R: Read>(rd: &mut R, n: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    let mut t = rd.take(n as u64);
    t.read_exact(&mut buf)?;
    Ok(buf)
}

fn parse_header(s: String) -> Result<(String, String)> {
    let spl: Vec<&str> = s.split(": ").collect();

    if spl.len() != 2 {
        bail!("expected header in format {{Name}}: {{Value}}")
    }
    Ok((spl[0].to_owned(), spl[1].to_owned()))
}

fn parse_content(s: Vec<u8>) -> Result<()> {
    let msg: language_server::LsBaseMessage = serde_json::from_str(String::from_utf8(s)?.as_str())?;
    // TODO: Map from message type to params
    println!("m: {msg:#?}");
    language_server::on_message(msg)
}
