use std::iter::Extend;
use crate::{
	Position,
	Span,
	Metrics
};

/// Text layout.
///
/// Keep track of the byte index of each line in a UTF8-encoded so it can be indexed by cursor
/// position.
pub struct Layout<M: Metrics> {
	/// Byte index of each line.
	lines: Vec<usize>,

	/// Span of the text.
	span: Span,

	/// Metrics used.
	metrics: M,

	/// Byte length of the text.
	len: usize
}

impl<M: Metrics> Layout<M> {
	/// Create a new empty layout from a given metrics.
	pub fn new(metrics: M) -> Layout<M> {
		Layout {
			lines: vec![0],
			span: Span::default(),
			metrics,
			len: 0
		}
	}

	/// Get the layout's span.
	pub fn span(&self) -> Span {
		self.span
	}

	/// Create a new layout from a `char` iterator.
	pub fn from<Chars: Iterator<Item=char>>(chars: Chars, metrics: M) -> Layout<M> {
		let mut layout = Layout::new(metrics);
		layout.extend(chars);
		layout
	}

	/// Try to create a new layout from an unreliable `char` iterator.
	pub fn try_from<E, Chars: Iterator<Item=Result<char, E>>>(chars: Chars, metrics: M) -> Result<Layout<M>, E> {
		let mut layout = Layout::new(metrics);

		for c in chars {
			layout.push(c?)
		}

		Ok(layout)
	}

	/// Extend the layout with a new character.
	pub fn push(&mut self, c: char) {
		self.span.push(c, &self.metrics);
		self.len += c.len_utf8();
		if c == '\n' {
			self.lines.push(self.len)
		}
	}

	/// Get the byte index mapping to the given position in the input string slice.
	///
	/// It is assumed that the input string slice matches the layout.
	/// Otherwise, the returned index may not point to an UTF8 character boundary nor even be in
	/// the slice bounds.
	pub fn byte_index(&self, str: &str, position: Position) -> Option<usize> {
		if let Some(line_offset) = self.lines.get(position.line) {
			let mut column = 0;
			for (i, c) in str[*line_offset..].char_indices() {
				if column == position.column {
					return Some(line_offset + i)
				}

				if c == '\n' {
					return None
				}

				column += self.metrics.char_width(c)
			}
		}

		None
	}

	/// Get the sub slice of the input string matching the given span.
	pub fn span_slice<'a>(&self, str: &'a str, span: Span) -> &'a str {
		let start = match self.byte_index(str, span.start) {
			Some(index) => index,
			None => 0
		};

		let end = match self.byte_index(str, span.end) {
			Some(index) => index,
			None => str.len()
		};

		&str[start..end]
	}
}

impl<M: Metrics> Extend<char> for Layout<M> {
	fn extend<Chars: IntoIterator<Item=char>>(&mut self, chars: Chars) {
		for c in chars {
			self.push(c)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_byte_index1() {
		let str = "Hello World!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.byte_index(str, Position::new(0, 2)), Some(2));
	}

	#[test]
	fn get_byte_index2() {
		let str = "Hello\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.byte_index(str, Position::new(1, 0)), Some(6));
	}

	#[test]
	fn get_byte_index3() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.byte_index(str, Position::new(2, 0)), Some(7));
	}

	#[test]
	fn get_byte_index_out_of_bounds1() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.byte_index(str, Position::new(3, 0)), None);
	}

	#[test]
	fn get_byte_index_out_of_bounds2() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.byte_index(str, Position::new(1, 3)), None);
	}

	#[test]
	fn get_span_slice1() {
		let str = "Hello\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		assert_eq!(layout.span_slice(str, layout.span), str);
	}

	#[test]
	fn get_span_slice2() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		let span = Span::new(Position::new(0, 0), Position::new(0, 3), Position::new(1, 0));
		assert_eq!(layout.span_slice(str, span), "Hel\n");
	}

	#[test]
	fn get_span_slice3() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		let span = Span::new(Position::new(1, 0), Position::new(1, 2), Position::new(2, 0));
		assert_eq!(layout.span_slice(str, span), "lo\n");
	}

	#[test]
	fn get_span_slice4() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		let span = Span::new(Position::new(2, 0), Position::new(2, 5), Position::new(2, 6));
		assert_eq!(layout.span_slice(str, span), "World!");
	}

	#[test]
	fn get_span_slice5() {
		let str = "Hel\nlo\nWorld!";
		let layout = Layout::from(str.chars(), crate::DEFAULT_METRICS);

		let span = Span::new(Position::new(0, 2), Position::new(2, 2), Position::new(2, 3));
		assert_eq!(layout.span_slice(str, span), "l\nlo\nWor");
	}
}
