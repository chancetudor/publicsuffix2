#[derive(Clone, Copy)]
/// Parse-time options for loading a Public Suffix List (PSL) into a RuleSet.
///
/// These affect I/O and parsing only; they do not change how lookups behave
/// at runtime (see `MatchOpts` for that).
pub struct LoadOpts {
    /// How to handle PSL section markers (ICANN/PRIVATE) during parsing.
    pub sections: SectionPolicy, // Auto | Ignore | Require
    /// Which kinds of comment lines to accept while parsing.
    pub comments: CommentPolicy, // Common | OfficialOnly
    /// If true, reject malformed rules with an error instead of skipping them.
    pub strict_rules: bool,      // fail on malformed rules
    /// If true, collect non-fatal parser warnings (e.g., duplicated rules).
    pub collect_warnings: bool,  // optional non-fatal notes
}
impl Default for LoadOpts {
    /// Defaults suitable for most applications:
    /// - sections: Auto
    /// - comments: Common
    /// - strict_rules: false (best-effort parsing)
    /// - collect_warnings: false
    fn default() -> Self {
        Self {
            sections: SectionPolicy::Auto,
            comments: CommentPolicy::Common,
            strict_rules: false,
            collect_warnings: false,
        }
    }
}

#[derive(Clone, Copy)]
/// Policy for handling PSL section markers (ICANN / PRIVATE) during parsing.
///
/// This affects only how lists are loaded; it does not impact match behavior.
/// - Auto: Honor section markers when present; tolerate files without markers.
/// - Ignore: Ignore section markers; treat all rules as unclassified.
/// - Require: Require well-formed section markers; error if missing or malformed.
pub enum SectionPolicy {
    Auto,
    Ignore,
    Require,
}
#[derive(Clone, Copy)]
/// Which comment syntaxes are accepted when parsing a PSL file.
/// 
/// Common: Accept both the official `// ...` and commonly-seen `# ...` comments.
/// OfficialOnly: Accept only the official PSL `// ...` comments.
pub enum CommentPolicy {
    Common,
    OfficialOnly,
}

#[derive(Clone, Copy)]
/// Match-time options for splitting a host into prefix/SLL/SLD/TLD.
///
/// These options do not modify the RuleSet; they control how a specific host
/// string is interpreted during lookups. See `Default` for typical settings.
/// The lifetime `'n` ties the borrowed `Normalizer` to this struct.
pub struct MatchOpts<'n> {
    /// Enable PSL wildcard rules (e.g., `*.uk`). When false, only exact-label
    /// rules are considered and wildcard matches are ignored.
    pub wildcard: bool,
    /// Require a rule-derived suffix. If true and no rule matches (or the
    /// ruleset is empty), return `None` instead of falling back to
    /// “last label is the TLD”.
    pub strict: bool,
    /// Which PSL sections are eligible for matching (ICANN, Private, or Any).
    pub types: super::rules::TypeFilter,
    /// Optional borrowed normalizer applied to the input view (zero-copy tweaks
    /// like stripping a trailing dot). For lowercasing or IDNA mapping, preprocess
    /// in an owned buffer before matching and pass that string here.
    pub normalizer: Option<&'n Normalizer>,
}
impl Default for MatchOpts<'_> {
    /// Default implementation for MatchOpts:
    /// - wildcard = true (enable wildcard PSL rules)
    /// - strict = false (allow non-strict fallback when rules are empty)
    /// - types = TypeFilter::Any (accept ICANN and Private sections)
    /// - normalizer = None (no normalization applied)
    fn default() -> Self {
        Self {
            wildcard: true,
            strict: false,
            types: super::rules::TypeFilter::Any,
            normalizer: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct Normalizer {
    pub lowercase: bool,
    pub strip_trailing_dot: bool,
    pub idna_ascii: bool,
}
