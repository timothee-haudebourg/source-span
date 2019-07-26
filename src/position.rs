use std::fmt;
use std::cmp::{PartialOrd, Ord, Ordering};

/// Position in a source file.
///
/// This holds the line and column position of a character in a source file.
/// Some operations are available to move position in a file. In partular, the [`next`](Position::next) method
/// computes the next cursor position after reading a given [`char`].
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// Line number, starting at `0`.
    pub line: usize,

    /// Column number, starting at `0`.
    pub column: usize
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Position) -> Ordering {
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            ord => ord
        }
    }
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

    /// Return the maximum position.
    ///
    /// Defined as `(std::usize::MAX, std::usize::MAX)`.
    pub fn end() -> Position {
        Position {
            line: std::usize::MAX,
            column: std::usize::MAX
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
    /// ## Control characters
    ///
    /// This crate is intended to help with incremental lexing/parsing. Therefore, any control
    /// character moving the cursor backward will be ignored: it will be
    /// treated as a 0-width character with no semantics.
    ///
    /// ### New lines
    ///
    /// The `\n` character is interpreted with the Unix semantics, as the new line (NL) character.
    /// It will reset the column position to `0` and move to the next line.
    ///
    /// ### Tabulations
    ///
    /// The `\t` will move the cursor to the next horizontal tab-top.
    /// This function assumes there is a tab-stop every 8 columns.
    /// Note that there is no standard on the size of a tabulation, however a length of 8 columns
    /// seems typical.
    ///
    /// As of today, there is no way to use another tab length.
    ///
    /// I understand that this lacks of flexibility may become an issue in the near future,
    /// and I will try to add this possibility. In the meantime, you are very welcome to contribute
    /// if you need this feature right away.
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
            '\t' => Position {
                line: self.line,
                column: (self.column/8)*8 + 8
            },
            c if c.is_control() => *self,
            _ => self.next_column()
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line == std::usize::MAX || self.column == std::usize::MAX {
            if self.line == std::usize::MAX {
                if self.column == std::usize::MAX {
                    write!(f, "line [end] column [end]")
                } else {
                    write!(f, "line [end] column {}", self.column+1)
                }
            } else {
                write!(f, "line {} column [end]", self.line+1)
            }
        } else {
            write!(f, "line {} column {}", self.line+1, self.column+1)
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line == std::usize::MAX || self.column == std::usize::MAX {
            if self.line == std::usize::MAX {
                if self.column == std::usize::MAX {
                    write!(f, "[end]:[end]")
                } else {
                    write!(f, "[end]:{}", self.column+1)
                }
            } else {
                write!(f, "{}:[end]", self.line+1)
            }
        } else {
            write!(f, "{}:{}", self.line+1, self.column+1)
        }
    }
}
