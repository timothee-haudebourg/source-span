#![allow(clippy::needless_doctest_main)]
#![warn(clippy::nursery, clippy::must_use_candidate, clippy::pedantic)]
use std::cmp::{Ord, Ordering, PartialOrd};

mod position;

/// Lazy string buffer that fills up on demand.
pub mod lazy;

/// Source code formatter with span highlights and notes.
///
/// Here are the kind of things you can produce with the [`Formatter`](fmt::Formatter):
/// <pre><font color="#729FCF"><b>01 |     </b></font>pub fn fibonacci(n: i32) -&gt; u64 {
///    <font color="#729FCF"><b>|                     ________        </b></font><font color="#EF2929"><b>^</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>|  </b></font><font color="#EF2929"><b>__________________________</b></font><font color="#729FCF"><b>|</b></font><font color="#EF2929"><b>________|</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|                          </b></font><font color="#729FCF"><b>|</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>|                          </b></font><font color="#729FCF"><b>this is a pair of parenthesis</b></font>
/// <font color="#729FCF"><b>02 | </b></font><font color="#EF2929"><b>|           </b></font>if n &lt; 0 {
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|  __________________^</b></font>
/// <font color="#729FCF"><b>03 | </b></font><font color="#EF2929"><b>| |                 </b></font>panic!(&quot;{} is negative!&quot;, n);
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>^</b></font><font color="#8AE234"><b>&quot;</b></font><font color="#EF2929"><b>^^             </b></font><font color="#8AE234"><b>&quot;   </b></font><font color="#729FCF"><b>^ this is a pair of parenthesis</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>|</b></font><font color="#8AE234"><b>|_</b></font><font color="#EF2929"><b>|</b></font><font color="#8AE234"><b>_____________|   </b></font><font color="#729FCF"><b>|</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>|__</b></font><font color="#EF2929"><b>|</b></font><font color="#729FCF"><b>_____________</b></font><font color="#8AE234"><b>|</b></font><font color="#729FCF"><b>___|</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                          |             </b></font><font color="#8AE234"><b>|</b></font>
/// <font color="#8AE234"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                          this is a pair of braces</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                                        </b></font><font color="#8AE234"><b>|</b></font>
/// <font color="#8AE234"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                                        </b></font><font color="#8AE234"><b>this is a string</b></font>
/// <font color="#729FCF"><b>04 | </b></font><font color="#EF2929"><b>| |         </b></font>} else if n == 0 {
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |_________^                ^</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|  _________|________________|</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |         |</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |         this is a pair of braces</b></font>
/// <font color="#729FCF"><b>05 | </b></font><font color="#EF2929"><b>| |                 </b></font>panic!(&quot;zero is not a right argument to fibonacci()!&quot;);
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>^</b></font><font color="#8AE234"><b>&quot;                                         </b></font><font color="#729FCF"><b>__ </b></font><font color="#8AE234"><b>&quot;</b></font><font color="#729FCF"><b>^ this is a pair of parenthesis</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>|</b></font><font color="#8AE234"><b>|__________________________________________</b></font><font color="#729FCF"><b>|</b></font><font color="#8AE234"><b>_|</b></font><font color="#729FCF"><b>|</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                       </b></font><font color="#729FCF"><b>|___________________________________________|_</b></font><font color="#8AE234"><b>|</b></font><font color="#729FCF"><b>|</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                                                                   </b></font><font color="#729FCF"><b>| </b></font><font color="#8AE234"><b>|</b></font>
/// <font color="#8AE234"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                                                                   </b></font><font color="#729FCF"><b>this is a pair of parenthesis</b></font>
/// <font color="#729FCF"><b>   | </b></font><font color="#EF2929"><b>| |                                                                     </b></font><font color="#8AE234"><b>|</b></font>
/// <font color="#8AE234"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |                                                                     </b></font><font color="#8AE234"><b>this is a string</b></font>
/// <font color="#729FCF"><b>06 | </b></font><font color="#EF2929"><b>| |         </b></font>} else if n == 1 {
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |_________^                ^</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|  _________|________________|</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |         |</b></font>
/// <font color="#EF2929"><b>   </b></font><font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |         this is a pair of braces</b></font>
/// <font color="#729FCF"><b>07 | </b></font><font color="#EF2929"><b>| |                 </b></font>return 1;
/// <font color="#729FCF"><b>08 | </b></font><font color="#EF2929"><b>| |         </b></font>}
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |_________^ this is a pair of braces</b></font>
/// <font color="#729FCF"><b>09 | </b></font><font color="#EF2929"><b>|   </b></font>
/// <font color="#729FCF"><b>10 | </b></font><font color="#EF2929"><b>|           </b></font>let mut sum = 0;
/// <font color="#729FCF"><b>11 | </b></font><font color="#EF2929"><b>|           </b></font>let mut last = 0;
/// <font color="#729FCF"><b>12 | </b></font><font color="#EF2929"><b>|           </b></font>let mut curr = 1;
/// <font color="#729FCF"><b>13 | </b></font><font color="#EF2929"><b>|           </b></font>for _i in 1..n {
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|  ________________________^</b></font>
/// <font color="#729FCF"><b>14 | </b></font><font color="#EF2929"><b>| |                 </b></font>sum = last + curr;
/// <font color="#729FCF"><b>15 | </b></font><font color="#EF2929"><b>| |                 </b></font>last = curr;
/// <font color="#729FCF"><b>16 | </b></font><font color="#EF2929"><b>| |                 </b></font>curr = sum;
/// <font color="#729FCF"><b>17 | </b></font><font color="#EF2929"><b>| |         </b></font>}
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>| |_________^ this is a pair of braces</b></font>
/// <font color="#729FCF"><b>18 | </b></font><font color="#EF2929"><b>|           </b></font>sum
/// <font color="#729FCF"><b>19 | </b></font><font color="#EF2929"><b>|   </b></font>}
///    <font color="#729FCF"><b>| </b></font><font color="#EF2929"><b>|___^ this is a pair of braces</b></font></pre>
pub mod fmt;

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
///
/// Here is a basic example computing the span of every word/token in a `char` stream.
///
/// ```rust
/// use source_span::Span;
///
/// #[derive(Clone, Default)]
/// pub struct Token {
///     string: String,
///     span: Span,
/// }
///
/// let string = "This is an example String.".to_string();
/// let mut tokens = Vec::new();
/// let mut current = Token::default();
///
/// for c in string.chars() {
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.end.cmp(&other.end)
    }
}

impl Span {
    /// Create a new span from three positions.
    ///
    /// If the `end` position or the `last` position is before the `start` position then the
    /// returned span will be `[start, start]`.
    /// If the `last` position is equal to `end` while the span is not empty, it will panic.
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

    /// Return the position of the first character in the span.
    #[must_use]
    pub const fn start(&self) -> Position {
        self.start
    }

    /// Return the last position included in the span.
    #[must_use]
    pub const fn last(&self) -> Position {
        self.last
    }

    /// Return the position of the character directly following the span.
    ///
    /// It is not included in the span.
    #[must_use]
    pub const fn end(&self) -> Position {
        self.end
    }

    /// Checks if the span is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Checks if two span overlaps.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        (self.start <= other.start && self.end > other.start)
            || (other.start <= self.start && other.end > self.start)
    }

    /// The number of lines covered by the span.
    ///
    /// It is at least one, even if the span is empty.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.last.line - self.start.line + 1
    }

    /// Checks if the span includes the given line.
    #[must_use]
    pub fn includes_line(&self, line: usize) -> bool {
        line >= self.start.line && line <= self.end.line
    }

    /// Extends the span to include the next column.
    ///
    /// Note that this does not necessarily correspond
    /// to the next character (if it is a NL, or a full-width character for instance).
    /// To do that you can use the [`push`](Span::push) method.
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

    /// Extend the span to include the given character located at the spans `end` position.
    pub fn push(&mut self, c: char) {
        self.last = self.end;
        self.end = self.end.next(c);
    }

    /// Compute the union of two spans.
    ///
    /// If the two spans do not overlap, all positions in between will be included in the
    /// resulting span.
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
    /// If the two spans do not overlap, then the empty span located at the start of the most
    /// advanced span (maximum of the start of the two spans) is returned.
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
    ///  * `last` points to the last character of a line (column `std::usize::MAX - 1`).
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

impl ::std::fmt::Debug for Span {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.start, self.end)
    }
}
