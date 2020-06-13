use source_span::{
	fmt::{Color, Formatter, Style},
	Position, Span,
};

#[derive(Clone, Default)]
pub struct Token {
	string: String,
	span: Span,
}

#[derive(PartialEq)]
pub enum Kind {
	Space,
	Separator,
	Alphabetic,
	Numeric,
}

const SOURCE: &'static str = "################################
################################
################################
################################
################################
################################
################################
################################
################################
################################";

fn safe_char(c: char) -> Result<char, ()> {
	Ok(c)
}

fn main() {
	let metrics = source_span::DEFAULT_METRICS;
	let span = Span::of_string(SOURCE, &metrics);
	let mut fmt = Formatter::with_margin_color(Color::Blue);

	fmt.add(span, Some("label Z".to_string()), Style::Error);

	fmt.add(
		Span::new(Position::new(1, 8), Position::new(7, 8), Position::new(7, 9)),
		Some("label A\nwith\nmany\nlines\n...".to_string()),
		Style::Note
	);

	fmt.add(
		Span::new(Position::new(1, 4), Position::new(7, 4), Position::new(7, 5)),
		Some("label B".to_string()),
		Style::Warning
	);

	fmt.add(
		Span::new(Position::new(3, 0), Position::new(5, 4), Position::new(5, 5)),
		Some("label C".to_string()),
		Style::Help
	);

	fmt.add(
		Span::new(Position::new(3, 0), Position::new(5, 4), Position::new(5, 5)),
		Some("label D".to_string()),
		Style::Error
	);

	fmt.add(
		Span::new(Position::new(3, 5), Position::new(3, 12), Position::new(3, 13)),
		Some("label E".to_string()),
		Style::Note
	);

	fmt.add(
		Span::new(Position::new(3, 25), Position::new(6, 12), Position::new(6, 13)),
		Some("label F".to_string()),
		Style::Warning
	);

	fmt.add(
		Span::new(Position::new(8, 0), Position::new(8, 31), Position::new(9, 0)),
		Some("label G".to_string()),
		Style::Warning
	);

	let formatted = fmt.render(SOURCE.chars().map(safe_char), span, &metrics).unwrap();
	println!("{}", formatted);
}
