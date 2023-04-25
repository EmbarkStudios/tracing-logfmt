<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `🪵️ tracing-logfmt`

**Logfmt formatter for tracing-subscriber**

[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)
[![Embark](https://img.shields.io/badge/discord-embark-%237289da.svg?logo=discord)](https://discord.gg/dAuKfZS)
[![Crates.io](https://img.shields.io/crates/v/tracing-logfmt.svg)](https://crates.io/crates/tracing-logfmt)
[![Docs](https://docs.rs/tracing-logfmt/badge.svg)](https://docs.rs/tracing-logfmt)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/tracing-logfmt/status.svg)](https://deps.rs/repo/github/EmbarkStudios/tracing-logfmt)
[![Build status](https://github.com/EmbarkStudios/tracing-logfmt/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/tracing-logfmt/actions)
</div>

## Logfmt

Logfmt is a compact and simple log format for structured logging. Each log row contains one level of key/value pairs. To keep it as compact and readable as possible, values are only quoted if needed.

```logfmt
key=value otherkey="value with spaces" third="with escaped \"chars\""
```

There is no strict standard for the format, but it was first documented in [this article](https://brandur.org/logfmt) by Brandur Leach.

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4-ff69b4.svg)](CODE_OF_CONDUCT.md)

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.
Please also read our [Contributor Terms](CONTRIBUTING.md#contributor-terms) before you make any contributions.

Any contribution intentionally submitted for inclusion in an Embark Studios project, shall comply with the Rust standard licensing model (MIT OR Apache 2.0) and therefore be dual licensed as described below, without any additional terms or conditions:

### License

This contribution is dual licensed under EITHER OF

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

For clarity, "your" refers to Embark or any other licensee/user of the contribution.
