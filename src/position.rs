use crate::Metrics;
use std::fmt;

/// Position in a source file.
///
/// This holds the line and column position of a character in a source file.
/// Some operations are available to move position in a file. In partular, the
/// [`next`](Position::next) method computes the next cursor position after
/// reading a given [`char`].
///
/// ## Display
///
/// The struct implements two different format traits:
///
///  * [`fmt::Display`] will format the position as `line {line} column
///    {column}`
///  * [`fmt::Debug`] will format the position as `{line}:{column}`.
///
/// Both of them will display lines and columns starting at `1` even though the
/// internal representation starts at `0`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Position {
	/// Line number, starting at `0`.
	pub line: usize,

	/// Column number, starting at `0`.
	pub column: usize,
}

impl Position {
	/// Create a new position given a line and column.
	///
	/// Indexes starts at `0`.
	#[must_use]
	pub const fn new(line: usize, column: usize) -> Self { Self { line, column } }

	/// Return the maximum position.
	///
	/// # Example
	///
	/// ```
	/// use source_span::Position;
	///
	/// assert_eq!(
	/// 	Position::end(),
	/// 	Position::new(usize::max_value(), usize::max_value())
	/// 	);
	/// ```
	#[must_use]
	pub const fn end() -> Self {
		Self {
			line: usize::max_value(),
			column: usize::max_value(),
		}
	}

	/// Move to the next column.
	#[must_use]
	pub const fn next_column(&self) -> Self {
		Self {
			line: self.line,
			column: self.column + 1,
		}
	}

	/// Move to the begining of the line.
	#[must_use]
	pub const fn reset_column(&self) -> Self {
		Self {
			line: self.line,
			column: 0,
		}
	}

	/// Move to the next line, and reset the column position.
	#[must_use]
	pub const fn next_line(&self) -> Self {
		Self {
			line: self.line + 1,
			column: 0,
		}
	}

	/// Move to the position following the given [`char`] using the given [`Metrics`].
	///
	/// ## Control characters
	///
	/// This crate is intended to help with incremental lexing/parsing.
	/// Therefore, any control character moving the cursor backward will be
	/// ignored: it will be treated as a 0-width character with no
	/// semantics.
	///
	/// ### New lines
	///
	/// The `\n` character is interpreted with the Unix semantics, as the new
	/// line (NL) character. It will reset the column position to `0` and
	/// move to the next line.
	///
	/// ### Tabulations
	///
	/// The `\t` will move the cursor to the next horizontal tab-top.
	/// The length of a tab-stop (in columns) is given by the `metrics` parameter.
	///
	/// ## Full-width characters
	///
	/// Note that, as for now, double-width characters of full-width characters are *not*
	/// supported by the [`DefaultMetrics`].
	/// They will move the cursor by only one column as any other
	/// regular-width character. You are welcome to contribute to handle
	/// them.
	#[must_use]
	pub fn next<M: Metrics>(&self, c: char, metrics: &M) -> Self {
		match c {
			'\n' => self.next_line(),
			'\t' => {
				let ts = metrics.tab_stop();
				Self {
					line: self.line,
					column: (self.column / ts) * ts + ts,
				}
			}
			c if c.is_control() => *self,
			_ => {
				Self {
					line: self.line,
					column: self.column + metrics.char_width(c),
				}
			}
		}
	}

	pub fn shift<M: Metrics>(&mut self, c: char, metrics: &M) { *self = self.next(c, metrics) }
}

impl fmt::Display for Position {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.line == usize::max_value() && self.column == usize::max_value() {
			write!(f, "line [end] column [end]")
		} else if self.line == usize::max_value() {
			write!(f, "line [end] column {}", self.column + 1)
		} else if self.column == usize::max_value() {
			write!(f, "line {} column [end]", self.line + 1)
		} else {
			write!(f, "line {} column {}", self.line + 1, self.column + 1)
		}
	}
}

impl fmt::Debug for Position {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.line == usize::max_value() && self.column == usize::max_value() {
			write!(f, "[end]:[end]")
		} else if self.line == usize::max_value() {
			write!(f, "[end]:{}", self.column + 1)
		} else if self.column == usize::max_value() {
			write!(f, "{}:[end]", self.line + 1)
		} else {
			write!(f, "{}:{}", self.line + 1, self.column + 1)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! min {
        ($x: expr) => ($x);
        ($x: expr, $($z: expr),+ $(,)* ) => (::std::cmp::min($x, min!($($z),*)));
    }

	macro_rules! max {
        ($x: expr) => ($x);
        ($x: expr, $($z: expr),+ $(,)* ) => (::std::cmp::max($x, max!($($z),*)));
    }

	// An order is a total order if it is (for all a, b and c):
	// - total and antisymmetric: exactly one of a < b, a == b or a > b is true; and
	// - transitive, a < b and b < c implies a < c. The same must hold for both ==
	//   and >.
	#[test]
	fn test_ord_position() {
		assert_eq!(
			min!(
				Position::new(1, 2),
				Position::new(1, 3),
				Position::new(1, 4),
				Position::new(1, 2),
				Position::new(2, 1),
				Position::new(3, 12),
				Position::new(4, 4),
			),
			Position::new(1, 2)
		);

		assert_eq!(
			max!(
				Position::new(1, 2),
				Position::new(1, 3),
				Position::new(1, 4),
				Position::new(1, 2),
				Position::new(2, 1),
				Position::new(3, 12),
				Position::new(4, 4),
			),
			Position::new(4, 4)
		);
	}

	#[test]
	fn test_debug() {
		assert_eq!(format!("{:?}", Position::new(2, 3)), "3:4".to_string());
		assert_eq!(
			format!("{:?}", Position::new(usize::max_value(), 3)),
			"[end]:4".to_string()
		);
		assert_eq!(
			format!("{:?}", Position::new(3, usize::max_value())),
			"4:[end]".to_string()
		);
		assert_eq!(
			format!(
				"{:?}",
				Position::new(usize::max_value(), usize::max_value())
			),
			"[end]:[end]".to_string()
		);
	}
}
