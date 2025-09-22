# PublicSuffix2

[![crates.io](https://img.shields.io/crates/v/publicsuffix2.svg)](https://crates.io/crates/publicsuffix2)
[![docs.rs](https://docs.rs/publicsuffix2/badge.svg)](https://docs.rs/publicsuffix2)

A native Rust library for parsing and using Mozilla's [Public Suffix List](https://publicsuffix.org/).

This library is a fork of [rushmorem/publicsuffix](https://github.com/rushmorem/publicsuffix) and aims to reach feature parity with the popular Python library [python-publicsuffix2](https://github.com/aboutcode-org/python-publicsuffix2)

The Public Suffix List is a collection of all TLDs (Top-Level Domains) and other domains under which Internet users can directly register names. This library allows you to determine the "public suffix" part of a domain name.

## Features

*   **Find the Public Suffix (TLD/eTLD):** Extract the effective Top-Level Domain from any hostname.
*   **Find the Second-Level Domain (SLD/eTLD + 1):** Extract the main, registrable part of a domain.
*   **Configurable Normalization:** Control over lowercasing, handling of trailing dots, and IDNA (Punycode) conversion.
*   **ICANN and Private Rules:** Filter matches to include only ICANN-managed TLDs or also include privately-managed domains (e.g., `github.io`).
*   **Wildcard and Exception Rule Support:** Correctly handles complex rules like `*.ck` and `!www.ck`.
*   **IDN and Punycode:** Works seamlessly with both Unicode (e.g., `食狮.中国`) and Punycode (`xn--fiqs8s.xn--fiq228c`) domain names.
*   **High Performance:** Uses a trie data structure for fast lookups.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
publicsuffix2 = "0.1"
```

## Usage

First, you need a copy of the Public Suffix List. You can download it from the [official site](https://publicsuffix.org/list/public_suffix_list.dat).

### Basic Example

```rust
use publicsuffix2::{List, options::MatchOpts};

// Load the PSL data into the library.
// It's recommended to do this once and reuse the `List` object.
const PSL: &str = include_str!("path/to/your/public_suffix_list.dat");

fn main() {
    let list = List::parse(PSL).expect("Failed to parse PSL");

    let domain = "www.example.co.uk";

    // Get the public suffix, also known as the TLD or eTLD.
    let tld = list.tld(domain, MatchOpts::default());
    assert_eq!(tld.as_deref(), Some("co.uk"));

    // Get the second-level domain (the registrable part).
    let sld = list.sld(domain, MatchOpts::default());
    assert_eq!(sld.as_deref(), Some("example.co.uk"));

    // `tld` and `sld` also work on hostnames that are already a public suffix.
    let domain2 = "co.uk";
    let tld2 = list.tld(domain2, MatchOpts::default());
    assert_eq!(tld2.as_deref(), Some("co.uk"));
    let sld2 = list.sld(domain2, MatchOpts::default());
    assert_eq!(sld2.as_deref(), Some("co.uk"));
}
```

### Advanced Options

You can customize matching behavior using `MatchOpts` and `Normalizer`.

```rust
use publicsuffix2::{List, options::{MatchOpts, Normalizer, TypeFilter}};

const PSL: &str = include_str!("path/to/your/public_suffix_list.dat");

fn main() {
    let list = List::parse(PSL).expect("Failed to parse PSL");

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
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.