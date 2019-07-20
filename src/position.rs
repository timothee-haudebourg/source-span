use std::fmt;

/// Position in a source file.
///
/// This holds the line and column position of a character in a source file.
/// Some operations are available to move position in a file. In partular, the [`next`](Position::next) method
/// deduce the next cursor position after reading a given [`char`].
///
/// ## Display
///
/// The struct implements two different format traits:
///
///  * [`fmt::Display`] will format the position as `line {line} column {column}`
///  * [`fmt::Debug`] will format the position as `{line}:{column}`.
///
/// Both of them will display lines and columns starting at `1` even though the internal
/// representation starts at `0`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
    /// Line number, starting at `0`.
    pub line: usize,

    /// Column number, starting at `0`.
    pub column: usize
}

impl Position {
    /// Create a new position given a line and column.
    ///
    /// Indexes starts at `0`.
    pub fn new(line: usize, column: usize) -> Position {
        Position {
            line: line,
            column: column
        }
    }

    /// Move to the next column.
    pub fn next_column(&self) -> Position {
        Position {
            line: self.line,
            column: self.column+1
        }
    }

    /// Move to the begining of the line.
    pub fn reset_column(&self) -> Position {
        Position {
            line: self.line,
            column: 0
        }
    }

    /// Move to the next line, and reset the column position.
    pub fn next_line(&self) -> Position {
        Position {
            line: self.line+1,
            column: 0
        }
    }

    /// Move to the position following the given [`char`].
    ///
    /// ## New lines
    ///
    /// The `\n` character is interpreted with the Unix semantics, as the new line (NL) character.
    /// It will reset the column position to `0` and move to the next line.
    /// However the Windows semantics of `\n` as a line feed (LF) may be supported in the future,
    /// and you are welcome to add it as a contribution.
    ///
    /// ## Full-width characters
    ///
    /// As for now, double-width characters of full-width characters are *not* supported. They
    /// will move the cursor by only one column as any other regular-width character. You are
    /// welcome to contribute to handle them.
    pub fn next(&self, c: char) -> Position {
        match c {
            '\n' => self.next_line(),
            '\r' => self.reset_column(),
            c if c.is_alphanumeric() => self.next_column(),
            _ => self.clone()
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {} column {}", self.line+1, self.column+1)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line+1, self.column+1)
    }
}
