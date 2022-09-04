mod ls;
mod parser;

use std::{collections::HashMap, fs::File};

use anyhow::{bail, Ok, Result};
use tokio::{
    io::{AsyncBufReadExt, BufStream},
    net::{TcpListener, TcpStream},
};

// #[tokio::main]
fn main() {
    // let listener = TcpListener::bind("127.0.0.1:8080").await?;

    // loop {
    //     let (stream, _) = listener.accept().await?;
    //     // TODO: Spawn thread
    //     let mut stream = BufStream::new(stream);
    //     // header: value\r\n
    //     // header: value\r\n
    //     // \r\n
    //     // {content}\r\n
    //     // TODO: This should only exit out the current connection
    //     process_headers(&mut stream).await?;
    //     break;
    // }

    let f = File::open("/Users/charliehowe/Projects/rust/proto_ls/test.proto").unwrap();
    let mut sc = parser::Scanner::new(f);
    let file = parser::scan_file(&mut sc).unwrap();

    println!("{:#?}", file);
    // Ok(())
}

async fn process_headers(stream: &mut BufStream<TcpStream>) -> Result<HashMap<String, String>> {
    let mut m = HashMap::new();
    // TODO: Make sure we consume both \r\n and trim them
    loop {
        let mut buf = String::new();
        stream.read_line(&mut buf).await?;
        if buf.is_empty() {
            break;
        }
        let split: Vec<&str> = buf.split(": ").collect();

        if split.len() != 2 {
            bail!("invalid header format received: '{buf}'")
        }
        m.insert(split[0].to_string(), split[1].to_string());
    }
    Ok(m)
}

async fn process_content(stream: &mut BufStream<TcpStream>) -> Result<()> {
    let mut buf = String::new();
    stream.read_line(&mut buf).await?;
    todo!()
}
