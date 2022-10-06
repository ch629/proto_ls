use serde::{Deserialize, Serialize};

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

enum LsMethod {}

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
