use core::fmt;
extern crate alloc;
#[cfg(feature = "std")]
use std::error::Error as StdError;

/// A specialized `Result` type for this crate's operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
/// The error type for operations that can fail.
pub enum Error {
    /// The Public Suffix List is empty or contains no valid rules.
    EmptyList,
    /// The list is missing the required `BEGIN ICANN DOMAINS` or `BEGIN PRIVATE DOMAINS` markers.
    MissingSections,
    /// A rule in the Public Suffix List has invalid syntax.
    InvalidRule {
        /// The invalid rule.
        rule: alloc::string::String,
        /// The reason why the rule is invalid.
        reason: RuleSyntax,
    },
    /// The input data is not valid UTF-8.
    NotUtf8,
    /// An error occurred during IDNA processing.
    #[cfg(feature = "idna")]
    IdnaError(alloc::string::String),
    /// An error occurred when making an HTTP request
    #[cfg(feature = "fetch")]
    Fetch(Box<dyn StdError + Send + Sync + 'static>),
    /// A label in a domain name is longer than the 63-character limit.
    LabelTooLong {
        /// The label that is too long.
        label: alloc::string::String,
    },
    /// A rule in the Public Suffix List exceeds the maximum allowed depth.
    RuleDepthExceeded {
        /// The depth of the rule.
        depth: usize,
    },
    /// An I/O error occurred while reading the Public Suffix List.
    #[cfg(feature = "std")]
    Io(std::io::Error),
}

/// Represents non-fatal issues encountered while parsing the Public Suffix List.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Warning {
    /// A rule was found more than once in the list.
    DuplicateRule {
        /// The duplicated rule.
        rule: alloc::string::String,
    },
    /// A rule is shadowed by another more general rule (e.g., `b.com` is shadowed by `com`).
    ShadowedRule {
        /// The shadowed rule.
        rule: alloc::string::String,
    },
    /// A line was encountered that looked like a section marker but was not recognized.
    UnknownMarker {
        /// The content of the unrecognized marker line.
        line: alloc::string::String,
    },
    /// A rule contained a trailing dot, which was stripped.
    TrailingDotRule {
        /// The rule with the trailing dot.
        rule: alloc::string::String,
    },
}

/// Describes the reason for a rule syntax error.
#[derive(Debug, Clone, Copy)]
pub enum RuleSyntax {
    /// The rule was empty.
    Empty,
    /// The rule contained an empty label (e.g., `a..b`).
    HasEmptyLabel,
    /// The rule started or ended with a dot.
    StartsOrEndsWithDot,
    /// The rule contained whitespace.
    ContainsWhitespace,
    /// The rule contained an illegal character.
    ContainsIllegalChar,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
#[cfg(feature = "std")]
impl StdError for Error {}
