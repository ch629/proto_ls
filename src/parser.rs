use std::{
    cell::RefCell,
    io::{self, BufReader, Read},
    vec,
};

use anyhow::{bail, Context, Result};

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

#[derive(Debug, strum::Display)]
pub enum ProtoToken {
    FullIdentifier(Vec<Vec<u8>>),
    Identifier(Vec<u8>),
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
    Reserved,
    Extend,
    Extensions,
    /// To is used for ranges `reserved 10 to max;`
    To,
    /// Max is interpreted as 2,147,483,647
    Max,
}

#[derive(Debug)]
pub struct PositionedProtoToken {
    token: ProtoToken,
    character: usize,
    line: usize,
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
    let mut messages = vec![];
    if let ProtoToken::Identifier(name) = scan.next().context("expected message name")? {
        if let ProtoToken::OpenBracket = scan.next().context("expected open bracket")? {
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
            return Ok(ProtoMessage {
                name,
                fields,
                messages,
            });
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

fn scan_package<T: Read>(scan: &mut Scanner<T>) -> Result<Vec<Vec<u8>>> {
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

fn read_n<R: Read>(reader: &mut R, bytes_to_read: u64) -> Result<Vec<u8>> {
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    // TODO: Return error if n != bytes_to_read?
    let _ = chunk.read_to_end(&mut buf)?;
    Ok(buf)
}

pub struct Scanner<T: Read> {
    reader: BufReader<T>,
    done: bool,
    buffer: RefCell<Vec<u8>>,
}

impl<'a, T: Read> Scanner<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader: BufReader::new(reader),
            done: false,
            buffer: RefCell::new(Vec::with_capacity(8)),
        }
    }

    fn peek(&mut self) -> Option<u8> {
        if self.is_buffer_empty() {
            let _ = self.load_buffer(1);
        }

        match self.buffer.borrow().first() {
            Some(i) => Some(*i),
            None => None,
        }
    }

    // TODO: I only really need this for 2 bytes for comments
    fn peek_string(&mut self, len: usize) -> Result<String> {
        let b_len = self.buffer.borrow().len();
        if b_len < len {
            // TODO: Check we're long enough
            self.append_buffer(len as u64 - b_len as u64)?;

            if b_len < len {
                bail!("not enough bytes left")
            }
        }
        let chars = &self.buffer.borrow()[0..len];
        Ok(String::from_utf8(chars.to_vec())?)
    }

    fn load_buffer(&mut self, len: u64) -> Result<usize> {
        if !self.is_buffer_empty() {
            bail!("buffer is not clear")
        }

        self.append_buffer(len)
    }

    fn is_buffer_empty(&self) -> bool {
        self.buffer.borrow().is_empty()
    }

    fn append_buffer(&mut self, len: u64) -> Result<usize> {
        let mut bytes = read_n(&mut self.reader, len)?;
        self.buffer.borrow_mut().append(&mut bytes);
        Ok(bytes.len())
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn pop(&mut self) -> Option<()> {
        if self.is_buffer_empty() {
            // TODO: We can avoid the extra buffer logic without using peek
            let b = self.peek();
            self.buffer.borrow_mut().clear();
            b.map(|_| ())
        } else {
            let mut buf = self.buffer.borrow_mut();
            buf.drain(0..1);
            Some(())
        }
    }

    fn scan(&mut self, predicate: impl Fn(&u8) -> bool) -> Option<Vec<u8>> {
        let mut seq = vec![];
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

    fn ident(&mut self) -> Option<Vec<u8>> {
        if let Some(c) = self.peek() {
            match c {
                b'a'..=b'z' => {}
                b'A'..=b'Z' => {}
                _ => return None,
            }
        }
        self.scan(|c| match c {
            b'a'..=b'z' => true,
            b'A'..=b'Z' => true,
            b'0'..=b'9' => true,
            b'_' => true,
            _ => false,
        })
    }

    fn dot(&mut self) -> Option<()> {
        match self.peek() {
            Some(c) => {
                if c == b'.' {
                    Some(())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn full_ident(&mut self) -> Result<Option<Vec<Vec<u8>>>> {
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

    fn int_literal(&mut self) -> Option<Vec<u8>> {
        if let Some(c) = self.peek() {
            match c {
                b'1'..=b'9' => {}
                _ => return None,
            }
        }
        self.scan(|c| match c {
            b'0'..=b'9' => true,
            _ => false,
        })
    }

    fn string(&mut self) -> Option<Result<Vec<u8>>> {
        let open_char;
        match self.peek() {
            Some(b'"') => open_char = b'"',
            Some(b'\'') => open_char = b'\'',
            _ => return None,
        }
        self.pop();

        Some(self.string_literal(&open_char))
    }

    fn string_literal(&mut self, open_char: &u8) -> Result<Vec<u8>> {
        Ok(self.take_until_consume_including(|c| c != open_char)?)
    }

    // whitespace consumes all of the whitespace characters
    fn whitespace(&mut self) {
        let _ = self.take_until(
            |c| match c {
                b' ' | b'\r' | b'\n' | b'\t' => true,
                _ => false,
            },
            true,
        );
    }

    fn comment(&mut self) -> Option<Result<()>> {
        if let Ok(s) = self.peek_string(2) {
            // TODO: Need to pop 2 -> Or can we rely on take_until to do this?
            return match s.as_str() {
                "//" => Some(self.line_comment()),
                "/*" => Some(self.block_comment()),
                _ => None,
            };
        }
        None
    }

    fn line_comment(&mut self) -> Result<()> {
        self.take_until(|c| c != &b'\n' && c != &b'\x00', true)
    }

    fn block_comment(&mut self) -> Result<()> {
        self.take_until_str(2, |s| s == "*/")
    }

    fn consume_comments(&mut self) -> Result<()> {
        while let Some(r) = self.comment() {
            r?
        }

        Ok(())
    }

    // take_until_consume takes each character until a predicate no longer matches & consumes them,
    // including the character which fails the predicate
    fn take_until_consume_including(&mut self, predicate: impl Fn(&u8) -> bool) -> Result<Vec<u8>> {
        match self.take_until_consume(predicate) {
            Ok(s) => {
                self.pop();
                Ok(s)
            }
            err => err,
        }
    }

    // take_until_consume takes each character until a predicate no longer matches & consumes them
    fn take_until_consume(&mut self, predicate: impl Fn(&u8) -> bool) -> Result<Vec<u8>> {
        let mut buf = vec![];
        while let Some(c) = self.peek() {
            if predicate(&c) {
                self.pop();
                buf.push(c);
            } else {
                return Ok(buf);
            }
        }
        bail!("didn't match predicate")
    }

    fn take_until_str(&mut self, str_len: usize, predicate: impl Fn(&str) -> bool) -> Result<()> {
        while let Ok(s) = self.peek_string(str_len) {
            if predicate(&s) {
                for _ in 0..str_len {
                    self.pop();
                }
            } else {
                return Ok(());
            }
        }

        bail!("didn't match predicate")
    }

    // take_until consumes all characters while the predicate is passes but throws away the result
    // returns an Ok if include_eof is true & it hits the EOF
    fn take_until(&mut self, predicate: impl Fn(&u8) -> bool, include_eof: bool) -> Result<()> {
        while let Some(c) = self.peek() {
            if predicate(&c) {
                self.pop();
            } else {
                return Ok(());
            }
        }

        if include_eof && self.peek() == None {
            return Ok(());
        }
        bail!("didn't match predicate")
    }

    fn pop_buffer(&mut self) -> Result<String> {
        let mut buf = self.buffer.borrow_mut();
        let str = String::from_utf8(buf.to_vec())?;
        buf.clear();
        Ok(str)
    }
}

impl<T: io::Read> Iterator for Scanner<T> {
    type Item = ProtoToken;

    // TODO: Comments
    // TODO: Multiline comments /* * * * */
    // TODO: How do we handle errors? -> Use a Result<ProtoToken> as Item?
    fn next(&mut self) -> Option<Self::Item> {
        self.whitespace();
        if self.is_done() {
            return None;
        }

        if let Ok(Some(name)) = self.full_ident() {
            if name.len() == 1 {
                let name = &name[0];
                let name_vec = name.to_vec();
                let name_string = String::from_utf8(name_vec.clone()).unwrap();
                // Keywords or identifier
                return Some(match name_string.as_str() {
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
                    "reserved" => ProtoToken::Reserved,
                    "extend" => ProtoToken::Extend,
                    "extensions" => ProtoToken::Extensions,
                    "to" => ProtoToken::To,
                    "max" => ProtoToken::Max,
                    _ => ProtoToken::Identifier(name_vec),
                });
            }

            return Some(ProtoToken::FullIdentifier(name));
        }

        match self.string() {
            Some(opt) => {
                return match opt {
                    Ok(s) => Some(ProtoToken::StringLiteral(String::from_utf8(s).unwrap())),
                    Err(_) => None,
                };
            }
            None => {}
        }

        if let Some(i) = self.int_literal() {
            // TODO: Remove unwrap
            return Some(ProtoToken::IntLiteral(
                String::from_utf8(i).unwrap().parse().unwrap(),
            ));
        }

        if let Some(c) = self.peek() {
            let token = match c {
                b';' => Some(ProtoToken::SemiColon),
                b'=' => Some(ProtoToken::Equals),
                b'{' => Some(ProtoToken::OpenBracket),
                b'}' => Some(ProtoToken::CloseBracket),
                b'(' => Some(ProtoToken::OpenParen),
                b')' => Some(ProtoToken::CloseParen),
                b'[' => Some(ProtoToken::OpenBrace),
                b']' => Some(ProtoToken::CloseBrace),
                b':' => Some(ProtoToken::Colon),
                _ => None,
            };

            if token.is_some() {
                self.pop();
            }
            return token;
        }
        None
    }
}
