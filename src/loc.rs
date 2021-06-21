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
use std::convert::TryInto;
use std::fmt;
use crate::Span;

/// Located data.
///
/// This is a simple wrapper around data that can be located in a source file.
/// It is useful to wrap abstract syntax tree nodes.
///
/// It derefs into the inner value.
pub struct Loc<T: ?Sized> {
	span: Span,
	value: T
}

impl<T: ?Sized> Loc<T> {
	/// Associate a span location to some data by wrapping it under `Loc`.
	pub fn new(t: T, span: Span) -> Loc<T> where T: Sized {
		Self {
			span: span,
			value: t
		}
	}

	/// Get the span location of the data.
	pub fn span(&self) -> Span {
		self.span
	}

	/// Maps the inner value using the given function.
	pub fn map<U, F>(self, f: F) -> Loc<U> where F: FnOnce(T) -> U, T: Sized {
		Loc {
			span: self.span,
			value: f(self.value)
		}
	}

	/// Convert the inner value.
	pub fn inner_into<U>(self) -> Loc<U> where T: Into<U> {
		Loc {
			span: self.span,
			value: self.value.into()
		}
	}

	/// Tries to map the inner value using the given function.
	pub fn try_map<U, F, E>(self, f: F) -> Result<Loc<U>, E> where F: FnOnce(T) -> Result<U, E>, T: Sized {
		Ok(Loc {
			span: self.span,
			value: f(self.value)?
		})
	}

	/// Try to convert the inner value.
	pub fn inner_try_into<U>(self) -> Result<Loc<U>, <T as TryInto<U>>::Error> where T: TryInto<U> {
		Ok(Loc {
			span: self.span,
			value: self.value.try_into()?
		})
	}

	/// Unwrap the data.
	pub fn into_inner(self) -> T where T: Sized {
		self.value
	}

	/// Break the wrapper into the value and its location.
	pub fn into_raw_parts(self) -> (T, Span) where T: Sized {
		(self.value, self.span)
	}
}

impl<T> Loc<Option<T>> {
	/// Transforms a `Option<Loc<T>>` into a `Loc<Option<T>>`.
	/// 
	/// If the input is `None` then this function returns `Loc::new(None, span)`.
	pub fn transposed(t: Option<Loc<T>>, span: Span) -> Self {
		match t {
			Some(t) => t.map(|t| Some(t)),
			None => Self::new(None, span)
		}
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