/// Default character metrics.
pub static DEFAULT_METRICS: DefaultMetrics = DefaultMetrics::new();

/// Gives the size of each character and tab stop length.
///
/// This is used to correctly compute spans in the source text,
/// and render the text using a [`Formatter`](`crate::fmt::Formatter`).
pub trait Metrics {
	/// Get the size (width in columns) of a character.
	fn char_width(&self, c: char) -> usize;

	/// Get the tab stop length.
	fn tab_stop(&self) -> usize;
}

/// Default metrics infos.
///
/// By default, tab stop length is 8, but it can be set using [`DefaultMetrics::with_tab_stop`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DefaultMetrics {
	tab_stop: usize,
}

impl DefaultMetrics {
	/// Create a new default metrics instance.
	///
	/// Tab stop length will be 8.
	#[must_use]
	pub const fn new() -> DefaultMetrics { Self::with_tab_stop(8) }

	/// Create a new default metrics with a custom tab stop length.
	#[must_use]
	pub const fn with_tab_stop(tab_stop: usize) -> DefaultMetrics { DefaultMetrics { tab_stop } }
}

impl Metrics for DefaultMetrics {
	fn char_width(&self, c: char) -> usize {
		match c {
			'\r' | '\n' => 0,
			_ => 1,
		}
	}

	fn tab_stop(&self) -> usize { self.tab_stop }
}
