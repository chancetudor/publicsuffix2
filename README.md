# PublicSuffix2

[![crates.io](https://img.shields.io/crates/v/publicsuffix2.svg?logo=rust)](https://crates.io/crates/publicsuffix2)
![docs.rs](https://img.shields.io/docsrs/publicsuffix2?logo=docsdotrs)
[![CI](https://github.com/chancetudor/publicsuffix2/actions/workflows/ci.yml/badge.svg)](https://github.com/chancetudor/publicsuffix2/actions/workflows/ci.yml)
[![codecov](https://codecov.io/github/chancetudor/publicsuffix2/graph/badge.svg?token=9NPNU81LVZ)](https://codecov.io/github/chancetudor/publicsuffix2)
[![License](https://img.shields.io/crates/l/publicsuffix2.svg)](https://github.com/chancetudor/publicsuffix2/blob/main/LICENSE)

A native Rust library for parsing and using Mozilla's [Public Suffix List](https://publicsuffix.org/).

This library is a fork of [rushmorem/publicsuffix](https://github.com/rushmorem/publicsuffix) and aims to reach feature parity with the popular Python library [python-publicsuffix2](https://github.com/aboutcode-org/python-publicsuffix2)

The Public Suffix List is a collection of all TLDs (Top-Level Domains) and other domains under which Internet users can directly register names. This library allows you to determine the "public suffix" part of a domain name.

## Features

* **Find the Public Suffix (TLD/eTLD):** Extract the effective Top-Level Domain from any hostname.
* **Find the Second-Level Domain (SLD/eTLD + 1):** Extract the main, registrable part of a domain.
* **Configurable Normalization:** Control over lowercasing, handling of trailing dots, and IDNA (Punycode) conversion.
* **ICANN and Private Rules:** Filter matches to include only ICANN-managed TLDs or also include privately-managed domains (e.g., `github.io`).
* **Wildcard and Exception Rule Support:** Correctly handles complex rules like `*.ck` and `!www.ck`.
* **IDN and Punycode:** Works seamlessly with both Unicode (e.g., `食狮.中国`) and Punycode (`xn--fiqs8s.xn--fiq228c`) domain names.
* **High Performance:** Uses a trie data structure for fast lookups.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
publicsuffix2 = "0.5.2"
```

To fetch the list from a URL, enable the `fetch` feature:

```toml
[dependencies]
publicsuffix2 = { version = "0.5.2", features = ["fetch"] }
```

## Usage

### Getting Started

The easiest way to use the library is to create a `List` using `List::default()`. This uses a built-in copy of the Public Suffix List, so no file loading is required.

```rust
use publicsuffix2::{List, MatchOpts};

// Create a list from the built-in PSL data.
// It's recommended to do this once and reuse the `List` object.
let list = List::default();

let domain = "www.example.co.uk";

// Get the public suffix, also known as the TLD or eTLD.
let tld = list.tld(domain, MatchOpts::default());
assert_eq!(tld.as_deref(), Some("co.uk"));

// Get the registrable domain (the part you can register).
let sld = list.sld(domain, MatchOpts::default());
assert_eq!(sld.as_deref(), Some("example.co.uk"));

// `tld` and `sld` also work on hostnames that are already a public suffix.
let domain2 = "co.uk";
let tld2 = list.tld(domain2, MatchOpts::default());
assert_eq!(tld2.as_deref(), Some("co.uk"));
let sld2 = list.sld(domain2, MatchOpts::default());
assert_eq!(sld2.as_deref(), Some("co.uk"));
```

### Splitting a Domain

The `split` method deconstructs a domain into all its parts.

```rust
use publicsuffix2::{List, MatchOpts};

let list = List::default();
let parts = list.split("sub.www.example.co.uk", MatchOpts::default()).unwrap();

assert_eq!(parts.tld(), Some("co.uk"));
assert_eq!(parts.sld(), Some("example.co.uk"));
assert_eq!(parts.sll(), Some("example"));
assert_eq!(parts.prefix(), Some("sub.www"));
```

### Loading a Custom List

If you need to use a custom or updated Public Suffix List, you can create a `List` instance from a string, file, or URL.

```rust
use publicsuffix2::{List, Result};

fn main() -> Result<()> {
    // From a string using the FromStr trait
    const PSL: &str = "com\nuk\nco.uk\n";
    let list_from_str: List = PSL.parse()?;

    // From a file path (requires the `std` feature)
    let list_from_file = List::from_file("path/to/your/public_suffix_list.dat")?;

    // From a URL (requires the `fetch` feature)
    #[cfg(feature = "fetch")]
    let list_from_url = List::from_url("https://publicsuffix.org/list/public_suffix_list.dat")?;

    Ok(())
}
```

### Advanced Options

You can customize matching behavior using `MatchOpts` and `Normalizer`.

```rust
use publicsuffix2::{List, options::{MatchOpts, Normalizer, TypeFilter}};

let list = List::default();

// --- Example 1: Handling trailing dots ---
let norm_strip_dot = Normalizer {
    strip_trailing_dot: true,
    ..Default::default()
};
let opts_strip_dot = MatchOpts {
    normalizer: Some(&norm_strip_dot),
    ..Default::default()
};
let sld1 = list.sld("foo.com.", opts_strip_dot);
assert_eq!(sld1.as_deref(), Some("foo.com"));


// --- Example 2: Filtering for ICANN rules only ---
// By default, private domains like `blogspot.com` are treated as TLDs.
let sld_default = list.sld("my-blog.blogspot.com", MatchOpts::default());
assert_eq!(sld_default.as_deref(), Some("my-blog.blogspot.com"));

// You can filter to only use ICANN section rules.
let opts_icann_only = MatchOpts {
    type_filter: TypeFilter::Icann,
    ..Default::default()
};
let sld_icann = list.sld("my-blog.blogspot.com", opts_icann_only);
assert_eq!(sld_icann.as_deref(), Some("blogspot.com"));


// --- Example 3: Handling Internationalized Domain Names (IDN) ---
// The default normalizer converts to ASCII Punycode.
let sld_punycode = list.sld("食狮.中国", MatchOpts::default());
assert_eq!(sld_punycode.as_deref(), Some("xn--85x722f.xn--fiqs8s"));

// You can disable IDNA conversion to keep Unicode characters.
let norm_no_idna = Normalizer {
    idna_ascii: false,
    ..Default::default()
};
let opts_no_idna = MatchOpts {
    normalizer: Some(&norm_no_idna),
    ..Default::default()
};
let sld_unicode = list.sld("食狮.中国", opts_no_idna);
assert_eq!(sld_unicode.as_deref(), Some("食狮.中国"));
```

## License

This project is licensed under the MIT License and Apache License. See the [LICENSE](LICENSE) or [LICENSE-APACHE](LICENSE-APACHE) file for details.
