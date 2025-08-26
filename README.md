#

`packageurl.rs` [![Star me](https://img.shields.io/github/stars/scm-rs/packageurl.rs.svg?style=social&label=Star)](https://github.com/scm-rs/packageurl.rs/stargazers)

*Read and generate Package URLs in Rust.*

[![CI](https://github.com/scm-rs/packageurl.rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/scm-rs/packageurl.rs/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/packageurl.svg)](https://crates.io/crates/packageurl)
[![docs.rs](https://docs.rs/packageurl/badge.svg)](https://docs.rs/packageurl)
[![GitHub issues](https://img.shields.io/github/issues/scm-rs/packageurl.rs.svg)](https://github.com/scm-rs/packageurl.rs/issues)

## About

This crate is an implementation of the [Package URL](https://github.com/package-url/purl-spec)
specification for the [Rust](http://rust-lang.org/) programming language.

## 🔌 Usage

Add this crate to the `Cargo.toml`:

```toml
[dependencies]
packageurl = "0.5.0"
```

Parse a string containing a raw PURL using the
[`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) trait:

```rust
use std::str::FromStr;
use packageurl::PackageUrl;

fn example() {
    let purl = PackageUrl::from_str("pkg:cargo/packageurl@0.3.0");
}
```

Creating a PURL from Rust and then generating a string representation using:
the [`ToString`](https://doc.rust-lang.org/std/string/trait.ToString.html) trait:

```rust
use std::string::ToString;

fn example() {
    let canonical = packageurl::PackageUrl::new("cargo", "packageurl")
        .expect("only fails if type is invalid")
        .with_version("0.3.0")
        .to_string();
}
```

`serde` serialization and deserialization is also supported, provided the
`serde` feature is enabled:

```toml
[dependencies]
packageurl = { version = "0.5.0", features = ["serde"] }
```

See more in the [online documentation](https://docs.rs/packageurl/).

## 📝 Features

- `memchr`: use the [`memchr`](https://docs.rs/memchr/) crate to locate
  separator when parsing.
- `serde`: enable serialization and deserialization support with the
  [`serde`](https://docs.rs/serde) crate.

## 📋 Changelog

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html) and provides
a changelog as part of the [GitHub releases](https://github.com/scm-rs/packageurl.rs/releases).

## 💭 Feedback

Found a bug? Have an enhancement request? Head over to the
[GitHub issue tracker](https://github.com/scm-rs/packageurl.rs/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.

## ⚖️ License

This library is provided under the open-source [MIT license](https://choosealicense.com/licenses/mit/).
