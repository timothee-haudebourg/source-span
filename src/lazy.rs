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
        if self.error.is_none() {
            let line = self.span.end().line;
            while line == self.span.end().line {
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

	/// Get the index of the char at the given cursor position if it is in the buffer.
    /// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
    ///
    /// Returns `None` if the given position if previous to the buffer start positions, if the
    /// source stream ends before the given position, or if the line at the given position is
    /// shorter than the given position column.
	fn index_at(&mut self, pos: Position) -> Option<Result<usize>> {
		if pos < self.span.start() {
			None
		} else {
			while pos >= self.span.end() && self.read_line() { }

			if pos >= self.span.end() {
                let mut error = None;
                std::mem::swap(&mut error, &mut self.error);
				match error {
                    Some(e) => Some(Err(e)),
                    None => None
                }
			} else {
                // line index relative to the first line of the buffer.
				let relative_line = pos.line - self.span.start().line;
                // get the index of the char of the begining of the line in the buffer.
                let mut i = self.lines[relative_line];
                // place a virtual cursor at the begining of the target line.
				let mut cursor = Position::new(pos.line, 0);

                while cursor < pos {
                    cursor = cursor.next(self.data[i]);
                    i += 1;
                }

                if cursor == pos {
                    // found it!
                    Some(Ok(i))
                } else {
                    // the position does not exist in the buffer.
                    None
                }
			}
		}
	}

	/// Get the character at the given index.
	///
	/// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
	/// Returns `None` if the source stream ends before the given position.
	fn get(&mut self, i: usize) -> Option<Result<char>> {
		while i >= self.data.len() && self.read_line() { }

		if i >= self.data.len() {
			let mut error = None;
			std::mem::swap(&mut error, &mut self.error);
			match error {
				Some(e) => Some(Err(e)),
				None => None
			}
		} else {
			Some(Ok(self.data[i]))
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
                lines: vec![0],
				span: position.into()
			})
		}
	}

	/// Get the span of the entire buffered data.
	pub fn span(&self) -> Span {
		self.p.borrow().span
	}

	/// Get the index of the char at the given cursor position if it is in the buffer.
    /// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
    ///
    /// Returns `None` if the given position if previous to the buffer start positions, if the
    /// source stream ends before the given position, or if the line at the given position is
    /// shorter than the given position column.
	pub fn index_at(&self, pos: Position) -> Option<Result<usize>> {
		self.p.borrow_mut().index_at(pos)
	}

    /// Get the char at the given position if it is in the buffer.
    /// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
    ///
    /// Returns `None` if the given position if previous to the buffer start positions, if the
    /// source stream ends before the given position, or if the line at the given position is
    /// shorter than the given position column.
	pub fn at(&self, pos: Position) -> Option<Result<char>> {
		match self.index_at(pos) {
			Some(Ok(i)) => self.p.borrow_mut().get(i),
			Some(Err(e)) => Some(Err(e)),
			None => None
		}
	}

	/// Get the character at the given index.
	///
	/// If it is not in the buffer but after the buffered content, the input stream will be read
    /// until the buffer span includes the given position.
	/// Returns `None` if the source stream ends before the given position.
	fn get(&self, i: usize) -> Option<Result<char>> {
		self.p.borrow_mut().get(i)
	}

    /// Returns an iterator through the characters of the buffer from the begining of it.
    ///
    /// When it reaches the end of the buffer, the buffer will start reading from the source
    /// stream.
	pub fn iter(&self) -> Iter<I> {
		Iter {
			buffer: &self,
			i: Some(Ok(0))
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
			i: self.index_at(std::cmp::max(self.p.borrow().span.start(), pos))
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
    i: Option<Result<usize>>
}

impl<'b, I: 'b + Iterator<Item=Result<char>>> Iterator for Iter<'b, I> {
	type Item = Result<char>;

	fn next(&mut self) -> Option<Result<char>> {
		match &mut self.i {
			Some(Ok(ref mut i)) => {
				match self.buffer.get(*i) {
					Some(Ok(c)) => {
						*i = *i+1;
						Some(Ok(c))
					},
		            Some(Err(e)) => Some(Err(e)),
					None => None
				}
			},
			None => None,
			ref mut i => {
				let mut new_i = None;
				std::mem::swap(&mut new_i, i);
				if let Some(Err(e)) = new_i {
					Some(Err(e))
				} else {
					unreachable!()
				}
			},
		}
	}
}
