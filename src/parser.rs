use std::{
    io::{self, BufReader, Read},
    vec,
};

use anyhow::{bail, Context, Result};

#[derive(Debug)]
pub struct ProtoFile {
    syntax: ProtoSyntax,
    package: Vec<String>,
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
    name: String,
    value: String,
}

// TODO: Embedding messages in messages
#[derive(Debug)]
struct ProtoMessage {
    name: String,
    fields: Vec<MessageField>,
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
    r#type: Vec<String>,
    name: String,
    index: u16,
}

#[derive(Debug, strum::Display)]
pub enum ProtoToken {
    FullIdentifier(Vec<String>),
    Identifier(String),
    StringLiteral(String),
    IntLiteral(isize),
    Colon,
    SemiColon,
    Syntax,
    Package,
    Option,
    Import,
    Message,
    Service,
    Enum,
    OneOf,
    Repeated,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Equals,
    Weak,
    Public,
}

// proto = syntax { import | package | option | topLevelDef | emptyStatement }
// topLevelDef = message | enum | service
pub fn scan_file<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoFile> {
    scan.consume_comments()?;
    let syntax = scan_syntax(scan)?;

    let mut imports = vec![];
    let mut options = vec![];
    let mut package = vec![];
    let mut messages = vec![];
    let mut services = vec![];

    while let Some(token) = scan.next() {
        scan.consume_comments()?;
        match token {
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
    if let ProtoToken::Identifier(name) = scan.next().context("expected message name")? {
        if let ProtoToken::OpenBracket = scan.next().context("expected open bracket")? {
            let mut fields = vec![];
            while let Some(token) = scan.next() {
                match token {
                    ProtoToken::CloseBracket => {
                        break;
                    }
                    ProtoToken::Message => todo!(),
                    token => fields.push(scan_message_field(scan, token)?),
                };
            }
            return Ok(ProtoMessage { name, fields });
        }
    }
    bail!("invalid message");
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

    if let Some(ProtoToken::Identifier(name)) = scan.next() {
        if let Some(ProtoToken::Equals) = scan.next() {
            if let Some(ProtoToken::IntLiteral(index)) = scan.next() {
                if let Some(ProtoToken::SemiColon) = scan.next() {
                    return Ok(MessageField {
                        r#type,
                        name,
                        index: index.try_into()?,
                    });
                }
            }
        }
    }
    bail!("invalid message field")
}

fn scan_service<T: Read>(_scan: &mut Scanner<T>) -> Result<ProtoService> {
    todo!()
}

fn scan_syntax<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoSyntax> {
    if let Some(ProtoToken::Syntax) = scan.next() {
        if let Some(ProtoToken::Equals) = scan.next() {
            if let Some(ProtoToken::StringLiteral(syntax)) = scan.next() {
                let s = match syntax.as_str() {
                    "proto3" => ProtoSyntax::PROTO3,
                    "proto2" => ProtoSyntax::PROTO2,
                    _ => bail!("expected a syntax of either 'proto3' or 'proto2'"),
                };

                if let Some(ProtoToken::SemiColon) = scan.next() {
                    return Ok(s);
                }
            }
        }
    }
    bail!("invalid syntax");
}

fn scan_import<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoImport> {
    // TODO: Optional public or weak
    if let Some(ProtoToken::StringLiteral(import)) = scan.next() {
        if let Some(ProtoToken::SemiColon) = scan.next() {
            return Ok(ProtoImport {
                r#type: ProtoImportType::DEFAULT,
                path: import,
            });
        }
    }
    bail!("invalid import")
}

fn scan_package<T: Read>(scan: &mut Scanner<T>) -> Result<Vec<String>> {
    if let Some(ProtoToken::FullIdentifier(pkg)) = scan.next() {
        if let Some(ProtoToken::SemiColon) = scan.next() {
            return Ok(pkg);
        }
    }
    bail!("invalid package")
}

fn scan_option<T: Read>(scan: &mut Scanner<T>) -> Result<ProtoOption> {
    if let Some(ProtoToken::Identifier(id)) = scan.next() {
        if let Some(ProtoToken::Equals) = scan.next() {
            if let Some(ProtoToken::StringLiteral(opt)) = scan.next() {
                if let Some(ProtoToken::SemiColon) = scan.next() {
                    return Ok(ProtoOption {
                        name: id,
                        value: opt,
                    });
                }
            }
        }
    }
    bail!("invalid option")
}

#[derive(Debug)]
pub struct Scanner<T: Read> {
    reader: BufReader<T>,
    done: bool,
    look_ahead: Option<char>,
}

// TODO: We're using u8 from the reader which isn't necessarily a full char
// -> could use utf8_char_width(c) - this should only be needed for string literals
impl<'a, T: Read> Scanner<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader: BufReader::new(reader),
            done: false,
            look_ahead: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        match self.look_ahead {
            Some(b) => Some(b as char),
            None => {
                let mut b: [u8; 1] = [0u8; 1];
                match self.reader.read(&mut b) {
                    Ok(i) => {
                        // EOF
                        if i == 0 {
                            self.done = true;
                            None
                        } else {
                            self.look_ahead = Some(b[0] as char);
                            Some(b[0] as char)
                        }
                    }
                    Err(_) => None,
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn pop(&mut self) -> Option<char> {
        match self.look_ahead {
            Some(b) => {
                self.look_ahead = None;
                Some(b)
            }
            None => {
                let mut b: [u8; 1] = [0u8; 1];
                match self.reader.read(&mut b) {
                    Ok(i) => {
                        if i == 0 {
                            self.done = true;
                            None
                        } else {
                            Some(b[0] as char)
                        }
                    }
                    Err(_) => {
                        self.done = true;
                        None
                    }
                }
            }
        }
    }

    fn scan(&mut self, predicate: impl Fn(&char) -> bool) -> Option<String> {
        let mut seq = String::new();
        loop {
            match self.peek() {
                Some(c) => {
                    if predicate(&c) {
                        self.pop();
                        seq.push(c);
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        if seq.is_empty() {
            None
        } else {
            Some(seq)
        }
    }

    fn ident(&mut self) -> Option<String> {
        if let Some(c) = self.peek() {
            match c {
                'a'..='z' => {}
                'A'..='Z' => {}
                _ => return None,
            }
        }
        self.scan(|c| match c {
            'a'..='z' => true,
            'A'..='Z' => true,
            '0'..='9' => true,
            '_' => true,
            _ => false,
        })
    }

    fn dot(&mut self) -> Option<()> {
        match self.peek() {
            Some(c) => {
                if c == '.' {
                    Some(())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn full_ident(&mut self) -> Result<Option<Vec<String>>> {
        let mut seq = vec![];
        let mut required = false;
        loop {
            match self.ident() {
                Some(id) => {
                    seq.push(id);
                    match self.dot() {
                        Some(_) => {
                            self.pop();
                            required = true;
                        }
                        None => return Ok(Some(seq)),
                    }
                }
                None => {
                    if required {
                        bail!("ident is required after a dot")
                    }
                    return if seq.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(seq))
                    };
                }
            }
        }
    }

    fn int_literal(&mut self) -> Option<String> {
        if let Some(c) = self.peek() {
            match c {
                '1'..='9' => {}
                _ => return None,
            }
        }
        self.scan(|c| match c {
            '0'..='9' => true,
            _ => false,
        })
    }

    fn string_literal(&mut self) -> Result<Option<String>> {
        let open_char: char;
        match self.peek() {
            Some('"') => {
                open_char = '"';
            }
            Some('\'') => open_char = '\'',
            _ => return Ok(None),
        }
        self.pop();
        let res = self.scan(|c| c != &open_char);
        // Consume end quote
        self.pop();

        Ok(res)
    }

    // whitespace consumes all of the whitespace characters and returns Some if any were consumed
    // or None if no whitespace chars were consumed
    fn whitespace(&mut self) -> Option<()> {
        let mut found_any = false;
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\r' | '\n' | '\t' => {
                    self.pop();
                    found_any = true;
                }
                _ => break,
            }
        }

        if found_any {
            Some(())
        } else {
            None
        }
    }

    fn consume_comments(&mut self) -> Result<()> {
        loop {
            if let Some(c) = self.peek() {
                if c == '/' {
                    self.pop();
                    if let Some(c) = self.peek() {
                        if c == '/' {
                            // Consume everything until a newline
                            self.scan(|c| c != &'\n');
                            self.whitespace();
                        } else {
                            bail!("expected '//'")
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl<T: io::Read> Iterator for Scanner<T> {
    type Item = ProtoToken;

    // TODO: Comments
    // TODO: How do we handle errors? -> Use a Result<ProtoToken> as Item?
    fn next(&mut self) -> Option<Self::Item> {
        self.whitespace();
        if self.is_done() {
            return None;
        }

        if let Ok(Some(name)) = self.full_ident() {
            if name.len() == 1 {
                let name = &name[0];
                // Keywords or identifier
                return Some(match name.as_str() {
                    "syntax" => ProtoToken::Syntax,
                    "package" => ProtoToken::Package,
                    "option" => ProtoToken::Option,
                    "import" => ProtoToken::Import,
                    "message" => ProtoToken::Message,
                    "service" => ProtoToken::Service,
                    "enum" => ProtoToken::Enum,
                    "oneof" => ProtoToken::OneOf,
                    "repeated" => ProtoToken::Repeated,
                    "weak" => ProtoToken::Weak,
                    "public" => ProtoToken::Public,
                    _ => ProtoToken::Identifier(name.to_string()),
                });
            }

            return Some(ProtoToken::FullIdentifier(name));
        }

        match self.string_literal() {
            Ok(opt) => {
                if let Some(literal) = opt {
                    return Some(ProtoToken::StringLiteral(literal));
                }
            }
            Err(_) => return None,
        }

        if let Some(i) = self.int_literal() {
            // TODO: Remove unwrap
            return Some(ProtoToken::IntLiteral(i.parse().unwrap()));
        }

        if let Some(c) = self.peek() {
            // TODO: Only pop if its expected
            self.pop();
            return match c {
                ';' => Some(ProtoToken::SemiColon),
                '=' => Some(ProtoToken::Equals),
                '{' => Some(ProtoToken::OpenBracket),
                '}' => Some(ProtoToken::CloseBracket),
                '(' => Some(ProtoToken::OpenParen),
                ')' => Some(ProtoToken::CloseParen),
                '[' => Some(ProtoToken::OpenBrace),
                ']' => Some(ProtoToken::CloseBrace),
                ':' => Some(ProtoToken::Colon),
                _ => None,
            };
        }
        None
    }
}
