<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.3.7] - 2026-01-30
### Fixed
- [PR#21](https://github.com/EmbarkStudios/tracing-logfmt/pull/21) resolved [#20](https://github.com/EmbarkStudios/tracing-logfmt/issues/20) by returning 0 on unsupported platforms.

## [0.3.6] - 2026-01-28
### Added
- [PR#17](https://github.com/EmbarkStudios/tracing-logfmt/pull/17) added support for [span events](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html#method.with_span_events).
- [PR#18](https://github.com/EmbarkStudios/tracing-logfmt/pull/18) added support for emitting thread names (if the thread was assigned one) and/or thread ids. Note that this current implementation emits the OS assigned thread id, not the [`std::thread::ThreadId`](https://doc.rust-lang.org/std/thread/struct.ThreadId.html), unlike eg. `tokio-tracing`.

### Changed
- [PR#19](https://github.com/EmbarkStudios/tracing-logfmt/pull/19) added the lockfile to the repository, updated the lints to 1.93.0, and updated the deny configuration.

## [0.3.5] - 2024-08-05
### Added
- Add support for disabling ansi color when the feature is enabled ([#16](https://github.com/EmbarkStudios/tracing-logfmt/pull/16))

## [0.3.4] - 2024-03-01
### Added
- Add formatter options for location (file + line number) and module_path ([#13](https://github.com/EmbarkStudios/tracing-logfmt/pull/13))

## [0.3.3] - 2023-08-22
### Added
- Added feature `ansi_logs` that when enabled will colorize the output ([#11](https://github.com/EmbarkStudios/tracing-logfmt/pull/11))
- Reduce `tracing-subscriber` features that are enabled by default ([#12](https://github.com/EmbarkStudios/tracing-logfmt/pull/12))

## [0.3.2] - 2023-04-24
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
[Unreleased]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.7...HEAD
[0.3.7]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.6...0.3.7
[0.3.6]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.5...0.3.6
[0.3.5]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.4...0.3.5
[0.3.4]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.3...0.3.4
[0.3.3]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.2...0.3.3
[0.3.2]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.1...0.3.2
[0.3.1]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/EmbarkStudios/tracing-logfmt/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/EmbarkStudios/tracing-logfmt/releases/tag/0.1.0
