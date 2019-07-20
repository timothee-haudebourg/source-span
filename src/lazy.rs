use std::cell::{RefCell, RefMut};
use std::io::{Result, Error, Read, Bytes};
use std::ops::{Deref, DerefMut};
use std::fmt;

use crate::{Position, Span};

/// Lazy string buffer that fills up on demand.
///
/// The `lazy::Buffer` wraps aroung a `char` iterator. It can be itself used as a `char` iterator,
/// or as a `Buffer` to access an arbitrary fragment of the input source stream.
pub struct Buffer<I: Iterator<Item=Result<char>>> {
	p: RefCell<Inner<I>>
}

struct Inner<I: Iterator<Item=Result<char>>> {
    /// Input source `char` stream.
	input: I,

    /// Buffer error state.
	error: Option<Error>,

    /// The buffer data.
	data: Vec<char>,

    /// Lines index.
    ///
    /// Contains the index of the first character of each line.
    lines: Vec<usize>,

    /// The span of the buffer.
    span: Span
}

impl<I: Iterator<Item=Result<char>>> Inner<I> {
    /// Read the next line from the input stream and add it to the buffer.
    /// Returns `true` if a new line has been added. Returns `false` if the source stream is
    /// done.
	fn read_line(&mut self) -> bool {
        if self.result.is_none() {
            let line = self.span.begin().line;
            while line == self.span.begin().line {
                match self.input.next() {
                    Some(Ok(c)) => {
                        self.data.push(c);
                        self.span.push(c);
                    },
                    Some(Err(e)) => {
                        self.error = Some(e);
                        return false
                    },
                    None => {
                        return false
                    }
                }
            }

            // register the next line index.
            self.lines.push(self.data.len());

            true
        } else {
            false
        }
	}

	/// Get the char at the given position if it is in the buffer.
    /// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
    ///
    /// Returns `None` if the given position if previous to the buffer start positions, if the
    /// source stream ends before the given position, or if the line at the given position is
    /// shorter than the given position column.
	fn get(&mut self, pos: Position) -> Option<Result<char>> {
		if pos < self.span.start() {
			None
		} else {
			while pos >= self.span.end() && self.read_line() {}

			if pos >= self.span.end() {
				match self.error {
                    Some(e) => Some(Err(e)),
                    None => None
                }
			} else {
                // line index relative to the first line of the buffer.
				let relative_line = pos.line - self.span.start().line;
                // get the index of the char of the begining of the line in the buffer.
                let i = self.lines[relative_line];
                // place a virtual cursor at the begining of the target line.
				let cursor = Position::new(i, 0);

                while cursor < pos {
                    cursor.push(self.data[i]);
                    i += 1;
                }

                if cursor == pos {
                    // found it!
                    Some(Ok(self.data[i]))
                } else {
                    // the position does not exist in the buffer.
                    None
                }
			}
		}
	}
}
//
impl<I: Iterator<Item=Result<char>>> Buffer<I> {
	/// Create a new empty buffer starting at the given position.
	pub fn new(input: I, position: Position) -> Buffer<I> {
		Buffer {
			p: RefCell::new(Inner {
				input: input,
				error: None,
				data: Vec::new(),
                lines: vec![position],
				span: position.into()
			})
		}
	}

    /// Get the char at the given position if it is in the buffer.
    /// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
    ///
    /// Returns `None` if the given position if previous to the buffer start positions, if the
    /// source stream ends before the given position, or if the line at the given position is
    /// shorter than the given position column.
	pub fn get(&self, pos: Position) -> Option<Result<char>> {
		self.p.borrow_mut().get(pos)
	}

    /// Returns an iterator through the characters of the buffer from the begining of it.
    ///
    /// When it reaches the end of the buffer, the buffer will start reading from the source
    /// stream.
	pub fn iter(&self) -> Iter<I> {
		Iter {
			buffer: &self,
			pos: self.p.borrow().start
		}
	}

    /// Returns an iterator through the characters of the buffer from the given position.
    ///
    /// If the input position precedes the buffer start position, that it will start from the
    /// buffer start position.
    /// When it reaches the end of the buffer, the buffer will start reading from the source
    /// stream.
	pub fn iter_from(&self, pos: Position) -> Iter<I> {
		Iter {
			buffer: &self,
			pos: std::cmp::max(self.p.borrow().start, pos)
		}
	}
}

/// Iterator over the characters of a [`Buffer`].
///
/// This iterator is created using the [`Buffer::iter`] method or the [`Buffer::iter_from`] method.
/// When it reaches the end of the buffer, the buffer will start reading from the source
/// stream until the stream itself return `None`.
pub struct Iter<'b, I: 'b + Iterator<Item=Result<char>>> {
	buffer: &'b Buffer<I>,
    pos: Position
}

impl<'b, I: 'b + Iterator<Item=Result<char>>> Iterator for Iter<'b, I> {
	type Item = Result<char>;

	fn next(&mut self) -> Option<Result<char>> {
		match self.buffer.get(self.pos) {
			Some(Ok(c)) => {
				self.pos = self.pos.next(c);
				Some(Ok(c))
			},
            Some(Err(e)) => Some(Err(e)),
			None => None
		}
	}
}
