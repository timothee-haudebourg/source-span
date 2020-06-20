//! This crate provides utilities to locate characters and ranges of characters
//! (spans) in a source file. It also provides ways to print fragments of the
//! source file with span informations, hints, errors, warning and notes,
//! just like the `rustc` compiler.
//!
//! ## Basic usage
//!
//! This crate is designed as an incremental parsing utility.
//! Its primary function is to keep track of the line and column position of
//! each character in a character stream:
//! ```rust
//! use source_span::Position;
//!
//! let metrics = &source_span::DEFAULT_METRICS; // characters metrics
//! let mut pos = Position::new(0, 0);
//! let str = "Hello\nWorld!";
//!
//! for c in str.chars() {
//! 	// `pos` holds the position (line, column) of
//! 	// the current character at all points.
//! 	pos.shift(c, metrics)
//! }
//! ```
//!
//! Using the `Span` type, it is also possible to build ranges of characters.
//!
//! ```rust
//! # use source_span::{Position, Span};
//! # let metrics = source_span::DEFAULT_METRICS;
//! let mut chars = "1 + (2 * 2) / 3".chars();
//! let mut pos = Position::new(0, 0);
//! while let Some(c) = chars.next() {
//! 	if c == '(' {
//! 		break
//! 	}
//!
//! 	pos.shift(c, &metrics)
//! }
//!
//! let mut span: Span = pos.into();
//!
//! while let Some(c) = chars.next() {
//! 	span.push(c, &metrics);
//!
//! 	if c == ')' {
//! 		break
//! 	}
//! }
//!
//! // `span` now holds the beginning and end position of the `"(2 * 2)"` slice.
//! ```
//! ## SourceBuffer
//!
//! This crate provides a simple `SourceBuffer` buffer
//! to index a character stream by character position.
//!
//! ```rust
//! # use std::io::Read;
//! use std::fs::File;
//! use source_span::{DEFAULT_METRICS, Position, SourceBuffer};
//!
//! let file = File::open("examples/fib.txt").unwrap();
//! let chars = utf8_decode::UnsafeDecoder::new(file.bytes());
//! let metrics = DEFAULT_METRICS;
//! let buffer = SourceBuffer::new(chars, Position::default(), metrics);
//!
//! buffer.at(Position::new(4, 2)); // get the character at line 4, column 2.
//! ```
//!
//! The `SourceBuffer` type works as a wrapper around a character iterator.
//! It is lazy: new characters are pulled from the wrapped iterator and put in
//! the buffer only when needed.
//! It can be used to access characters at a specific cursor position (as seen
//! above) or iterate a slice of the text using a `Span`:
//!
//! ```rust
//! # use std::io::Read;
//! # use std::fs::File;
//! # use source_span::{DEFAULT_METRICS, Position, SourceBuffer};
//! # let file = File::open("examples/fib.txt").unwrap();
//! # let chars = utf8_decode::UnsafeDecoder::new(file.bytes());
//! # let metrics = DEFAULT_METRICS;
//! # let buffer = SourceBuffer::new(chars, Position::default(), metrics);
//! # let span = buffer.span();
//! for c in buffer.iter_span(span) {
//!     // do something.
//! }
//! ```
//!
//! ## Formatting
//!
//! This crate also provides a way to format decorated text, highlighting
//! portions of the source text using ASCII art.
//! It can be used to produce outputs similar as the following:
//!
//! ```txt
//! 1 |   fn main() {
//!   |  ___________^
//! 2 | |     println!("Hello World!")
//!   | |              ^^^^^^^^^^^^^^ a string
//! 3 | | }
//!   | |_^ a block
//! ```
//!
//! Each highlight is described by a span, can be associated to a label and
//! drawn with a specific style (defining what characters and color to use to
//! draw the lines).

#![allow(clippy::needless_doctest_main)]
#![warn(clippy::nursery, clippy::must_use_candidate, clippy::pedantic)]
use std::cmp::{Ord, Ordering, PartialOrd};

mod buffer;
pub mod fmt;
mod loc;
mod metrics;
mod position;
mod layout;

pub use buffer::SourceBuffer;
pub use loc::Loc;
pub use metrics::*;
pub use position::Position;
pub use layout::*;

/// Span in a source file.
///
/// A span points to a range of caracters between two cursor [`Position`].
///
/// ## Span construction with the `push*` methods
///
/// A span can be directly created using the [`new`](Span::new) method, however
/// in the context of parsing (or lexing) it might be useful to build spans
/// incrementally. The `push*` methods family will help you do that.
///
///   * [`push`](Span::push) will extend the span to include the given character
///     located at the spans `end`.
///   * [`push_column`](Span::push_column) will extend the span to include the
///     next column. Note that this does not
/// necessarily correspond to the next character (if it is a NL, or a full-width
/// character for instance).
///   * [`push_line`](Span::push_line) will extend the span to include the rest
///     of the line. The end of the span will be
/// placed at the begining of the next line.
///
///   * The [`next`](Span::next) method can finally be used to create the span
///     to `[end, end]` (when a token has
/// been read entirely for instance) and start building the next span. The
/// [`clear`](Span::clear) method does the same but *in place*.
///
/// ## Example
///
/// Here is a basic example computing the span of every word/token in a `char`
/// stream.
///
/// ```rust
/// use source_span::{Span, DEFAULT_METRICS};
///
/// #[derive(Clone, Default)]
/// pub struct Token {
/// 	string: String,
/// 	span: Span,
/// }
///
/// let string = "This is an example String.".to_string();
/// let mut tokens = Vec::new();
/// let mut current = Token::default();
/// let metrics = &DEFAULT_METRICS;
///
/// for c in string.chars() {
/// 	if c.is_whitespace() {
/// 		// save the current token.
/// 		if !current.string.is_empty() {
/// 			tokens.push(current.clone());
/// 		}
///
/// 		// reset current token.
/// 		current.string.clear();
/// 		current.span.clear(); // the span here is moved to the end of itself.
/// 	} else {
/// 		current.string.push(c);
/// 		current.span.push(c, metrics);
/// 	}
/// }
///
/// if !current.string.is_empty() {
/// 	tokens.push(current);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
	/// The position of the first character in the span.
	start: Position,

	/// The last position in the span.
	last: Position,

	/// The position of the character directly following the span.
	///
	/// It is not included in the span.
	end: Position,
}

impl PartialOrd for Span {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Span {
	fn cmp(&self, other: &Self) -> Ordering {
		if self == other {
			Ordering::Equal
		} else if self.includes(other) {
			Ordering::Greater
		} else if other.includes(self) {
			Ordering::Less
		} else {
			self.start.cmp(&other.start)
		}
	}
}

impl Span {
	/// Create a new span from three positions.
	///
	/// If the `end` position or the `last` position is before the `start`
	/// position then the returned span will be `[start, start]`.
	/// If the `last` position is equal to `end` while the span is not empty, it
	/// will panic.
	#[must_use]
	pub fn new(start: Position, mut last: Position, mut end: Position) -> Self {
		if end < start || last < start {
			last = start;
			end = start;
		}

		if last >= end && end != start {
			panic!("invalid span ({:?}, {:?}, {:?})", start, last, end);
		}

		Self { start, last, end }
	}

	pub fn of_string<M: Metrics>(str: &str, metrics: &M) -> Self {
		let mut last = Position::new(0, 0);
		let mut end = Position::new(0, 0);
		for c in str.chars() {
			last = end;
			end.shift(c, metrics)
		}

		Span {
			start: Position::new(0, 0),
			last,
			end,
		}
	}

	/// Return the position of the first character in the span.
	#[must_use]
	pub const fn start(&self) -> Position { self.start }

	/// Return the last position included in the span.
	#[must_use]
	pub const fn last(&self) -> Position { self.last }

	/// Return the position of the character directly following the span.
	///
	/// It is not included in the span.
	#[must_use]
	pub const fn end(&self) -> Position { self.end }

	/// Checks if the span is empty.
	#[must_use]
	pub fn is_empty(&self) -> bool { self.start == self.end }

	/// Checks if two span overlaps.
	#[must_use]
	pub fn overlaps(&self, other: &Span) -> bool {
		(self.start <= other.start && self.end > other.start)
			|| (other.start <= self.start && other.end > self.start)
	}

	/// Checks if the given span is included it this span.
	#[must_use]
	pub fn includes(&self, other: &Span) -> bool {
		self.start <= other.start && self.last >= other.last
	}

	/// The number of lines covered by the span.
	///
	/// It is at least one, even if the span is empty.
	#[must_use]
	pub const fn line_count(&self) -> usize { self.last.line - self.start.line + 1 }

	/// Checks if the span includes the given line.
	#[must_use]
	pub fn includes_line(&self, line: usize) -> bool {
		line >= self.start.line && line <= self.end.line
	}

	/// Extends the span to include the next column.
	///
	/// Note that this does not necessarily correspond
	/// to the next character (if it is a NL, or a full-width character for
	/// instance). To do that you can use the [`push`](Span::push) method.
	pub fn push_column(&mut self) {
		self.last = self.end;
		self.end = self.end.next_column();
	}

	/// Extends the span to include the rest of the line.
	///
	/// The end of the span will be placed at the begining of the next line.
	pub fn push_line(&mut self) {
		self.last = self.end;
		self.end = self.end.next_line();
	}

	/// Extend the span to include the given character located at the spans
	/// `end` position.
	pub fn push<M: Metrics>(&mut self, c: char, metrics: &M) {
		self.last = self.end;
		self.end = self.end.next(c, metrics);
	}

	/// Compute the union of two spans.
	///
	/// If the two spans do not overlap, all positions in between will be
	/// included in the resulting span.
	#[must_use]
	pub fn union(&self, other: Self) -> Self {
		if other.last > self.last && other.end > self.end {
			Self {
				start: std::cmp::min(self.start, other.start),
				last: other.last,
				end: other.end,
			}
		} else {
			Self {
				start: std::cmp::min(self.start, other.start),
				last: self.last,
				end: self.end,
			}
		}
	}

	/// Computes the intersection of the two spans.
	///
	/// If the two spans do not overlap, then the empty span located at the
	/// start of the most advanced span (maximum of the start of the two
	/// spans) is returned.
	#[must_use]
	pub fn inter(&self, other: Self) -> Self {
		let start = std::cmp::max(self.start, other.start);
		Self::new(start, other.last, other.end)
	}

	/// Extend the span to the end of the given span.
	///
	/// This is the *in-place* version of [`union`](Span::union), except that
	/// nothing happens if the input span finishes before the end of `self`.
	pub fn append(&mut self, other: Self) {
		if other.last > self.last && other.end > self.end {
			self.last = other.last;
			self.end = other.end;
		}
	}

	/// Return the next span (defined as `[end, end]`).
	#[must_use]
	pub const fn next(&self) -> Self {
		Self {
			start: self.end,
			last: self.end,
			end: self.end,
		}
	}

	/// Set the span to [`next`](Span::next) (`[end, end]`).
	pub fn clear(&mut self) {
		self.start = self.end;
		self.last = self.end;
	}

	/// Return the span aligned on line boundaries.
	///
	/// This will compute the smallest span including `self` such that
	///  * `start` is at the begining of a line (column 0),
	///  * `end` is at the end of a line (column [`std::usize::MAX`]),
	///  * `last` points to the last character of a line (column
	///    `std::usize::MAX - 1`).
	#[must_use]
	pub const fn aligned(&self) -> Self {
		Self {
			start: Position {
				line: self.start.line,
				column: 0,
			},
			last: Position {
				line: self.end.line,
				column: usize::max_value() - 1,
			},
			end: Position {
				line: self.end.line,
				column: usize::max_value(),
			},
		}
	}
}

impl From<Position> for Span {
	fn from(pos: Position) -> Self {
		Self {
			start: pos,
			last: pos,
			end: pos,
		}
	}
}

impl ::std::fmt::Display for Span {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		if self.start == self.last {
			write!(f, "{}", self.start)
		} else {
			write!(f, "from {:?} to {:?}", self.start, self.end)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_display_span() {
		assert_eq!(
			Span::new(
				Position::new(0, 0),
				Position::new(1, 20),
				Position::new(3, 41),
			)
			.to_string(),
			"from 1:1 to 4:42".to_string()
		);
	}
}
