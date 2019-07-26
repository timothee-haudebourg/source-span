use std::fmt;
use std::fmt::Write;
use std::io::Result;
use std::convert::TryInto;

#[cfg(feature="colors")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
    Magenta,
    Yellow,
    Cyan
}

#[cfg(feature="colors")]
impl termion::color::Color for Color {
    fn write_fg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Color::*;
        match self {
            Red => termion::color::LightRed.write_fg(f),
            Green => termion::color::LightGreen.write_fg(f),
            Blue => termion::color::LightBlue.write_fg(f),
            Magenta => termion::color::LightMagenta.write_fg(f),
            Yellow => termion::color::LightYellow.write_fg(f),
            Cyan => termion::color::LightCyan.write_fg(f)
        }
    }

    fn write_bg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Color::*;
        match self {
            Red => termion::color::LightRed.write_bg(f),
            Green => termion::color::LightGreen.write_bg(f),
            Blue => termion::color::LightBlue.write_bg(f),
            Magenta => termion::color::LightMagenta.write_bg(f),
            Yellow => termion::color::LightYellow.write_bg(f),
            Cyan => termion::color::LightCyan.write_bg(f)
        }
    }
}

#[cfg(not(feature="colors"))]
pub type Color = ();

use crate::Span;

/// Highlight format description.
///
/// Specifies how the highlight should be rendered:
///  * What character to use to draw the line under one-lined highlights.
/// ```txt
/// 1 | fn main() {
/// 2 |     println!("Hello World!")
///   |              ++++++++++++++ highlighting this string
/// 3 | }
/// ```
/// In this example, the line character is `+`.
///
///  * What boundary marker character to use to point the first and last elements of a
///    multi-line highlight.
/// ```txt
/// 1 |   fn main() {
///   |  ___________^
/// 2 | |     println!("Hello World!")
/// 3 | | }
///   | |_^ this span covers more than one line
/// ```
/// In this example, the boundary marker is `^`. The line character is not used.
///
/// ## Colors
///
/// If the `colors` feature is enabled, it is also possible to set a color to draw the lines.
/// This will also make the highlights more bright (or bold), along with the line numbers.
#[derive(Clone, Copy)]
pub enum Style {
    /// Red curvy underline.
    Error,

    /// Yellow curvy underline.
    Warning,

    /// Blue straight underline.
    Note,

    /// Custom highlight format.
    ///
    /// Specifies the line character, the boundary marker and the color (if the `colors` feature
    /// is enabled) used to render the highlight.
    Custom(char, char, Color)
}

impl Style {
    /// Create a new custom highlight style.
    ///
    /// The `line` character is user to draw the line under the span elements.
    /// The `marker` character is used to point to the first and last elements of the span when
    /// relevant.
    #[cfg(not(feature="colors"))]
    pub const fn new(line: char, marker: char) -> Style {
        Style::Custom(line, marker, ())
    }

    /// Create a new custom highlight style.
    ///
    /// The `line` character is user to draw the line under the span elements.
    /// The `marker` character is used to point to the first and last elements of the span when
    /// relevant.
    #[cfg(feature="colors")]
    pub const fn new(line: char, marker: char, color: Color) -> Style {
        Style::Custom(line, marker, color)
    }

    /// The character used to draw the line under the span elements.
    pub fn line(&self) -> char {
        use Style::*;
        match self {
            Error => '^',
            Warning => '^',
            Note => '_',
            Custom(line, _, _) => *line
        }
    }

    /// The character used to point the first and last element of the span when relevant.
    pub fn marker(&self) -> char {
        use Style::*;
        match self {
            Error => '^',
            Warning => '^',
            Note => '^',
            Custom(_, marker, _) => *marker
        }
    }

    /// Get the color used to draw the highlight.
    pub fn color(&self) -> Color {
        #[cfg(not(feature="colors"))]
        {
            ()
        }
        #[cfg(feature="colors")]
        {
            use Style::*;
            match self {
                Error => Color::Red,
                Warning => Color::Yellow,
                Note => Color::Blue,
                Custom(_, _, color) => *color
            }
        }
    }
}

/// Span highlight.
///
/// Used to define what should be highlighted in the text formatted with the [`Formatter`].
/// Given a span a label and a style, the formatter will add an line under the elements of the
/// highlight span, along with the label (if any).
///
/// ```txt
/// 1 | fn main() {
/// 2 |     println!("Hello World!")
///   |              ^^^^^^^^^^^^^^ highlighting this string
/// 3 | }
/// ```
/// # Multiline spans
///
/// The highlight span can cover multiple lines. In that case, only the first and last elements
/// will be underlined (or pointed).
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
/// Different highlights can overlap without breaking the formatted output, but it may become
/// difficult to read the exact boundary of each highlight.
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
/// Here the underline character for the string is the same as the boundary marker for the
/// parenthesis, making it hard to see which is which.
/// One possible workaround is to change the [`Style`] of the highlights. Changing the boundary
/// marker for the parenthesis to `|` makes it easier to read the formatted output:
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
    style: Style
}

/// Text formatter with span highlights.
///
/// This allows you to format a given input `char` stream with highlights and colors (if the
/// `colors` feature is enabled).
/// A [`Highlight`] is defined by a [`Span`], a string label and a [`Style`], and will be rendered
/// with the stream:
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
    show_line_numbers: bool,
    margin_color: Color
}

/// Highlight with some more information about where to draw the vertical line so it does not
/// colide with other highlights, and chere to draw the label.
struct MappedHighlight<'a> {
    h: &'a Highlight,
    column_offset: usize,
    label_position: (usize, usize)
}

/// Character with style information.
#[derive(Clone, Copy)]
pub enum Char {
    Text(char),
    Margin(char, Color),
    Label(char, Color),
    SpanUnderline(char, Color),
    SpanMarker(char, Color),
    SpanLine(Color),
    SpanColumn(ColumnStyle, Color)
}

#[derive(Clone, Copy)]
pub enum ColumnStyle {
    Normal,
    Abbreviated
}

impl Char {
    fn label(c: char, color: Color) -> Char {
        Char::Label(c, color)
    }

    fn space() -> Char {
        Char::Text(' ')
    }

    fn unwrap(self) -> char {
        match self {
            Char::Text(c) => c,
            Char::Margin(c, _) => c,
            Char::Label(c, _) => c,
            Char::SpanUnderline(c, _) => c,
            Char::SpanMarker(c, _) => c,
            Char::SpanLine(_) => '_',
            Char::SpanColumn(ColumnStyle::Normal, _) => '|',
            Char::SpanColumn(ColumnStyle::Abbreviated, _) => '/'
        }
    }

    fn with(&self, c: char) -> Char {
        match self {
            Char::Text(_) => Char::Text(c),
            Char::Margin(_, color) => Char::Margin(c, *color),
            Char::Label(_, color) => Char::Label(c, *color),
            Char::SpanUnderline(_, color) => Char::SpanUnderline(c, *color),
            Char::SpanMarker(_, color) => Char::SpanMarker(c, *color),
            Char::SpanLine(color) => Char::SpanLine(*color),
            Char::SpanColumn(style, color) => Char::SpanColumn(*style, *color)
        }
    }

    fn color(&self) -> Option<Color> {
        match self {
            Char::Text(_) => None,
            Char::Margin(_, color) => Some(*color),
            Char::Label(_, color) => Some(*color),
            Char::SpanUnderline(_, color) => Some(*color),
            Char::SpanMarker(_, color) => Some(*color),
            Char::SpanLine(color) => Some(*color),
            Char::SpanColumn(_, color) => Some(*color)
        }
    }

    fn is_label(&self) -> bool {
        match self {
            Char::Label(_, _) => true,
            _ => false
        }
    }

    fn is_free(&self) -> bool {
        match self {
            Char::Text(' ') => true,
            _ => false
        }
    }

    fn is_span_line(&self) -> bool {
        match self {
            Char::SpanLine(_) => true,
            _ => false
        }
    }

    fn is_span_column(&self) -> bool {
        match self {
            Char::SpanColumn(_, _) => true,
            _ => false
        }
    }

    fn is_margin_column(&self) -> bool {
        match self {
            Char::Margin('|', _) => true,
            _ => false
        }
    }
}

impl From<char> for Char {
    fn from(c: char) -> Char {
        Char::Text(c)
    }
}

/// A single line of [`Char`] in the formatted output.
struct Line {
    data: Vec<Char>,
    offset: usize,
    margin_color: Color
}

impl Line {
    fn new(margin: usize, margin_color: Color) -> Line {
        let mut data = Vec::new();
        data.resize(margin, Char::space());
        Line {
            data: data,
            offset: margin,
            margin_color: margin_color
        }
    }

    fn is_empty(&self) -> bool {
        self.data.len() == self.offset
    }

    fn get(&self, i: usize) -> Char {
        if let Some(c) = self.data.get(i) {
            *c
        } else {
            Char::space()
        }
    }

    fn is_free(&self, i: usize, j: usize) -> bool {
        for k in i..j {
            if !self.get(k).is_free() {
                return false
            }
        }

        true
    }

    fn is_first_char(&self, i: usize) -> bool {
        for k in self.offset..i {
            if let Char::Text(_) = self.get(k) {
                return false
            }
        }

        true
    }

    fn push(&mut self, c: Char) {
        if c.unwrap() == '\t' {
            let len = self.data.len() - self.offset;
            let tab_len = 8 - len%8; // tab length is 8.
            self.data.resize(self.offset + len + tab_len, c.with(' '));
        } else {
            self.data.push(c);
        }
    }

    fn set(&mut self, i: usize, c: Char) {
        if self.data.len() <= i {
            self.data.resize(i+1, Char::space());
        }
        if !self.data[i].is_label() {
            self.data[i] = c;
        }
    }

    fn draw_label(&mut self, label: &String, i: usize, style: Style) {
        for (k, c) in label.chars().enumerate() {
            self.set(i+k, Char::label(c, style.color()))
        }
    }

    fn draw_line_number(&mut self, mut i: usize, margin: usize) {
        let w = margin-3;

        self.set(margin-2, Char::Margin('|', self.margin_color));

        for k in 0..w {
            let codepoint = 0x30 + i as u32 %10;
            i /= 10;
            self.set(w-k-1, Char::Margin(codepoint.try_into().unwrap(), self.margin_color));
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature="colors")]
        let mut current_color = None;

        for c in &self.data {
            #[cfg(feature="colors")]
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
        '\n'.fmt(f)
    }
}

struct LineBuffer {
    index: usize,
    margin: usize,
    lines: Vec<Line>,
    margin_color: Color
}

impl LineBuffer {
    fn new(index: usize, line: Line, margin: usize, margin_color: Color) -> LineBuffer {
        LineBuffer {
            index: index,
            margin: margin,
            lines: vec![line],
            margin_color: margin_color
        }
    }

    fn draw(&mut self, mh: &MappedHighlight) -> Option<usize> {
        if mh.h.span.line_count() > 1 {
            let column = self.margin - mh.column_offset -1;
            if mh.h.span.start().line == self.index {
                self.draw_boundary(column+1, self.margin + mh.h.span.start().column, mh.h.style, true);
                None
            } else if mh.h.span.end().line == self.index {
                let j = self.margin + mh.h.span.last().column;
                self.draw_boundary(column+1, j, mh.h.style, false);
                self.draw_column(column, mh.h.style);
                Some(j)
            } else if mh.h.span.start().line < self.index && mh.h.span.end().line > self.index {
                self.draw_column(column, mh.h.style);
                None
            } else {
                None
            }
        } else {
            if mh.h.span.start().line == self.index {
                let i = self.margin + mh.h.span.start().column;
                let j = self.margin + mh.h.span.last().column;
                self.draw_inline_span(i, j, mh.h.style);
                Some(j)
            } else {
                None
            }
        }
    }

    fn draw_inline_span(&mut self, i: usize, j: usize, style: Style) {
        let index = self.find_free_line(i, j+1, false);

        if index == 1 {
            let line = &mut self.lines[index];
            for k in i..(j+1) {
                line.set(k, Char::SpanUnderline(style.line(), style.color()));
            }
        } else {
            for l in 1..(index+1) {
                let line = &mut self.lines[l];
                if l == 1 {
                    line.set(i, Char::SpanMarker(style.marker(), style.color()));
                    line.set(j, Char::SpanMarker(style.marker(), style.color()));
                } else if l == index {
                    line.set(i, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                    line.set(j, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                    for k in (i+1)..j {
                        line.set(k, Char::SpanLine(style.color()));
                    }
                } else {
                    line.set(i, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                    line.set(j, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                }
            }

        }
    }

    fn draw_column(&mut self, i: usize, style: Style) {
        for line in self.lines.iter_mut() {
            line.set(i, Char::SpanColumn(ColumnStyle::Normal, style.color()));
        }
    }

    fn draw_boundary(&mut self, i: usize, j: usize, style: Style, abbreviate: bool) {
        if abbreviate && self.lines[0].is_first_char(j) {
            self.draw_column(i-1, style);
            let first_line = &mut self.lines[0];
            first_line.set(i-1, Char::SpanColumn(ColumnStyle::Abbreviated, style.color()));
        } else {
            let index = self.find_free_line(i, j+1, false);
            for l in 1..(index+1) {
                let line = &mut self.lines[l];
                if l == 1 {
                    line.set(j, Char::SpanMarker(style.marker(), style.color()));
                } else {
                    line.set(j, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                }
                if l == index {
                    for k in i..j {
                        line.set(k, Char::SpanLine(style.color()));
                    }
                }
            }
        }
    }

    /// Draw label at the given column.
    fn draw_label(&mut self, label: &String, i: usize, style: Style) {
        let j = i+label.len()+1;

        if self.lines[1].is_free(i+1, j+2) {
            self.lines[1].draw_label(label, i+2, style);
        } else {
            let index = self.find_free_line(i, j, true);
            for l in 2..(index+1) {
                let line = &mut self.lines[l];
                if l == index {
                    line.draw_label(label, i, style);
                } else {
                    line.set(i, Char::SpanColumn(ColumnStyle::Normal, style.color()));
                }
            }
        }
    }

    /// Return the index of a secondary line where every character between `i` (included) and
    /// `j` (excluded) is free (a whitespace).
    fn find_free_line(&mut self, i: usize, j: usize, label: bool) -> usize {
        let mut index = 1;
        'next_line: loop {
            if index >= self.lines.len() {
                self.extend();
            }

            if index > 100 {
                return index
            }

            if self.lines[index].is_free(i, j) {
                if label && index > 1 {
                    let last_line = &self.lines[index-1];
                    for k in i..(j+1) {
                        let top = last_line.get(k);
                        if top.is_span_column() || top.is_label() {
                            index += 1;
                            continue 'next_line;
                        }
                    }
                }

                return index
            }

            index += 1;
        }
    }

    fn extend(&mut self) {
        let mut new_line = Line::new(self.margin, self.margin_color);
        {
            let last_line = self.lines.last().unwrap();
            for i in 0..self.margin {
                let top = last_line.get(i);
                let top_right = last_line.get(i+1);

                if top_right.is_span_line() && top.is_free() {
                    new_line.set(i, Char::SpanColumn(ColumnStyle::Normal, top_right.color().unwrap()));
                }
                if (!top_right.is_span_line() && top.is_span_column()) || top.is_margin_column() {
                    new_line.set(i, top);
                }
            }
        }
        self.lines.push(new_line);
    }
}

impl fmt::Display for LineBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            line.fmt(f)?;
        }
        Ok(())
    }
}

impl Highlight {
    fn map(&self, others: &[MappedHighlight]) -> MappedHighlight {
        let mut column_offset = 0;

        if self.span.line_count() > 1 {
            column_offset = 1;
        }

        for h in others.iter() {
            if h.h.span.overlaps(&self.span) {
                if column_offset > 0 && h.h.span.line_count() > 1 {
                    column_offset = std::cmp::max(column_offset, h.column_offset+2);
                }
            }
        }

        MappedHighlight {
            h: self,
            column_offset: column_offset,
            label_position: (0, 0)
        }
    }
}

impl Formatter {
    /// Create a new formatter with no highlights.
    ///
    /// By default line numbers are showing. You can disable them using the
    /// [`hide_line_numbers`](Formatter::hide_line_numbers) method.
    pub fn new() -> Formatter {
        Formatter {
            highlights: Vec::new(),
            show_line_numbers: true,
            margin_color: Color::Blue
        }
    }

    /// Show the line numbers in the output.
    ///
    /// This is the default.
    pub fn show_line_numbers(&mut self) {
        self.show_line_numbers = true;
    }

    /// Hide the line numbers in the output.
    ///
    /// By default, line numbers are visible.
    pub fn hide_line_numbers(&mut self) {
        self.show_line_numbers = false;
    }

    /// Add a span highlight.
    pub fn add(&mut self, span: Span, label: Option<String>, style: Style) {
        self.highlights.push(Highlight {
            span: span,
            label: label,
            style: style
        });
        self.highlights.sort_by(|a, b| a.span.cmp(&b.span));
    }

    /// Produce the formatted output.
    ///
    /// Create a [`String`] containing the content of the input iterator given span, with the
    /// previously added highlights.
    pub fn get<I: Iterator<Item=Result<char>>>(&self, input: I, span: Span) -> std::io::Result<String> {
        let line_number_margin = if self.show_line_numbers {
            (((span.last().line+1) as f32).log10() as usize) + 4
        } else {
            0
        };

        // map every highlights and the left margin.
        let mut highlights = Vec::new();
        let mut highlights_margin = 0;
        for h in &self.highlights {
            let mapped_h = h.map(&highlights);
            highlights_margin = std::cmp::max(highlights_margin, mapped_h.column_offset);
            highlights.push(mapped_h);
        }

        if highlights_margin > 0 {
            highlights_margin += 1;
        }

        let margin = line_number_margin + highlights_margin;

        let mut lines = Vec::new();

        let mut line_buffer = Line::new(margin, self.margin_color);
        let mut line_span: Span = span.start().into();

        if self.show_line_numbers {
            line_buffer.draw_line_number(line_span.start().line+1, line_number_margin);
        }

        for c in input {
            let c = c?;

            if line_span.end() >= span.end() {
                break
            }

            line_span.push(c);

            if c == '\n' {
                let mut new_line_buffer = Line::new(margin, self.margin_color);
                std::mem::swap(&mut new_line_buffer, &mut line_buffer);
                lines.push(LineBuffer::new(line_span.start().line, new_line_buffer, margin, self.margin_color));
                line_span.clear();
                if self.show_line_numbers {
                    line_buffer.draw_line_number(line_span.start().line+1, line_number_margin);
                }
            } else {
                line_buffer.push(Char::from(c));
            }

            if line_span.end() >= span.last() {
                break
            }
        }

        if !line_buffer.is_empty() {
            lines.push(LineBuffer::new(line_span.start().line, line_buffer, margin, self.margin_color));
        }

        for (i, line) in lines.iter_mut().enumerate() {
            for mh in highlights.iter_mut() {
                if let Some(pos) = line.draw(mh) {
                    mh.label_position = (i, pos);
                }
            }
        }

        for mh in &highlights {
            if let Some(label) = &mh.h.label {
                let line = &mut lines[mh.label_position.0];
                line.draw_label(label, mh.label_position.1, mh.h.style);
            }
        }

        let mut buffer = String::new();
        for line in &lines {
            write!(buffer, "{}", line).unwrap();
        }

        Ok(buffer)
    }
}
