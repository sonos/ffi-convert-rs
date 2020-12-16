# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
 - `as_rust_extra_field` attribute for `AsRust` custom derive, can be used to specify initialize fields in the rust struct that are not present in the C one
 - `c_repr_of_convert` attribute for `CReprOf` custom derive, can be used to perform custom conversion. Note that fields with this attribute are ignored by the `AsRust` custom derive.
 - `target_name` attribute for `AsRust` and `CReprOf` custom derive, can be used if a field has different names in rust and C
 
### Changed
 - `RawPointerConverter` no longer has a blanket implementation for all types (this could to easily be misused leading to nasty errors), the trait as been reworked to include mut variants, and a custom derive has been added for it

### Fixed
 - Missing `Drop` implementation for CStringArray causing memory leaks

### Removed
 - Removed the `point_to_string` legacy function (this should have been marked `unsafe` and has no place in this crate)


## [0.3.0] - 2017-10-05
### Added
 - `CReprOf`, `CDrop` and `AsRust` implementations for `bool`
### Fixed
 - Typos in doc
### Changed
 - Error management is now using `thiserror` instead of `failure`
### Removed
 - Legacy conversion macros, use directly the traits instead
 
## [0.2.2] - 2017-06-15
### Added
 - `Debug` impl for `CArray<T>`

## [0.2.1] - 2017-04-08
### Fixed
 - `CDrop` custom derive now honors the `#[nullable]` field attribute

## [0.2.0] - 2017-03-32
### Added
 - `CRange` a struct representing a `Range` with implementations for `CReprOf`, `CDrop` and `AsRust`

## [0.1.2] - 2017-03-23
### Fixed
 - use fully qualified names in macros

## [0.1.1] - 2017-03-19
### Fixed
 - double free on pointer fields

## [0.1.0] - 2017-03-17
### Added
 - first release

[Unreleased]: https://github.com/sonos/ffi-convert-rs/compare/0.3.0...HEAD
[0.3.0]: https://github.com/sonos/ffi-convert-rs/compare/0.2.2...0.3.0
[0.2.2]: https://github.com/sonos/ffi-convert-rs/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/sonos/ffi-convert-rs/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/sonos/ffi-convert-rs/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/sonos/ffi-convert-rs/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/sonos/ffi-convert-rs/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/sonos/ffi-convert-rs/releases/tag/0.1.0