pub mod errors;
pub mod options;

mod engine;
#[cfg(feature = "fetch")]
mod http;
mod loader;
mod rules;

pub use engine::Parts;
pub use errors::{Error, Result, Warning};
use once_cell::sync::Lazy;
pub use options::{CommentPolicy, LoadOpts, MatchOpts, Normalizer, SectionPolicy};
pub use rules::{Type, TypeFilter};
#[cfg(feature = "std")]
use std::path::Path;
use std::{borrow::Cow, str::FromStr};

static GLOBAL_LIST: Lazy<List> = Lazy::new(|| {
    let text = include_str!("../tests/fixtures/public_suffix_list.dat");
    List::from_file(text).expect("parsing the embedded public suffix list should not fail")
});

#[derive(Clone, Debug)]
/// A compiled Public Suffix List (PSL) and matcher.
///
/// This type owns the parsed rule tree and provides PS2-compatible queries:
/// - tld: public suffix (PSL match)
/// - sld: registrable domain (eTLD+1)
/// - split: prefix / SLL / SLD / TLD
///
/// Cloning `List` is cheap (the underlying rules are shared).
pub struct List {
    rules: rules::RuleSet,
}

impl FromStr for List {
    type Err = Error;
    /// Parses a string slice into a `List`.
    ///
    /// This is an implementation of the `FromStr` trait, allowing you to use
    /// the `.parse()` method on strings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use publicsuffix2::List;
    ///
    /// let psl_data = "com\nuk\nco.uk";
    /// let list: List = psl_data.parse().expect("Failed to parse list");
    /// ```
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl List {
    /// Parse a PSL text into a `List` using `LoadOpts::default()`.
    ///
    /// Use [`parse_with`] to customize parsing (sections, comments, etc).
    pub fn parse(text: &str) -> Result<Self> {
        Self::parse_with(text, LoadOpts::default())
    }

    /// Parse a PSL text into a `List` using explicit `LoadOpts`.
    ///
    /// Load options affect only parsing (e.g., handling of ICANN/PRIVATE
    /// sections and comment styles), not match-time behavior.
    pub fn parse_with(text: &str, opts: LoadOpts) -> Result<Self> {
        loader::load(text, opts).map(|rules| Self { rules })
    }

    /// Parse a PSL from a file path using `LoadOpts::default()`.
    ///
    /// This method is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with(path, LoadOpts::default())
    }

    /// Parse a PSL from a file path using explicit `LoadOpts`.
    ///
    /// This method is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    pub fn from_file_with<P: AsRef<Path>>(path: P, opts: LoadOpts) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(Error::Io)?;
        Self::parse_with(&text, opts)
    }

    /// Parse a PSL from a URL using `LoadOpts::default()`.
    ///
    /// This method is only available when the `fetch` feature is enabled.
    #[cfg(feature = "fetch")]
    pub fn from_url(url: &str) -> Result<Self> {
        Self::from_url_with(url, LoadOpts::default())
    }

    /// Parse a PSL from a URL using explicit `LoadOpts`.
    ///
    /// This method is only available when the `fetch` feature is enabled.
    #[cfg(feature = "fetch")]
    pub fn from_url_with(url: &str, opts: LoadOpts) -> Result<Self> {
        let text = http::get(url)?;
        Self::parse_with(&text, opts)
    }

    /// Registrable domain (eTLD+1) under PS2 semantics.
    ///
    /// Behavior is controlled by `MatchOpts` (wildcards, strict mode, type
    /// filter, normalization). Returns `None` if:
    /// - input is empty/invalid, or
    /// - `strict` is true and no rule matches.
    ///
    /// Without rules (and non-strict), the fallback treats the last label as
    /// the TLD, making the registrable domain the entire host.
    pub fn sld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Cow<'a, str>> {
        self.rules.sld(host, opts)
    }

    /// Public suffix (PSL match) under PS2 semantics.
    ///
    /// Honors `MatchOpts` (wildcards, strict mode, type filter, normalization).
    /// Returns `None` only when input is empty/invalid or `strict` is true and
    /// no rule matches. With no rules (and non-strict), the suffix is the last
    /// label of the host.
    pub fn tld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Cow<'a, str>> {
        self.rules.tld(host, opts)
    }

    /// Split a host into prefix / SLL / SLD / TLD (PS2-compatible).
    ///
    /// Definitions:
    /// - TLD: the public suffix (PSL match)
    /// - SLD: registrable domain (eTLD+1)
    /// - SLL: the single label immediately left of the TLD
    /// - Prefix: everything left of the SLD (may be `None`)
    ///
    /// Examples (default options):
    /// - "foo.bar.uk" → TLD="bar.uk", SLD="foo.bar.uk", SLL="foo", Prefix=None
    /// - "foo.city.uk" (exception) → TLD="uk", SLD="city.uk", SLL="city", Prefix=Some("foo")
    pub fn split<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<engine::Parts<'a>> {
        self.rules.split(host, opts)
    }

    /// Returns a reference to a globally shared `List` instance.
    ///
    /// The list is parsed from a built-in copy of the Public Suffix List
    /// on the first call and cached for subsequent uses.
    ///
    /// This is the easiest way to get started if you don't need a custom
    /// list or special loading options.
    pub fn global() -> &'static Self {
        &GLOBAL_LIST
    }
}
