<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
### Fixed
- Make `Builder` public ([#10](https://github.com/EmbarkStudios/tracing-logfmt/pull/10))

## [0.3.1] - 2023-03-13
### Changed
- `Builder::layer` now returns the concrete `tracing_subscriber::fmt::Layer` instead of a `impl Layer`. So that the methods from that type can be accessed.

## [0.3.0] - 2022-11-28
### Added
- Make extra fields configurable ([#8](https://github.com/EmbarkStudios/tracing-logfmt/pull/8)) **This is a breaking change**, as it changes `EventsFormatter` and `FieldsFormatter` from being unit-like structs to regular structs.

### Changed
- Improve performance and reduce allocations ([#5](https://github.com/EmbarkStudios/tracing-logfmt/pull/5))

## [0.2.0] - 2022-09-27
### Added
- Add fields `span` and `span_path`. `span` contains the current/leaf span name, and `span_path` contains all the nested span names. ([#4](https://github.com/EmbarkStudios/tracing-logfmt/pull/4))

## [0.1.2] - 2022-07-11
### Fixed
- Remove unnecessary heap allocation ([#2](https://github.com/EmbarkStudios/tracing-logfmt/pull/2))

## [0.1.1] - 2022-06-15
### Fixed
- Fix dependency status link in README.md ([#1](https://github.com/EmbarkStudios/tracing-logfmt/pull/1))

## [0.1.0] - 2022-06-15
### Added
- Initial implementation of a logfmt formatter for tracing-subscriber

<!-- next-url -->
[Unreleased]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.1...HEAD
[0.3.1]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/EmbarkStudios/tracing-logfmt/releases/tag/0.1.0
