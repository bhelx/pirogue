use std::{str::Chars, iter::Peekable};

use crate::bytecode::{ByteCodeBuilder, ByteCode};

#[derive(Debug, Clone)]
enum TokenKind {
    Int,
    Symbol,
    QuoteOpen,
    QuoteClose,
}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    text: String,
}

pub struct Compiler<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(code: &'a str) -> Compiler<'a> {
        Compiler {
            chars: code.chars().peekable(),
        }
    }

    pub fn compile(&mut self) -> ByteCode {
        let mut builder = ByteCodeBuilder::new();
        loop {
            let token = self.consume_token();
            if token.is_none() { break; }
            let token = token.unwrap();
            //println!("{:?}", token);
            match token.kind {
                TokenKind::Int => {
                    let val = token.text.parse::<u8>().unwrap();
                    builder.push_val(val);
                }
                TokenKind::Symbol => {
                    builder.push_symbol(token.text.as_str());
                }
                TokenKind::QuoteOpen => {
                    builder.push_quote();
                }
                TokenKind::QuoteClose => {
                    builder.pop_quote();
                }
            }
        };
        builder.bytecode().clone()
    }

    fn consume_token(&mut self) -> Option<Token> {
        let mut text = String::new();
        loop {
            let chr = self.chars.next();
            if chr.is_none() { return None }
            let chr = chr.unwrap();
            match chr {
                '\n' | '\r' => {
                    break;
                }
                '[' | ']' => {
                    if text.len() == 0 {
                        text.push(chr);
                    }
                    break;
                }
                ' ' => {
                    if text.len() == 0 {
                        continue
                    } else {
                        break;
                    }
                }
                _ => {
                    text.push(chr)
                }
            }
            let next = self.chars.peek();
            if next.is_none() { break }
            match next.unwrap() {
                '[' | ']' => {
                    break;
                }
                ' ' => {
                    if text.len() == 0 {
                        continue
                    } else {
                        break;
                    }
                }
                _ => { continue }
            }
        }

        if text.len() == 0 { return None }

        let kind = match text.as_str() {
            "[" => TokenKind::QuoteOpen,
            "]" => TokenKind::QuoteClose,
            _ => {
                if text.parse::<i32>().is_ok() {
                    TokenKind::Int
                } else {
                    TokenKind::Symbol
                }
            }
        };
        Some(Token { text, kind })

    }
}
