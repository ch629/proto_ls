use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

mod types;

type DocumentUri = String;

#[derive(Debug)]
pub struct LsMessage<T> {
    // Headers
    // TODO: We might not know the length until we've serialized it
    pub content_length: usize,
    pub content_type: String,
    pub headers: Vec<String>,

    pub content: LsMessageContent<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LsBaseMessage {
    pub json_rpc: String,
    pub id: usize,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LsMessageContent<T> {
    // Version: always 2.0
    pub json_rpc: String,
    pub id: usize,
    // TODO: Method enum? -> This coincides with the params type, so we can maybe use that?
    pub method: String,
    pub params: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: DocumentUri,
    pub language_id: String,
    pub version: isize,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentUri,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentFilter {
    pub language: Option<String>,
    pub scheme: Option<String>,
    pub pattern: Option<String>,
}

enum LsMethod {
    // method: initialize
    Initialize(types::InitializeParams),
}

#[derive(Debug, Serialize)]
pub struct LsResponse {
    pub id: String,
    // TODO: result?: string | number | boolean | object | null;
    pub result: Option<String>,
    pub error: Option<LsResponseError>,
}

#[derive(Debug, Serialize)]
pub struct LsResponseError {
    pub code: isize,
    pub message: String,
    // TODO: data?: string | number | boolean | array | object | null;
    pub data: Option<String>,
}

// TODO: Generate all of this with a proc macro DSL?
fn get_method(msg: LsBaseMessage) -> Result<LsMethod> {
    match msg.method.as_str() {
        "initialize" => Ok(LsMethod::Initialize(serde_json::from_value(msg.params)?)),
        unknown => bail!("unknown method type: {unknown}"),
    }
}

fn handle_call(msg: LsMethod) -> Result<()> {
    match msg {
        LsMethod::Initialize(req) => {
            println!("received initialize: {:#?}", req);
            Ok(())
        }
    }
}

pub fn on_message(msg: LsBaseMessage) -> Result<()> {
    get_method(msg).map(handle_call)?
}
