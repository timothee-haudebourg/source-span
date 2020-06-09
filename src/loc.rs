use std::convert::{
	AsRef,
	AsMut
};
use std::ops::{
	Deref,
	DerefMut
};
use std::hash::{
	Hash,
	Hasher
};
use std::cmp::{
	Ord,
	PartialOrd,
	Ordering
};
use std::fmt;
use crate::Span;

/// Located data.
///
/// This is a simple wrapper around data that can be located in a source file.
/// It is useful to wrap abstract syntax tree nodes.
pub struct Loc<T: ?Sized> {
	span: Span,
	value: T
}

impl<T: ?Sized> Loc<T> {
	/// Associate a span location to some data by wrapping it under `Loc`.
	pub fn new(t: T, span: Span) -> Loc<T> where T: Sized {
		Loc {
			span: span,
			value: t
		}
	}

	/// Get the span location of the data.
	pub fn span(&self) -> Span {
		self.span
	}

	/// Unwrap the data.
	pub fn into_inner(self) -> T {
		self.value
	}

	/// Break the wrapper into the value and its location.
	pub fn into_raw_parts(self) -> (T, Span) {
		(self.value, self.span)
	}
}

impl<T: Clone> Clone for Loc<T> {
	fn clone(&self) -> Loc<T> {
		Loc {
			span: self.span,
			value: self.value.clone()
		}
	}
}

impl<T> AsRef<T> for Loc<T> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> AsMut<T> for Loc<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T: fmt::Display> fmt::Display for Loc<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.fmt(f)
	}
}

impl<T: fmt::Debug> fmt::Debug for Loc<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}:{}", self.value, self.span)
	}
}

impl<T: Hash> Hash for Loc<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.span.hash(state);
		self.value.hash(state);
	}
}

impl<T: PartialEq> PartialEq for Loc<T> {
	fn eq(&self, other: &Loc<T>) -> bool {
		self.span == other.span && self.value == other.value
	}
}

impl<T: Eq> Eq for Loc<T> {}

impl<T: PartialOrd> PartialOrd for Loc<T> {
	fn partial_cmp(&self, other: &Loc<T>) -> Option<Ordering> {
		self.value.partial_cmp(&other.value)
	}
}

impl<T: Ord> Ord for Loc<T> {
	fn cmp(&self, other: &Loc<T>) -> Ordering {
		self.value.cmp(&other.value)
	}
}

impl<T> Deref for Loc<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for Loc<T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}
