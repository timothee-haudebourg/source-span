extern crate source_span;
extern crate utf8_decode;

use source_span::Span;
use std::fs::File;
use std::io::Read;
use utf8_decode::UnsafeDecoder;

#[derive(Clone, Default)]
pub struct Token {
    string: String,
    span: Span,
}

#[derive(PartialEq)]
pub enum Kind {
    Space,
    Separator,
    Alphabetic,
    Numeric,
}

fn main() -> std::io::Result<()> {
    let file = File::open("examples/file.txt")?;
    let chars = UnsafeDecoder::new(file.bytes());

    let mut tokens = Vec::new();
    let mut current = Token::default();
    let mut kind = Kind::Space;

    for c in chars {
        let c = c?; // report eventual errors.

        let new_kind;
        if c.is_whitespace() {
            new_kind = Kind::Space;
        } else if c.is_alphabetic() {
            new_kind = Kind::Alphabetic;
        } else if c.is_numeric() {
            new_kind = Kind::Numeric;
        } else {
            new_kind = Kind::Separator;
        }

        if kind != new_kind || kind == Kind::Separator {
            // save the current token.
            if kind != Kind::Space {
                tokens.push(current.clone());
            }

            // reset current token.
            current.string.clear();
            current.span.clear(); // the span here is moved to the end of itself.
        }

        if new_kind != Kind::Space {
            current.string.push(c);
        }

        current.span.push(c);
        kind = new_kind;
    }

    if !current.string.is_empty() {
        tokens.push(current);
    }

    for token in &tokens {
        println!("{}: {}", token.string, token.span);
    }

    Ok(())
}
