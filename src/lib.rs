use std::fmt;

mod position;
pub use position::Position;

/// Span in a source file.
///
/// A span points to a range of caracters between two cursor [`Position`].
///
/// ## Span construction with the `push*` methods
///
/// A span can be directly created using the [`new`](Span::new) method, however in the context of parsing
/// (or lexing) it might be useful to build spans incrementally.
/// The `push*` methods family will help you do that.
///
///   * [`push`](Span::push) will extend the span to include the given character located at the spans `end`.
///   * [`push_column`](Span::push_column) will extend the span to include the next column. Note that this does not
/// necessarily correspond to the next character (if it is a NL, or a full-width character for
/// instance).
///   * [`push_line`](Span::push_line) will extend the span to include the rest of the line. The end of the span will be
/// placed at the begining of the next line.
///
///   * The [`next`](Span::next) method can finally be used to create the span to `[end, end]` (when a token has
/// been read entirely for instance) and start building the next span. The [`clear`](Span::clear) method
/// does the same but *in place*.
///
/// ## Example
/// Here is a basic example computing the span of every word/token in a `char` stream.
/// ```[rust]
/// for c in chars {
///     let c = c?; // report eventual I/O errors.
///     if c.is_whitespace() {
///         // save the current token.
///         if !current.string.is_empty() {
///             tokens.push(current.clone());
///         }
///
///         // reset current token.
///         current.string.clear();
///         current.span.clear(); // the span here is moved to the end of itself.
///     } else {
///         current.string.push(c);
///         current.span.push(c);
///     }
/// }
///
/// if !current.string.is_empty() {
///     tokens.push(current);
/// }
/// ```
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    /// The position of the first character in the span.
    start: Position,

    /// The position of the character directly following the span.
    ///
    /// It is not included in the span.
    end: Position
}

impl Span {
    /// Create a new span from two positions.
    ///
    /// If the `end` position is before the `start` position then the returned span will be
    /// `[start, start]`.
    pub fn new(start: Position, end: Position) -> Span {
        if end < start {
            Span {
                start: start,
                end: start
            }
        } else {
            Span {
                start: start,
                end: end
            }
        }
    }

    /// Return the position of the first character in the span.
    pub fn start(&self) -> Position {
        self.start
    }

    /// Return the position of the character directly following the span.
    ///
    /// It is not included in the span.
    pub fn end(&self) -> Position {
        self.end
    }

    /// Extends the span to include the next column.
    ///
    /// Note that this does not necessarily correspond
    /// to the next character (if it is a NL, or a full-width character for instance).
    /// To do that you can use the [`push`](Span::push) method.
    pub fn push_column(&mut self) {
        self.end = self.end.next_column();
    }

    /// Extends the span to include the rest of the line.
    ///
    /// The end of the span will be placed at the begining of the next line.
    pub fn push_line(&mut self) {
        self.end = self.end.next_line();
    }

    /// Extend the span to include the given character located at the spans `end` position.
    pub fn push(&mut self, c: char) {
        self.end = self.end.next(c);
    }

    /// Return the next span (defined as `[end, end]`).
    pub fn next(&self) -> Span {
        Span {
            start: self.end,
            end: self.end
        }
    }

    /// Set the span to [`next`](Span::next) (`[end, end]`).
    pub fn clear(&mut self) {
        self.start = self.end;
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "from {:?} to {:?}", self.start, self.end)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.start, self.end)
    }
}
