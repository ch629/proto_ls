#![allow(dead_code)]
mod ls;
mod parser;

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write},
};

use anyhow::{bail, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // parse_and_log_file();
    handle_io().await?;
    Ok(())
}

fn parse_and_log_file() {
    let f = File::open("/Users/charliehowe/Projects/rust/proto_ls/test.proto").unwrap();
    let mut sc = parser::Scanner::new(f);
    let f = parser::scan_file(&mut sc).unwrap();
    println!("{f:#?}");
}

async fn handle_io() -> Result<()> {
    // TODO: Organizing writes back for outputs
    let mut writer = BufWriter::new(stdout());
    let _ = writer.write("testing".as_bytes());
    let reader = RefCell::new(BufReader::new(stdin()));
    let mut parsing_headers = true;
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut buf = String::new();
        let mut temp_rd = reader.borrow_mut();
        temp_rd.read_line(&mut buf)?;
        // TODO: Trim is needed to remove the \n\r, can we just remove the end?
        let buf = buf.trim();
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
                let content = read_n(temp_rd.get_mut(), content_length)?;
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
    let msg: ls::LsBaseMessage = serde_json::from_str(String::from_utf8(s)?.as_str())?;
    // TODO: Map from message type to params
    println!("m: {msg:#?}");
    ls::on_message(msg)
}
