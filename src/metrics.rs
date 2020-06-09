pub static DEFAULT_METRICS: DefaultMetrics = DefaultMetrics::new();

pub trait Metrics {
	fn char_width(&self, c: char) -> usize;

	fn tab_stop(&self) -> usize;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DefaultMetrics {
	tab_stop: usize,
}

impl DefaultMetrics {
	pub const fn new() -> DefaultMetrics { Self::with_tab_stop(8) }

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
