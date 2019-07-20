extern crate utf8_decode;
extern crate source_span;

use std::fs::File;
use std::io::Read;
use utf8_decode::UnsafeDecoder;
use source_span::Span;

#[derive(Clone, Default)]
pub struct Token {
    string: String,
    span: Span
}

fn main() -> std::io::Result<()> {
    let file = File::open("examples/file.txt")?;
    let chars = UnsafeDecoder::new(file.bytes());

    let mut tokens = Vec::new();
    let mut current = Token::default();

    for c in chars {
        let c = c?; // report eventual errors.
        if c.is_whitespace() {
            // save the current token.
            if !current.string.is_empty() {
                tokens.push(current.clone());
            }

            // reset current token.
            current.string.clear();
            current.span.clear(); // the span here is moved to the end of itself.
        } else {
            current.string.push(c);
            current.span.push(c);
        }
    }

    if !current.string.is_empty() {
        tokens.push(current);
    }

    for token in &tokens {
        println!("{}: {}", token.string, token.span);
    }

    Ok(())
}
