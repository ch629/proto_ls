pub mod scanner;
pub mod tokens;

use std::{io::Read, vec};

use anyhow::{bail, Result};

use scanner::Scanner;
use tokens::ProtoToken;

#[derive(Debug)]
pub struct ProtoFile {
    syntax: ProtoSyntax,
    package: Vec<String>,
    imports: Vec<ProtoImport>,
    options: Vec<ProtoOption>,
    messages: Vec<ProtoMessage>,
    services: Vec<ProtoService>,
}

#[derive(Debug, strum::Display, PartialEq)]
enum ProtoSyntax {
    Proto2,
    Proto3,
}

#[derive(Debug, strum::Display, PartialEq)]
enum ProtoImportType {
    Default,
    Weak,
    Public,
}

#[derive(Debug)]
struct ProtoImport {
    r#type: ProtoImportType,
    path: String,
}

#[derive(Debug)]
struct ProtoOption {
    name: String,
    value: String,
}

#[derive(Debug)]
struct ProtoMessage {
    name: String,
    fields: Vec<MessageField>,
    messages: Vec<ProtoMessage>,
}

#[derive(Debug)]
struct ProtoService {
    name: String,
    rpcs: Vec<ProtoRpc>,
}

#[derive(Debug)]
struct ProtoRpc {
    name: String,
    params: Vec<String>,
    returns: String,
}

// TODO: Adding options to a message field
// string event_id = 1 [(validate.rules).string.uuid = true];
// TODO: Handle a map type - map<string, string>
#[derive(Debug)]
struct MessageField {
    r#type: ProtoFieldType,
    name: String,
    index: u16,
}

#[derive(Debug)]
pub struct PositionedProtoToken {
    token: ProtoToken,
    column: usize,
    line: usize,
}

#[derive(Debug, strum::Display, PartialEq)]
pub enum ProtoFieldType {
    FullIdentifier(Vec<String>),
    Identifier(String),
    Bool,
    String,
    Bytes,
    Float,
    Double,
    Map {
        key: Box<ProtoFieldType>,
        value: Box<ProtoFieldType>,
    },
}

impl ProtoFieldType {
    fn from_token<T: Read>(t: ProtoToken, scan: &mut Scanner<T>) -> Result<Self> {
        Ok(match t {
            ProtoToken::FullIdentifier(id) => Self::FullIdentifier(
                id.iter()
                    .map(|str| String::from_utf8(str.to_vec()))
                    // TODO: Remove flatten & check for errors in utf-8 parsing
                    .flatten()
                    .collect(),
            ),
            ProtoToken::Identifier(id) => Self::Identifier(String::from_utf8(id)?),
            ProtoToken::Bool => Self::Bool,
            ProtoToken::String => Self::String,
            ProtoToken::Bytes => Self::Bytes,
            ProtoToken::Float => Self::Float,
            ProtoToken::Double => Self::Double,
            ProtoToken::Map => {
                scan.expect(ProtoToken::LessThan)?;
                let Some(key_token) = scan.next() else {
                    bail!("")
                };
                let key = Self::from_token(key_token, scan)?;
                scan.expect(ProtoToken::Comma)?;
                let Some(value_token) = scan.next() else {
                    bail!("")
                };
                let value = Self::from_token(value_token, scan)?;
                scan.expect(ProtoToken::GreaterThan)?;
                Self::Map {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
            other => bail!("non proto field type {other}"),
        })
    }
}

// proto = syntax { import | package | option | topLevelDef | emptyStatement }
// topLevelDef = message | enum | service
pub fn scan_file<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoFile> {
    // TODO: Some of these errors can probably allow it to continue parsing (duplicate package) &
    // just batch the errors up
    let syntax = scan_syntax(scan)?;

    let mut imports = vec![];
    let mut options = vec![];
    let mut package = vec![];
    let mut messages = vec![];
    let mut services = vec![];

    while let Some(token) = scan.next() {
        match token {
            ProtoToken::Comment(_) => {}
            ProtoToken::SemiColon => {}
            ProtoToken::Syntax => todo!(),
            // TODO: Check we haven't already had a package
            ProtoToken::Package => package = scan_package(scan)?,
            ProtoToken::Option => options.push(scan_option(scan)?),
            ProtoToken::Import => imports.push(scan_import(scan)?),
            ProtoToken::Message => messages.push(scan_message(scan)?),
            ProtoToken::Service => services.push(scan_service(scan)?),
            ProtoToken::Enum => todo!(),
            other => bail!("unexpected token {other}"),
        }
    }

    Ok(ProtoFile {
        syntax,
        package: package
            .iter()
            .map(|str| String::from_utf8(str.to_vec()))
            // TODO: Remove flatten & check for errors in utf-8 parsing
            .flatten()
            .collect(),
        imports,
        options,
        messages,
        services,
    })
}

// TODO: Write some helper funcs to make this all cleaner, better errors, store line num + char num
fn scan_message<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoMessage> {
    let mut messages = vec![];
    let Some(ProtoToken::Identifier(name)) = scan.next() else {
        bail!("expected identifier")
    };
    scan.expect(ProtoToken::OpenBracket)?;
    let mut fields = vec![];
    while let Some(token) = scan.next() {
        match token {
            ProtoToken::CloseBracket => {
                break;
            }
            ProtoToken::Message => messages.push(scan_message(scan)?),
            token => fields.push(scan_message_field(scan, token)?),
        };
    }

    Ok(ProtoMessage {
        name: String::from_utf8(name)?,
        fields,
        messages,
    })
}

fn scan_message_field<T: Read>(
    scan: &mut Scanner<T>,
    first_token: ProtoToken,
) -> Result<MessageField> {
    let r#type = ProtoFieldType::from_token(first_token, scan)?;

    let Some(ProtoToken::Identifier(name)) = scan.next() else {
        bail!("expected identifier/type")
    };
    scan.expect(ProtoToken::Equals)?;
    let Some(ProtoToken::IntLiteral(index)) = scan.next() else {
        bail!("expected int literal")
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(MessageField {
        r#type,
        name: String::from_utf8(name)?,
        index: index.try_into()?,
    })
}

fn scan_service<T: Read>(_scan: &mut Scanner<T>) -> Result<ProtoService> {
    todo!()
}

fn scan_syntax<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoSyntax> {
    scan.expect(ProtoToken::Syntax)?;
    scan.expect(ProtoToken::Equals)?;
    let Some(ProtoToken::StringLiteral(syntax)) = scan.next() else {
        bail!("expected string literal")
    };
    let s = match syntax.as_str() {
        "proto3" => ProtoSyntax::Proto3,
        "proto2" => ProtoSyntax::Proto2,
        _ => bail!("expected a syntax of either 'proto3' or 'proto2'"),
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(s)
}

fn scan_import<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoImport> {
    let Some(mut next) = scan.next() else {
        bail!("expected either 'public', 'weak' or a string literal after 'import'")
    };
    let r#type = match next {
        ProtoToken::Public => ProtoImportType::Public,
        ProtoToken::Weak => ProtoImportType::Weak,
        _ => ProtoImportType::Default,
    };
    if r#type != ProtoImportType::Default {
        let Some(token) = scan.next() else {
            bail!("expected a string literal to import")
        };
        next = token;
    }
    let ProtoToken::StringLiteral(import) = next else{
        bail!("expected string literal")
    };
    scan.expect(ProtoToken::SemiColon)?;
    Ok(ProtoImport {
        r#type,
        path: import,
    })
}

fn scan_package<T: Read>(scan: &mut Scanner<T>) -> Result<Vec<Vec<u8>>> {
    let Some(ProtoToken::FullIdentifier(pkg)) = scan.next() else {
        bail!("expected identifier")
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(pkg)
}

fn scan_option<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoOption> {
    let Some(ProtoToken::Identifier(id)) = scan.next() else {
        bail!("expected identifier")
    };
    scan.expect(ProtoToken::Equals)?;
    let Some(ProtoToken::StringLiteral(opt)) = scan.next() else {
        bail!("expected string literal")
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(ProtoOption {
        name: String::from_utf8(id)?,
        value: opt,
    })
}

fn read_n<R: Read>(reader: &mut R, bytes_to_read: u64) -> Result<Vec<u8>> {
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    let bytes = chunk.read_to_end(&mut buf)?;
    if bytes as u64 != bytes_to_read {
        bail!("expected {bytes_to_read} bytes to read but only read {bytes}")
    }
    Ok(buf)
}
