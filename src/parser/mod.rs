pub mod scanner;
pub mod tokens;

use std::{io::Read, vec};

use anyhow::{bail, Result};

use scanner::Scanner;
use tokens::ProtoToken;

#[derive(Debug)]
pub struct ProtoFile {
    syntax: ProtoSyntax,
    package: Vec<Vec<u8>>,
    imports: Vec<ProtoImport>,
    options: Vec<ProtoOption>,
    messages: Vec<ProtoMessage>,
    services: Vec<ProtoService>,
}

#[derive(Debug)]
enum ProtoSyntax {
    PROTO2,
    PROTO3,
}

#[derive(Debug)]
enum ProtoImportType {
    DEFAULT,
    WEAK,
    PUBLIC,
}

#[derive(Debug)]
struct ProtoImport {
    r#type: ProtoImportType,
    path: String,
}

#[derive(Debug)]
struct ProtoOption {
    name: Vec<u8>,
    value: String,
}

#[derive(Debug)]
struct ProtoMessage {
    name: Vec<u8>,
    fields: Vec<MessageField>,
    messages: Vec<ProtoMessage>,
}

#[derive(Debug)]
struct ProtoService {
    name: Vec<u8>,
    rpcs: Vec<ProtoRpc>,
}

#[derive(Debug)]
struct ProtoRpc {
    name: Vec<u8>,
    params: Vec<Vec<u8>>,
    returns: Vec<u8>,
}

// TODO: Adding options to a message field
// string event_id = 1 [(validate.rules).string.uuid = true];
// TODO: Handle a map type - map<string, string>
#[derive(Debug)]
struct MessageField {
    r#type: Vec<Vec<u8>>,
    name: Vec<u8>,
    index: u16,
}

#[derive(Debug)]
pub struct PositionedProtoToken {
    token: ProtoToken,
    column: usize,
    line: usize,
}

// proto = syntax { import | package | option | topLevelDef | emptyStatement }
// topLevelDef = message | enum | service
pub fn scan_file<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoFile> {
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
        package,
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
        name,
        fields,
        messages,
    })
}

fn scan_message_field<T: Read>(
    scan: &mut Scanner<T>,
    first_token: ProtoToken,
) -> Result<MessageField> {
    let r#type = match first_token {
        ProtoToken::FullIdentifier(id) => id,
        ProtoToken::Identifier(id) => vec![id],
        _ => bail!("expected identifier type"),
    };

    let Some(ProtoToken::Identifier(name)) = scan.next() else {
        bail!("expected identifier")
    };
    scan.expect(ProtoToken::Equals)?;
    let Some(ProtoToken::IntLiteral(index)) = scan.next() else {
        bail!("expected int literal")
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(MessageField {
        r#type,
        name,
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
        "proto3" => ProtoSyntax::PROTO3,
        "proto2" => ProtoSyntax::PROTO2,
        _ => bail!("expected a syntax of either 'proto3' or 'proto2'"),
    };
    scan.expect(ProtoToken::SemiColon)?;

    Ok(s)
}

fn scan_import<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoImport> {
    // TODO: Optional public or weak
    let Some(ProtoToken::StringLiteral(import)) = scan.next() else{
        bail!("expected string literal")
    };
    scan.expect(ProtoToken::SemiColon)?;
    Ok(ProtoImport {
        r#type: ProtoImportType::DEFAULT,
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
        name: id,
        value: opt,
    })
}

fn read_n<R: Read>(reader: &mut R, bytes_to_read: u64) -> Result<Vec<u8>> {
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    // TODO: Return error if n != bytes_to_read?
    let _ = chunk.read_to_end(&mut buf)?;
    Ok(buf)
}
