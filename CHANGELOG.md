# Changelog

All notable changes to this project will be documented in this file.

## [2.7.0] - 2021-09-27

### Added
- `Position::from`, `Position::from_included` and `Position::to`.

## [2.6.0] - 2021-06-21

### Added
- `Loc::map`, `Loc::try_map`, `Loc::transpose`.

## [2.5.1] - 2021-05-16

### Changed
- Default styles `Note` and `Help` are now defined using the single character `-` which is closer to the intended Rust-like style.

## [2.5.0] - 2021-05-16

### Added
- `Formatter::span` returning the smallest span including all the formatted highlights.

## [2.4.0] - 2021-05-05

### Added
- `Formatter::margin_len` returning the size of the margin used to render the line numbers.

## [2.3.0] - 2021-04-28

### Changed
- Fix compilation failure without `colors` feature.
- Change many functions into `const fn`.
- Do not use `clippy::pedantic`.

## [2.2.0] - 2020-06-20

### Added
- New `Layout` type to index UTF8-string by cursor positions.

### Changed
- Moved `SourceBuffer` into a `lazy` submodule of `buffer`.
  This should not change anything for the user.

## [2.1.0] - 2020-06-13

### Added
- New functions `inner_into` and `inner_try_into` converting inner values in `Loc`.

## [2.0.1] - 2020-06-13

### Added
- Reference to the `README.md` in `Cargo.toml`

## [2.0.0] - 2020-06-13

### Added
- A `Log` type to wrap values that can be located in a source file (with a span).
- A `Metrics` trait to control the width of each character and the tab stop length.
The functions `Position::next` and `Span::push` now take a metrics as parameter.
- The `DefaultMetrics` type and `DEFAULT_METRICS` constant give a default implementation of the `Metrics` trait ready to use.
- A new `formatting_challenge` example has been added.

### Changed
- The `lazy` module have been removed and its content moved to the crate's root.
- `lazy::Buffer` is now `SourceBuffer`.
- The error type of `SourceBuffer` is now a parameter of the type.
It does not have to be `std::io::Error` anymore.
- Access functions `index_at`, `at` and `get` in `SourceBuffer` not returns
`Result<Option<_>, _>` instead of `Option<Result<_, _>>`.
- `Position::shift` to move a position in place.
- `fmt::Formatter::get` is now `fmt::Formatter::render`.
- The output of `fmt::Formatter::render` is not a string but a `Rendered` instance that implements `Display`.
- The whole formatting internal code has been improved:
	- Multi-line labels are now handled.
	- Lines can intertwine without causing non-termination.
- Examples have been modified to comply to all the changes.
