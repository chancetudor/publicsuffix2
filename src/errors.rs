use core::fmt;
extern crate alloc;
#[cfg(feature = "std")]
use std::error::Error as StdError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    EmptyList,
    MissingSections,
    InvalidRule {
        rule: alloc::string::String,
        reason: RuleSyntax,
    },
    NotUtf8,
    #[cfg(feature = "idna")]
    IdnaError(alloc::string::String),
    LabelTooLong {
        label: alloc::string::String,
    },
    RuleDepthExceeded {
        depth: usize,
    },
    #[cfg(feature = "std")]
    Io(std::io::Error),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Warning {
    DuplicateRule { rule: alloc::string::String },
    ShadowedRule { rule: alloc::string::String },
    UnknownMarker { line: alloc::string::String },
    TrailingDotRule { rule: alloc::string::String },
}

#[derive(Debug, Clone, Copy)]
pub enum RuleSyntax {
    Empty,
    HasEmptyLabel,
    StartsOrEndsWithDot,
    ContainsWhitespace,
    ContainsIllegalChar,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
#[cfg(feature = "std")]
impl StdError for Error {}
