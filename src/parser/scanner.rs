use std::{
    cell::RefCell,
    io::{self, BufReader, Read},
    vec,
};

use anyhow::{bail, Result};

use crate::parser::read_n;

use crate::parser::tokens::ProtoToken;

pub struct Scanner<T: Read> {
    reader: BufReader<T>,
    done: bool,
    buffer: RefCell<Vec<u8>>,
}

impl<T: Read> Scanner<T> {
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
        let mut buffer = self.take_until_consume(|c| c != open_char && c != &b'\\', false)?;
        while let Some(b) = self.peek() {
            if b != b'\\' {
                break;
            }
            self.pop();

            let Some(b) = self.peek() else {
                bail!("expected a character after an escape character")
            };
            self.pop();
            buffer.push(b);
            buffer.append(&mut self.take_until_consume(|c| c != open_char && c != &b'\\', false)?);
        }
        self.pop();
        Ok(buffer)
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

    // take_until_consume takes each character until a predicate no longer matches & consumes them,
    // including the character which fails the predicate
    fn take_until_consume_including(
        &mut self,
        predicate: impl Fn(&u8) -> bool,
        include_eof: bool,
    ) -> Result<Vec<u8>> {
        match self.take_until_consume(predicate, include_eof) {
            Ok(s) => {
                self.pop();
                Ok(s)
            }
            err => err,
        }
    }

    // take_until_consume takes each character until a predicate no longer matches & consumes them
    fn take_until_consume(
        &mut self,
        predicate: impl Fn(&u8) -> bool,
        include_eof: bool,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![];
        while let Some(c) = self.peek() {
            if predicate(&c) {
                self.pop();
                buf.push(c);
            } else {
                return Ok(buf);
            }
        }

        if include_eof && self.peek() == None {
            return Ok(buf);
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

    pub fn expect(&mut self, tkn: ProtoToken) -> Result<ProtoToken> {
        let Some(got_token) = self.next() else {
            bail!("wanted {tkn} but received EOF")
        };
        // Skip comments by recursively calling expect if it is one
        if let ProtoToken::Comment(_) = got_token {
            return self.expect(tkn);
        }
        if tkn != got_token {
            bail!("wanted {tkn} but got {got_token}")
        }
        Ok(tkn)
    }
}

impl<T: io::Read> Iterator for Scanner<T> {
    type Item = ProtoToken;

    // TODO: Could we wrap this in a comment ignoring iter which just calls next again if it's a
    // comment?
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
                    "map" => ProtoToken::Map,
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
                b'<' => Some(ProtoToken::LessThan),
                b'>' => Some(ProtoToken::GreaterThan),
                b',' => Some(ProtoToken::Comma),
                b'/' => {
                    self.pop();
                    if let Some(c) = self.peek() {
                        let token = match c {
                            b'/' => {
                                self.pop();
                                Some(ProtoToken::Comment(
                                    self.take_until_consume_including(
                                        |c| c != &b'\n' && c != &b'\x00',
                                        true,
                                    )
                                    .unwrap(),
                                ))
                            }
                            b'*' => {
                                self.pop();
                                let mut buf = vec![];
                                loop {
                                    buf.append(
                                        &mut self
                                            .take_until_consume(|c| c != &b'*', false)
                                            .unwrap(),
                                    );
                                    // pop '*'
                                    self.pop();

                                    let Some(ch) = self.peek() else {
                                        panic!("EOF before end of multiline comment")
                                    };

                                    if ch == b'/' {
                                        self.pop();
                                        break;
                                    }
                                    buf.push(b'*');
                                }

                                Some(ProtoToken::Comment(buf))
                            }
                            _ => None,
                        };
                        return token;
                    }

                    None
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! scan_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                let mut scan: Scanner<&[u8]> = Scanner::new(input.as_bytes().into());
                assert_eq!(Some(expected), scan.next());
            }
        )*
        }
    }

    scan_tests!(
        syntax: ("syntax", ProtoToken::Syntax),
        full_ident: ("foo.bar.baz", ProtoToken::FullIdentifier(vec!["foo".into(), "bar".into(), "baz".into()])),
        int_literal: ("42", ProtoToken::IntLiteral(42)),
        string_literal: (r#""string""#, ProtoToken::StringLiteral("string".to_owned())),
        string_literal_escaped: (r#""str\"ing""#, ProtoToken::StringLiteral(r#"str"ing"#.to_owned())),
        single_line_comment: ("//comment\n", ProtoToken::Comment("comment".into())),
        single_line_comment_eof: ("//comment", ProtoToken::Comment("comment".into())),
        multi_line_comment: ("/*comment*/", ProtoToken::Comment("comment".into())),
        multi_line_comment_extra_asterisk: ("/*comm*ent*/", ProtoToken::Comment("comm*ent".into())),
        multi_line_comment_newlines: ("/*comm\nent*/", ProtoToken::Comment("comm\nent".into())),
    );
}
