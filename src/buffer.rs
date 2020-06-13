use std::cell::RefCell;

use crate::{Metrics, Position, Span};

/// Lazy string buffer that fills up on demand, can be iterated and indexed by
/// character position.
///
/// The `SourceBuffer` wraps aroung a `char` iterator. It can be itself used as
/// a `char` iterator, or as a `SourceBuffer` to access an arbitrary fragment of
/// the input source stream.
pub struct SourceBuffer<E, I: Iterator<Item = Result<char, E>>, M: Metrics> {
	p: RefCell<Inner<E, I>>,

	/// Metrics used.
	metrics: M,
}

struct Inner<E, I: Iterator<Item = Result<char, E>>> {
	/// Input source `char` stream.
	input: I,

	/// SourceBuffer error state.
	error: Option<E>,

	/// Buffer data.
	data: Vec<char>,

	/// Lines index.
	///
	/// Contains the index of the first character of each line.
	lines: Vec<usize>,

	/// Span of the buffer.
	span: Span,
}

impl<E, I: Iterator<Item = Result<char, E>>> Inner<E, I> {
	/// Read the next line from the input stream and add it to the buffer.
	/// Returns `true` if a new line has been added. Returns `false` if the
	/// source stream is done.
	fn read_line<M: Metrics>(&mut self, metrics: &M) -> bool {
		if self.error.is_none() {
			let line = self.span.end().line;
			while line == self.span.end().line {
				match self.input.next() {
					Some(Ok(c)) => {
						self.data.push(c);
						self.span.push(c, metrics);
					}
					Some(Err(e)) => {
						self.error = Some(e);
						return false;
					}
					None => return false,
				}
			}

			// register the next line index.
			self.lines.push(self.data.len());

			true
		} else {
			false
		}
	}

	/// Get the index of the char at the given cursor position if it is in the
	/// buffer. If it is not in the buffer but after the buffered content,
	/// the input stream will be read until the buffer span includes the
	/// given position.
	///
	/// Returns `None` if the given position if previous to the buffer start
	/// positions, if the source stream ends before the given position, or
	/// if the line at the given position is shorter than the given position
	/// column.
	fn index_at<M: Metrics>(&mut self, pos: Position, metrics: &M) -> Result<Option<usize>, E> {
		if pos < self.span.start() {
			Ok(None)
		} else {
			while pos >= self.span.end() && self.read_line(metrics) {}

			if pos >= self.span.end() {
				let mut error = None;
				std::mem::swap(&mut error, &mut self.error);
				match error {
					Some(e) => Err(e),
					None => Ok(None),
				}
			} else {
				// line index relative to the first line of the buffer.
				let relative_line = pos.line - self.span.start().line;
				// get the index of the char of the begining of the line in the buffer.
				let mut i = self.lines[relative_line];
				// place a virtual cursor at the begining of the target line.
				let mut cursor = Position::new(pos.line, 0);

				while cursor < pos {
					cursor = cursor.next(self.data[i], metrics);
					i += 1;
				}

				if cursor == pos {
					// found it!
					Ok(Some(i))
				} else {
					// the position does not exist in the buffer.
					Ok(None)
				}
			}
		}
	}

	/// Get the character at the given index.
	///
	/// If it is not in the buffer but after the buffered content, the input
	/// stream will be read until the buffer span includes the given
	/// position. Returns `None` if the source stream ends before the given
	/// position.
	fn get<M: Metrics>(&mut self, i: usize, metrics: &M) -> Result<Option<char>, E> {
		while i >= self.data.len() && self.read_line(metrics) {}

		if i >= self.data.len() {
			let mut error = None;
			std::mem::swap(&mut error, &mut self.error);
			match error {
				Some(e) => Err(e),
				None => Ok(None),
			}
		} else {
			Ok(Some(self.data[i]))
		}
	}
}

impl<E, I: Iterator<Item = Result<char, E>>, M: Metrics> SourceBuffer<E, I, M> {
	/// Create a new empty buffer starting at the given position.
	pub fn new(input: I, position: Position, metrics: M) -> Self {
		Self {
			p: RefCell::new(Inner {
				input,
				error: None,
				data: Vec::new(),
				lines: vec![0],
				span: position.into(),
			}),
			metrics,
		}
	}

	/// Get the metrics used by the source buffer to map every character.
	pub fn metrics(&self) -> &M { &self.metrics }

	/// Get the span of the entire buffered data.
	pub fn span(&self) -> Span { self.p.borrow().span }

	/// Get the index of the char at the given cursor position if it is in the
	/// buffer. If it is not in the buffer but after the buffered content,
	/// the input stream will be read until the buffer span includes the
	/// given position.
	///
	/// Returns `None` if the given position if previous to the buffer start
	/// positions, if the source stream ends before the given position, or
	/// if the line at the given position is shorter than the given position
	/// column.
	pub fn index_at(&self, pos: Position) -> Result<Option<usize>, E> {
		self.p.borrow_mut().index_at(pos, &self.metrics)
	}

	/// Get the char at the given position if it is in the buffer.
	/// If it is not in the buffer yet, the input stream will be pulled until
	/// the buffer span includes the given position.
	///
	/// Returns `None` if the given position is out of range, if the source
	/// stream ends before the given position, or if the line at the given
	/// position is shorter than the given position column.
	pub fn at(&self, pos: Position) -> Result<Option<char>, E> {
		match self.index_at(pos) {
			Ok(Some(i)) => self.p.borrow_mut().get(i, &self.metrics),
			Ok(None) => Ok(None),
			Err(e) => Err(e)
		}
	}

	/// Get the character at the given index.
	///
	/// If it is not in the buffer but after the buffered content, the input
	/// stream will be read until the buffer span includes the given
	/// position. Returns `None` if the source stream ends before the given
	/// position.
	pub fn get(&self, i: usize) -> Result<Option<char>, E> { self.p.borrow_mut().get(i, &self.metrics) }

	/// Returns an iterator through the characters of the buffer from the
	/// begining of it.
	///
	/// When it reaches the end of the buffer, the buffer will start reading
	/// from the source stream.
	pub fn iter(&self) -> Iter<E, I, M> {
		Iter {
			buffer: self,
			i: Some(Ok(0)),
			pos: self.p.borrow().span.start(),
			end: Position::end(),
		}
	}

	/// Returns an iterator through the characters of the buffer from the given
	/// position.
	///
	/// If the input position precedes the buffer start position, then it will
	/// start from the buffer start position.
	/// When it reaches the end of the buffer, the buffer will start reading
	/// from the source stream.
	pub fn iter_from(&self, pos: Position) -> Iter<E, I, M> {
		let start = self.p.borrow().span.start();
		let pos = std::cmp::max(start, pos);

		Iter {
			buffer: self,
			i: self.index_at(pos).transpose(),
			pos,
			end: Position::end(),
		}
	}

	/// Returns an iterator through the characters of the buffer in the given
	/// span.
	///
	/// If the input start position precedes the buffer start position, then it
	/// will start from the buffer start position.
	/// When it reaches the end of the buffer, the buffer will start reading
	/// from the source stream.
	pub fn iter_span(&self, span: Span) -> Iter<E, I, M> {
		let start = self.p.borrow().span.start();
		let pos = std::cmp::max(start, span.start());

		Iter {
			buffer: self,
			i: self.index_at(pos).transpose(),
			pos,
			end: span.end(),
		}
	}
}

/// Iterator over the characters of a [`SourceBuffer`].
///
/// This iterator is created using the [`SourceBuffer::iter`] method or the
/// [`SourceBuffer::iter_from`] method. When it reaches the end of the buffer,
/// the buffer will start reading from the source stream until the stream itself
/// return `None`.
pub struct Iter<'b, E, I: 'b + Iterator<Item = Result<char, E>>, M: Metrics> {
	buffer: &'b SourceBuffer<E, I, M>,
	i: Option<Result<usize, E>>,
	pos: Position,
	end: Position,
}

impl<'b, E, I: 'b + Iterator<Item = Result<char, E>>, M: Metrics> Iter<'b, E, I, M> {
	pub fn into_string(self) -> Result<String, E> {
		let mut string = String::new();

		for c in self {
			string.push(c?);
		}

		Ok(string)
	}
}

impl<'b, E, I: 'b + Iterator<Item = Result<char, E>>, M: Metrics> Iterator for Iter<'b, E, I, M> {
	type Item = Result<char, E>;

	fn next(&mut self) -> Option<Result<char, E>> {
		if self.pos >= self.end {
			None
		} else {
			match &mut self.i {
				Some(Ok(ref mut i)) => match self.buffer.get(*i) {
					Ok(Some(c)) => {
						self.pos = self.pos.next(c, self.buffer.metrics());
						*i += 1;
						Some(Ok(c))
					}
					Ok(None) => None,
					Err(e) => Some(Err(e)),
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
				}
			}
		}
	}
}
