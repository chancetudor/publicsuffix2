pub mod errors;
pub mod options;

mod engine;
mod loader;
mod rules;

pub use engine::Parts;
pub use errors::{Error, Result, Warning};
pub use options::{CommentPolicy, LoadOpts, MatchOpts, Normalizer, SectionPolicy};
pub use rules::{Type, TypeFilter};
use std::borrow::Cow;

#[derive(Clone)]
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
}
