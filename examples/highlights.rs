extern crate source_span;
extern crate utf8_decode;

use source_span::{
    fmt::{Color, Formatter, Style},
    Position, Span,
};
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
    let file = File::open("examples/fib.txt")?;
    let chars = UnsafeDecoder::new(file.bytes());
    let buffer = source_span::lazy::Buffer::new(chars, Position::default());

    let mut fmt = Formatter::new(Color::Blue);

    let mut tokens = Vec::new();
    let mut current = Token::default();
    let mut kind = Kind::Space;

    let mut opened = Vec::new();
    let mut groups = Vec::new();

    let string_style = Style::new('^', '"', Color::Green);

    for c in buffer.iter() {
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

        match c {
            '(' => opened.push((current.span.start(), ')')),
            '[' => opened.push((current.span.start(), ']')),
            '{' => opened.push((current.span.start(), '}')),
            '"' if opened.is_empty() || opened.last().unwrap().1 != '"' => {
                opened.push((current.span.start(), '"'))
            }
            ')' | ']' | '}' | '"' => {
                if let Some((start, expected)) = opened.pop() {
                    if c == expected {
                        let span = Span::new(start, current.span.last(), current.span.end());
                        groups.push(span);
                        let (label, style) = match c {
                            ')' => ("this is a pair of parenthesis", Style::Note),
                            ']' => ("this is a pair of brackets", Style::Warning),
                            '}' => ("this is a pair of braces", Style::Warning),
                            _ => ("this is a string", string_style),
                        };
                        fmt.add(span, Some(label.to_string()), style);
                    } else {
                        // ...
                    }
                } else {
                    // ...
                }
            }
            _ => (),
        }
    }

    if !current.string.is_empty() {
        tokens.push(current);
    }

    let formatted = fmt.get(buffer.iter(), buffer.span())?;
    println!("{}", formatted);

    Ok(())
}
