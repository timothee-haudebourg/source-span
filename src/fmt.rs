//! Source text formatter with span highlights and notes.
//!
//! Here are the kind of things you can produce with the
//! [`Formatter`] (without colors):
//! ```text
//!  1 | /     pub fn fibonacci(n: i32) -> u64 {
//!    | |                     ^______^        ^
//!    | |  __________________________|________|
//!    | | |                          |
//!    | | |                          this is a pair of parenthesis
//!  2 | | |       if n < 0 {
//!    | | |  ______________^
//!  3 | | | |         panic!("{} is negative!", n);
//!    | | | |               ^"^^             "   ^ this is a pair of parenthesis
//!    | | | |               ||_|_____________|   |
//!    | | | |               |__|_____________|___|
//!    | | | |                  |             |
//!    | | | |                  |             this is a string
//!    | | | |                  |
//!    | | | |                  this is a pair of braces
//!  4 | | | |     } else if n == 0 {
//!    | | | |_____^                ^
//!    | | |  _____|________________|
//!    | | | |     |
//!    | | | |     this is a pair of braces
//!  5 | | | |         panic!("zero is not a right argument to fibonacci()!");
//!    | | | |               ^"                                         ^^ "^ parentheses
//!    | | | |               ||__________________________________________|_||
//!    | | | |               |___________________________________________|_||
//!    | | | |                                                           | |
//!    | | | |                                                           | this is a string
//!    | | | |                                                           |
//!    | | | |                                                           parentheses
//!  6 | | | |     } else if n == 1 {
//!    | | | |_____^                ^
//!    | | |       |                |
//!    | | |  _____|________________|
//!    | | | |     |
//!    | | | |     this is a pair of braces
//!  7 | | | |         return 1;
//!  8 | | | |     }
//!    | | | |_____^ this is a pair of braces
//!  9 | | |
//! 10 | | |       let mut sum = 0;
//! 11 | | |       let mut last = 0;
//! 12 | | |       let mut curr = 1;
//! 13 | | |       for _i in 1..n {
//!    | | |  ____________________^
//! 14 | | | |         sum = last + curr;
//! 15 | | | |         last = curr;
//! 16 | | | |         curr = sum;
//! 17 | | | |     }
//!    | | | |_____^ this is a pair of braces
//! 18 | | |       sum
//! 19 | | |   }
//!    | | |___^^ this is the whole program
//!    | |_____|| what a nice program!
//!    |       |
//!    |       this is a pair of braces
//! 20 |
//! ```

use crate::{Metrics, Position, Span};
use std::fmt;

/// Colors used to render the text.
#[cfg(feature = "colors")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
	Red,
	Green,
	Blue,
	Magenta,
	Yellow,
	Cyan,
}

#[cfg(feature = "colors")]
impl termion::color::Color for Color {
	fn write_fg(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Red => termion::color::LightRed.write_fg(f),
			Self::Green => termion::color::LightGreen.write_fg(f),
			Self::Blue => termion::color::LightBlue.write_fg(f),
			Self::Magenta => termion::color::LightMagenta.write_fg(f),
			Self::Yellow => termion::color::LightYellow.write_fg(f),
			Self::Cyan => termion::color::LightCyan.write_fg(f),
		}
	}

	fn write_bg(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Red => termion::color::LightRed.write_bg(f),
			Self::Green => termion::color::LightGreen.write_bg(f),
			Self::Blue => termion::color::LightBlue.write_bg(f),
			Self::Magenta => termion::color::LightMagenta.write_bg(f),
			Self::Yellow => termion::color::LightYellow.write_bg(f),
			Self::Cyan => termion::color::LightCyan.write_bg(f),
		}
	}
}

#[cfg(not(feature = "colors"))]
pub type Color = ();

/// Highlight format description.
///
/// Specifies how the highlight should be rendered:
///  * What character to use to underline highlights.
/// ```txt
/// 1 | fn main() {
/// 2 |     println!("Hello World!")
///   |              ^++++++++++++^ highlighting this string
/// 3 | }
/// ```
/// In this example, the underline character is `+`.
///
///  * What boundary marker character to use to point the first and last
///    elements of a highlight.
/// ```txt
/// 1 |   fn main() {
///   |  ___________^
/// 2 | |     println!("Hello World!")
/// 3 | | }
///   | |_^ this span covers more than one line
/// ```
/// In this example, the boundary marker is `^`.
/// Note that the underline character is not used.
///
/// ## Colors
///
/// If the `colors` feature is enabled, it is also possible to set a color to
/// draw the lines. This will also make the highlights more bright (or bold),
/// along with the line numbers.
#[derive(Clone, Copy)]
pub enum Style {
	/// Red curvy underline.
	Error,

	/// Yellow curvy underline.
	Warning,

	/// Blue straight underline.
	Note,

	/// Green straight underline.
	Help,

	/// Custom highlight format.
	///
	/// Specifies the underline character, the boundary marker and the color (if
	/// the `colors` feature is enabled) used to render the highlight.
	Custom(char, char, Color),
}

impl Style {
	/// Create a new custom highlight style.
	///
	/// The `line` character is user to draw the line under the span elements.
	/// The `marker` character is used to point to the first and last elements
	/// of the span when relevant.
	#[must_use]
	#[cfg(not(feature = "colors"))]
	pub const fn new(underline: char, marker: char) -> Self { Self::Custom(underline, marker, ()) }

	/// Create a new custom highlight style.
	///
	/// The `line` character is user to draw the line under the highlighted
	/// sections. The `marker` character is used to point to the first and last
	/// elements of the section when relevant.
	#[must_use]
	#[cfg(feature = "colors")]
	pub const fn new(line: char, marker: char, color: Color) -> Self {
		Self::Custom(line, marker, color)
	}

	/// The character used to underline the highlighted section.
	#[must_use]
	pub fn line(&self) -> char {
		match self {
			Self::Error | Self::Warning => '^',
			Self::Note | Self::Help => '_',
			Self::Custom(line, _, _) => *line,
		}
	}

	/// The character used to point the first and last element of the span when
	/// relevant.
	#[must_use]
	pub fn marker(&self) -> char {
		match self {
			Self::Error | Self::Warning | Self::Note | Self::Help => '^',
			Self::Custom(_, marker, _) => *marker,
		}
	}

	/// Get the color used to draw the highlight.
	#[must_use]
	pub fn color(&self) -> Color {
		#[cfg(not(feature = "colors"))]
		{
			()
		}
		#[cfg(feature = "colors")]
		{
			match self {
				Self::Error => Color::Red,
				Self::Warning => Color::Yellow,
				Self::Note => Color::Blue,
				Self::Help => Color::Green,
				Self::Custom(_, _, color) => *color,
			}
		}
	}
}

/// Text highlight.
///
/// Defines what should be highlighted in the text formatted with the
/// [`Formatter`].
/// A highlight is composed of a span a label and a style.
/// The formatter will underline the text in the given span and draw the label's
/// text on its side, with the given style.
///
/// ```txt
/// 1 | fn main() {
/// 2 |     println!("Hello World!")
///   |              ^^^^^^^^^^^^^^ highlighting this string
/// 3 | }
/// ```
/// # Highlight spanning multiple lines
///
/// The highlight span can cover multiple lines.
/// In that case, only the first and last elements will be decorated using the
/// style's marker charatcer (`^` in the below example).
///
/// ```txt
/// 1 |   fn main() {
///   |  ___________^
/// 2 | |     println!("Hello World!")
/// 3 | | }
///   | |_^ this span covers more than one line
/// ```
///
/// # Entangled highlights
///
/// Different highlights can overlap without breaking the formatted output, but
/// it may become difficult to read the exact boundary of each highlight.
///
/// ```txt
/// 1 |   fn main() {
///   |          __ ^
///   |  _________|_|
///   | |         |
///   | |         this is a pair of parenthesis
/// 2 | |     println!("Hello World!")
///   | |             ^^^^^^^^^^^^^^^^ this is a pair of parenthesis
///   | |             |_____________||
///   | |                           |
///   | |                           this is a string. Hard to see where it starts, uh?
/// 3 | | }
///   | |_^ this is a pair of braces
/// ```
///
/// Here the underline character for the string is the same as the boundary
/// marker for the parenthesis, making it hard to see which is which.
/// One possible workaround is to change the [`Style`] of the highlights.
/// Changing the boundary marker for the parenthesis to `|` makes it easier to
/// read the formatted output:
///
/// ```txt
/// 1 |   fn main() {
///   |          __ ^
///   |  _________|_|
///   | |         |
///   | |         this is a pair of parenthesis
/// 2 | |     println!("Hello World!")
///   | |             |^^^^^^^^^^^^^^| this is a pair of parenthesis
///   | |             |_____________||
///   | |                           |
///   | |                           this is a string. Hard to see where it starts, uh?
/// 3 | | }
///   | |_^ this is a pair of braces
/// ```
pub struct Highlight {
	span: Span,
	label: Option<String>,
	style: Style,
}

impl Highlight {
	/// Compute the "margin nesting level" of the highlight.
	///
	/// The "margin nesting level" for multiline highlights correspond to the
	/// horizontal position of the vertical bar in the margin linking the
	/// begining and the end of the highlight. In the example below, one
	/// highlight has a margin nesting level of 1 (further in), and the other of
	/// 2 (further out).
	///
	/// ```text
	///    ___^ ^
	///  _|_____|
	/// | |
	/// | |
	/// |_|________^   ^
	///   |____________|
	/// ```
	///
	/// Single line highlights have a margin nesting level of 0.
	///
	/// The nesting level is computed relatively to other mapped highlights that
	/// have a lower precedence.
	fn margin_nest_level(&self, highlights: &[MappedHighlight]) -> usize {
		if self.span.line_count() > 1 {
			let mut level = 2;
			for h in highlights {
				if self.span.overlaps(h.span()) {
					level = std::cmp::max(level, 2 + h.margin_nest_level)
				}
			}

			level
		} else {
			0
		}
	}

	fn start_nest_level(
		&self,
		highlights: &[MappedHighlight],
		first_non_whitespace: Option<usize>,
	) -> usize {
		if self.span.last.line > self.span.start.line
			&& first_non_whitespace.is_some()
			&& first_non_whitespace.unwrap() >= self.span.start.column
		{
			0
		} else {
			let mut level = 1;
			for h in highlights {
				if (self.span.start.line == h.span().start.line
					|| self.span.start.line == h.span().last.line)
					&& (self.span.overlaps(h.span()) || self.span.line_count() > 1)
				{
					level = std::cmp::max(level, 1 + h.start_nest_level)
				}
			}

			level
		}
	}

	fn end_nest_level(&self, highlights: &[MappedHighlight]) -> usize {
		let mut level = 1;
		for h in highlights {
			if (self.span.last.line == h.span().start.line
				|| self.span.last.line == h.span().last.line)
				&& self.span.overlaps(h.span())
			{
				level = std::cmp::max(level, 1 + h.end_nest_level)
			}
		}

		level
	}
}

/// Text formatter with span highlights.
///
/// This allows you to format a given input `char` stream with highlights and
/// colors (if the `colors` feature is enabled).
/// A [`Highlight`] is defined by a [`Span`], a string label and a [`Style`],
/// and will be rendered with the stream:
///
/// ```txt
/// 1 | fn main() {
/// 2 |     println!("Hello World!")
///   |              ^^^^^^^^^^^^^^ highlighting this string
/// 3 | }
/// ```
///
/// Highlights spans can cover multiple lines and overlap.
/// See the [`Highlight`] documentation for more informations.
pub struct Formatter {
	highlights: Vec<Highlight>,
	margin_color: Color,
	show_line_numbers: bool,
	use_line_begining_shortcut: bool,
	viewbox: Option<usize>,
}

impl Formatter {
	/// Create a new formatter with no highlights.
	///
	/// # Note
	///
	/// By default line numbers are shown. You can disable them using the
	/// [`hide_line_numbers`](Formatter::hide_line_numbers) method.
	#[must_use]
	pub fn new() -> Self { Self::default() }

	/// Create a new formatter with no highlights and the specified margin
	/// color.
	///
	/// # Note
	///
	/// By default line numbers are shown. You can disable them using the
	/// [`hide_line_numbers`](Formatter::hide_line_numbers) method.
	#[must_use]
	pub const fn with_margin_color(margin_color: Color) -> Self {
		Self {
			highlights: Vec::new(),
			margin_color,
			viewbox: Some(2),
			show_line_numbers: true,
			use_line_begining_shortcut: true,
		}
	}

	/// By default, line numbers are shown in a margin in the left side of the
	/// rendered text, like this:
	/// ```text
	/// 1 | fn main() {
	/// 2 |     println!("Hello World!")
	///   |              ^^^^^^^^^^^^^^ highlighting this string
	/// 3 | }
	/// ```
	/// The `margin_color` attribute is used to decorate the margin text (blue
	/// by default). You can use this function to enable or disable this
	/// functionality. Without line numbers, the previous example will look like
	/// this:
	///
	/// ```text
	/// fn main() {
	///     println!("Hello World!")
	///              ^^^^^^^^^^^^^^ highlighting this string
	/// }
	/// ```
	pub fn set_line_numbers_visible(&mut self, visible: bool) { self.show_line_numbers = visible; }

	/// Show the line numbers (this is the default).
	pub fn show_line_numbers(&mut self) { self.show_line_numbers = false; }

	/// Hide the line numbers.
	pub fn hide_line_numbers(&mut self) { self.show_line_numbers = false; }

	/// Set the viewbox (default is 2).
	///
	/// The viewbox is used to ommit non-important lines from the render.
	/// A line is considered important if it is included at the start or end of
	/// an highlighted span.
	/// In the below example, lines 1 and 12 are important. With a viewport of
	/// 2, the two lines around important lines will be visible, and the other
	/// ommited. In this example, lines 4 to 9 are ommited.
	///
	/// ```text
	///  1 |   fn main() {
	///    |  __________^
	///  2 | |     some code;
	///  3 | |     more code;
	/// .. | |
	/// 10 | |     more code;
	/// 11 | |     more code
	/// 12 | | }
	///    | |_^
	/// 13 |
	/// 14 |   fn another_function {
	/// ```
	/// Here is the same example with a viewbox of 1:
	/// ```text
	///  1 |   fn main() {
	///    |  __________^
	///  2 | |     some code;
	/// .. | |
	/// 11 | |     more code
	/// 12 | | }
	///    | |_^
	/// 13 |
	/// ```
	///
	/// You can disable the viewbox all together by passing `None` to this
	/// function. In this case, all the lines will be visible.
	pub fn set_viewbox(&mut self, viewbox: Option<usize>) { self.viewbox = viewbox }

	/// Add a span highlight.
	pub fn add(&mut self, span: Span, label: Option<String>, style: Style) {
		self.highlights.push(Highlight { span, label, style });
		self.highlights.sort_by(|a, b| a.span.cmp(&b.span));
	}
}

/// Highlight with some more information about how to draw the lines.
#[derive(Clone, Copy)]
struct MappedHighlight<'a> {
	h: &'a Highlight,
	margin_nest_level: usize,
	start_nest_level: usize,
	end_nest_level: usize,
}

impl<'a> MappedHighlight<'a> {
	pub const fn span(&self) -> &Span { &self.h.span }

	pub const fn style(&self) -> &Style { &self.h.style }

	pub const fn label(&self) -> Option<&String> { self.h.label.as_ref() }

	fn update_start_nest_level(
		&mut self,
		highlights: &[MappedHighlight],
		first_non_whitespace: Option<usize>,
	) {
		self.start_nest_level = self.h.start_nest_level(highlights, first_non_whitespace)
	}

	fn update_end_nest_level(&mut self, highlights: &[MappedHighlight]) {
		self.end_nest_level = self.h.end_nest_level(highlights)
	}
}

/// Character with style information.
#[derive(Clone, Copy)]
pub enum Char {
	Empty,
	Text(char),
	Margin(char, Color),
	Label(char, Color),
	SpanMarker(char, Color),
	SpanUnderline(char, Color),
	SpanVertical(Color),
	SpanHorizontal(Color),
	SpanMargin(Color),
	SpanMarginMarker(Color),
}

impl Char {
	const fn unwrap(self) -> char {
		match self {
			Self::Empty => ' ',
			Self::Text(c)
			| Self::Margin(c, _)
			| Self::Label(c, _)
			| Self::SpanUnderline(c, _)
			| Self::SpanMarker(c, _) => c,
			Self::SpanVertical(_) => '|',
			Self::SpanHorizontal(_) => '_',
			Self::SpanMargin(_) => '|',
			Self::SpanMarginMarker(_) => '/',
		}
	}

	#[cfg(feature = "colors")]
	const fn color(&self) -> Option<Color> {
		match self {
			Self::Empty | Self::Text(_) => None,
			Self::Margin(_, color)
			| Self::Label(_, color)
			| Self::SpanUnderline(_, color)
			| Self::SpanMarker(_, color)
			| Self::SpanVertical(color)
			| Self::SpanHorizontal(color)
			| Self::SpanMargin(color) | Self::SpanMarginMarker(color) => Some(*color),
		}
	}

	const fn is_free(&self) -> bool {
		match self {
			Self::Empty => true,
			_ => false,
		}
	}

	#[allow(clippy::trivially_copy_pass_by_ref)]
	const fn is_span_horizontal(&self) -> bool {
		match self {
			Self::SpanHorizontal(_) => true,
			_ => false,
		}
	}

	#[allow(clippy::trivially_copy_pass_by_ref)]
	const fn is_span_margin(&self) -> bool {
		match self {
			Self::SpanMargin(_) => true,
			_ => false,
		}
	}
}

impl From<char> for Char {
	fn from(c: char) -> Self { Self::Text(c) }
}

/// A 2D character map.
struct CharMap {
	data: Vec<Char>,
	width: usize,
	height: usize,
}

impl CharMap {
	fn new() -> CharMap {
		CharMap {
			data: vec![Char::Empty],
			width: 1,
			height: 1,
		}
	}

	fn from_label<M: Metrics>(text: &str, color: Color, metrics: &M) -> CharMap {
		let mut map = CharMap {
			data: Vec::with_capacity(text.len()),
			width: 0,
			height: 0,
		};

		let mut pos = Position::new(0, 0);
		for c in text.chars() {
			match c {
				'\n' | '\t' => (),
				_ => map.set(pos.column, pos.line, Char::Label(c, color)),
			}

			pos.shift(c, metrics)
		}

		map
	}

	// fn width(&self) -> usize { self.width }

	fn height(&self) -> usize { self.height }

	fn align<I: Iterator<Item = usize>>(&mut self, width: usize, _height: usize, it: I) {
		for i in it {
			let x = i % width;
			let y = i / width;

			if x < self.width {
				if y < self.height {
					let j = x + y * self.width;
					self.data[i] = self.data[j];
				} else {
					let my = self.height - 1;
					self.data[i] = match (self.get(x, my), self.get(x + 1, my)) {
						(Char::SpanMargin(_), Char::SpanHorizontal(_))
							if x == 0 || !self.get(x - 1, my).is_span_horizontal() =>
						{
							Char::Empty
						}
						(Char::SpanMargin(c), _) => Char::SpanMargin(c),
						(Char::SpanMarginMarker(c), _) => Char::SpanMargin(c),
						(Char::Empty, Char::SpanHorizontal(c)) => Char::SpanMargin(c),
						(Char::Margin('|', c), _) => Char::Margin('|', c),
						_ => Char::Empty,
					}
				}
			} else {
				self.data[i] = Char::Empty
			}
		}
	}

	fn resize(&mut self, width: usize, height: usize) {
		let len = width * height;

		if len != self.data.len() {
			if len > self.data.len() {
				self.data.resize(len, Char::Empty);
			}

			if width < self.width {
				self.align(width, height, 0..len);
			} else {
				self.align(width, height, (0..len).rev());
			}

			if len < self.data.len() {
				self.data.resize(len, Char::Empty);
			}

			self.width = width;
			self.height = height;
		}
	}

	fn reserve(&mut self, width: usize, height: usize) {
		self.resize(
			std::cmp::max(width, self.width),
			std::cmp::max(height, self.height),
		)
	}

	fn get(&self, x: usize, y: usize) -> Char {
		if x >= self.width || y >= self.height {
			Char::Empty
		} else {
			self.data[x + y * self.width]
		}
	}

	fn set(&mut self, x: usize, y: usize, c: Char) {
		self.reserve(x + 1, y + 1);
		self.data[x + y * self.width] = c;
	}

	fn draw_marker(&mut self, style: &Style, y: usize, x: usize) {
		let mut head = false;
		for j in 1..=y {
			let previous_c = self.get(x, j);
			if previous_c.is_free() || previous_c.is_span_horizontal() {
				let c = if head {
					Char::SpanVertical(style.color())
				} else {
					head = true;
					Char::SpanMarker(style.marker(), style.color())
				};

				self.set(x, j, c);
			}
		}
	}

	fn draw_open_line(&mut self, style: &Style, y: usize, start: usize, end: usize) {
		self.reserve(end + 1, y + 1);
		for x in start..=end {
			if x == end {
				self.draw_marker(style, y, x)
			} else {
				if !self.get(x, y).is_span_margin() {
					self.set(x, y, Char::SpanHorizontal(style.color()))
				}
			}
		}
	}

	fn draw_closed_line(&mut self, style: &Style, y: usize, start: usize, end: usize) {
		self.reserve(end + 1, y + 1);
		for x in start..=end {
			if x == start || x == end {
				self.draw_marker(style, y, x)
			} else {
				let c = if y == 1 {
					Char::SpanUnderline(style.line(), style.color())
				} else {
					Char::SpanHorizontal(style.color())
				};

				self.set(x, y, c)
			}
		}
	}

	/// Checks if the given rectangle is free in the char map.
	fn is_rect_free(&self, offset_x: usize, offset_y: usize, width: usize, height: usize) -> bool {
		for y in offset_y..(offset_y + height) {
			for x in offset_x..(offset_x + width) {
				if !self.get(x, y).is_free() {
					return false;
				}
			}
		}

		true
	}

	fn draw_charmap(&mut self, offset_x: usize, offset_y: usize, map: &CharMap) {
		self.reserve(offset_x + map.width, offset_y + map.height);
		for y in 0..map.height {
			for x in 0..map.width {
				self.set(offset_x + x, offset_y + y, map.get(x, y))
			}
		}
	}

	fn draw_charmap_if_free(&mut self, offset_x: usize, offset_y: usize, map: &CharMap) -> bool {
		let mut dx = 0;
		let mut dy = 0;

		if offset_x > 0 {
			dx = 1;
		}

		if offset_y > 1 {
			dy = 1;
		}

		if self.is_rect_free(
			offset_x - dx,
			offset_y - dy,
			map.width + dx + 1,
			map.height + dy + 1,
		) {
			self.draw_charmap(offset_x, offset_y, map);
			true
		} else {
			false
		}
	}
}

impl fmt::Display for CharMap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		#[cfg(feature = "colors")]
		let mut current_color = None;
		for y in 0..self.height {
			for x in 0..self.width {
				let i = x + y * self.width;
				let c = self.data[i];
				#[cfg(feature = "colors")]
				{
					if c.color() != current_color && !c.is_free() {
						current_color = c.color();
						if let Some(color) = current_color {
							write!(f, "{}{}", termion::style::Bold, termion::color::Fg(color))?;
						} else {
							write!(f, "{}", termion::style::Reset)?;
						}
					}
				}
				c.unwrap().fmt(f)?;
			}
			write!(f, "\n")?;
		}

		#[cfg(feature = "colors")]
		write!(f, "{}", termion::style::Reset)?;

		Ok(())
	}
}

/// Formatted text.
///
/// This is the result of the [`Formatter::render`] function.
/// It implements [`Display`](`fmt::Display`) and can hence be printted with a simple `printf!`.
pub struct Formatted(Vec<CharMap>);

impl fmt::Display for Formatted {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for map in &self.0 {
			map.fmt(f)?;
		}

		Ok(())
	}
}

/// A set of important lines to render.
pub enum ImportantLines {
	All,
	Lines(Vec<usize>, usize),
}

impl ImportantLines {
	fn includes(&self, line: usize) -> bool {
		match self {
			ImportantLines::All => true,
			ImportantLines::Lines(important_lines, viewbox) => important_lines
				.binary_search_by(|candidate| {
					use std::cmp::Ordering;
					if line <= candidate + viewbox
						&& line >= candidate - std::cmp::min(candidate, viewbox)
					{
						Ordering::Equal
					} else if line <= candidate + viewbox {
						Ordering::Greater
					} else {
						Ordering::Less
					}
				})
				.is_ok(),
		}
	}
}

impl Formatter {
	fn important_lines(&self) -> ImportantLines {
		if let Some(viewbox) = self.viewbox {
			let mut important_lines = Vec::new();
			for h in &self.highlights {
				important_lines.push(h.span.start.line);
				if h.span.start.line != h.span.last.line {
					important_lines.push(h.span.last.line)
				}
			}

			important_lines.sort_unstable();
			ImportantLines::Lines(important_lines, viewbox)
		} else {
			ImportantLines::All
		}
	}

	fn line_number_margin(&self, span: &Span) -> usize {
		if self.show_line_numbers {
			let last_line = match self.viewbox {
				Some(viewbox) => {
					if let Some(last_highlight) = self.highlights.last() {
						last_highlight.span.last().line + viewbox
					} else {
						return 0
					}
				},
				None => span.last().line
			};

			(((last_line + 1) as f32).log10() as usize) + 4
		} else {
			0
		}
	}

	/// Render the given input stream of character.
	/// The result implements [`Display`](`fmt::Display`) and can then be printed.
	///
	/// ```
	/// # use std::fs::File;
	/// # use std::io::Read;
	/// # use source_span::{DEFAULT_METRICS, SourceBuffer, Position};
	/// # use source_span::fmt::{Color, Style, Formatter};
	/// let file = File::open("examples/fib.txt").unwrap();
	/// let chars = utf8_decode::UnsafeDecoder::new(file.bytes());
	/// let metrics = DEFAULT_METRICS;
	/// let buffer = SourceBuffer::new(chars, Position::default(), metrics);
	///
	/// let mut fmt = Formatter::with_margin_color(Color::Blue);
	/// fmt.add(buffer.span(), None, Style::Error);
	///
	/// let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();
	/// println!("{}", formatted);
	/// ```
	pub fn render<E, I: Iterator<Item = Result<char, E>>, M: Metrics>(
		&self,
		input: I,
		span: Span,
		metrics: &M,
	) -> Result<Formatted, E> {
		let mut mapped_highlights = Vec::with_capacity(self.highlights.len());
		let mut nest_margin = 0;
		for h in &self.highlights {
			let margin_nest_level = h.margin_nest_level(&mapped_highlights);
			// let start_nest_level = 0;
			// let end_nest_level = h.end_nest_level(&mapped_highlights);

			if margin_nest_level > nest_margin {
				nest_margin = margin_nest_level;
			}

			mapped_highlights.push(MappedHighlight {
				h,
				margin_nest_level,
				start_nest_level: 0,
				end_nest_level: 0,
			});
		}

		let line_number_margin = self.line_number_margin(&span);
		let margin = line_number_margin + nest_margin;

		let mut pos = span.start();
		let mut lines = vec![CharMap::new()];
		let important_lines = self.important_lines();
		let mut is_important_line = important_lines.includes(pos.line);
		if is_important_line {
			lines.push(CharMap::new())
		}
		let mut first_non_whitespace = None;
		for c in input {
			if pos > span.last() {
				break;
			}

			let c = c?;
			let x = margin + pos.column;

			match c {
				'\n' => {
					if is_important_line {
						let line_charmap = lines.last_mut().unwrap();
						self.draw_line_number(Some(pos.line), line_charmap, line_number_margin);
						self.draw_line_highlights(
							pos.line,
							line_charmap,
							margin,
							&mut mapped_highlights,
							metrics,
							first_non_whitespace,
						);
					}
					first_non_whitespace = None;
					if important_lines.includes(pos.line + 1) {
						if !is_important_line && !lines.is_empty() {
							let mut viewbox_charmap = CharMap::new();
							self.draw_line_number(None, &mut viewbox_charmap, line_number_margin);
							self.draw_line_highlights(
								pos.line,
								&mut viewbox_charmap,
								margin,
								&mut mapped_highlights,
								metrics,
								None,
							);
							lines.push(viewbox_charmap)
						}
						is_important_line = true
					} else {
						is_important_line = false
					}

					if is_important_line {
						lines.push(CharMap::new())
					}
				}
				'\t' => (),
				_ => {
					if is_important_line {
						if self.use_line_begining_shortcut
							&& first_non_whitespace.is_none()
							&& !c.is_whitespace() && !c.is_control()
						{
							first_non_whitespace = Some(pos.column)
						}

						lines.last_mut().unwrap().set(x, 0, Char::Text(c))
					}
				}
			}

			pos.shift(c, metrics)
		}

		if is_important_line {
			let line_charmap = lines.last_mut().unwrap();
			self.draw_line_number(Some(pos.line), line_charmap, line_number_margin);
			self.draw_line_highlights(
				pos.line,
				line_charmap,
				margin,
				&mut mapped_highlights,
				metrics,
				first_non_whitespace,
			);
		}

		Ok(Formatted(lines))
	}

	fn draw_line_number(
		&self,
		line: Option<usize>,
		charmap: &mut CharMap,
		line_number_margin: usize,
	) {
		if line_number_margin > 0 {
			charmap.set(
				line_number_margin - 2,
				0,
				Char::Margin('|', self.margin_color),
			);
			match line {
				Some(mut line) => {
					let mut x = line_number_margin - 3;
					line += 1;

					while line > 0 {
						x -= 1;
						let d = line % 10;

						charmap.set(
							x,
							0,
							Char::Margin(
								std::char::from_digit(d as u32, 10).unwrap(),
								self.margin_color,
							),
						);

						line /= 10;
					}
				}
				None => {
					for x in 0..(line_number_margin - 3) {
						charmap.set(x, 0, Char::Margin('.', self.margin_color))
					}
				}
			}
		}
	}

	fn draw_line_highlights<M: Metrics>(
		&self,
		line: usize,
		charmap: &mut CharMap,
		margin: usize,
		highlights: &mut [MappedHighlight],
		metrics: &M,
		first_non_whitespace: Option<usize>,
	) {
		// span lines
		for i in 0..highlights.len() {
			let mut h = highlights[i];

			let mut shortcut = false;
			if h.span().start.line == line {
				h.update_start_nest_level(&highlights[0..i], first_non_whitespace);

				if h.span().last.line == line {
					charmap.draw_closed_line(
						h.style(),
						h.start_nest_level,
						margin + h.span().start.column,
						margin + h.span().last.column,
					)
				} else {
					if first_non_whitespace.is_some()
						&& h.span().start.column <= first_non_whitespace.unwrap()
					{
						// line begining shortcut
						shortcut = true;
						charmap.set(
							margin - h.margin_nest_level,
							0,
							Char::SpanMarginMarker(h.style().color()),
						)
					} else {
						charmap.draw_open_line(
							h.style(),
							h.start_nest_level,
							margin - h.margin_nest_level + 1,
							margin + h.span().start.column,
						)
					}
				}
			} else if h.span().last.line == line {
				h.update_end_nest_level(&highlights[0..i]);
				charmap.draw_open_line(
					h.style(),
					h.end_nest_level,
					margin - h.margin_nest_level + 1,
					margin + h.span().last.column,
				);
				// charmap.set(margin - h.margin_nest_level, h.end_nest_level,
				// Char::SpanMargin(h.style().color()))
			}

			if shortcut || (h.span().start.line < line && h.span().last.line >= line) {
				let end = if h.span().last.line == line {
					h.end_nest_level
				} else {
					charmap.height() - 1
				};

				let x = margin - h.margin_nest_level;
				let offset_y = if shortcut { 1 } else { 0 };

				for y in offset_y..=end {
					charmap.set(x, y, Char::SpanMargin(h.style().color()))
				}
			}

			highlights[i] = h;
		}

		// labels
		for h in highlights.iter().rev() {
			if h.span().last.line == line {
				if let Some(label) = h.label() {
					let label_charmap = CharMap::from_label(&label, h.style().color(), metrics);
					let x = margin + h.span().last.column;
					let mut y = 1;
					if !charmap.draw_charmap_if_free(x + 2, y, &label_charmap) {
						y += 2;
						while !charmap.draw_charmap_if_free(x, y, &label_charmap) {
							y += 1;
						}
					}

					for vy in 2..y {
						charmap.set(x, vy, Char::SpanVertical(h.style().color()));
					}
				}
			}
		}
	}
}

impl Default for Formatter {
	fn default() -> Formatter {
		Formatter {
			highlights: Vec::new(),
			#[cfg(not(feature = "colors"))]
			margin_color: (),
			#[cfg(feature = "colors")]
			margin_color: Color::Blue,
			viewbox: Some(2),
			show_line_numbers: true,
			use_line_begining_shortcut: true,
		}
	}
}
