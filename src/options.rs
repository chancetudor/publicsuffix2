#[derive(Clone, Copy)]
/// Parse-time options for loading a Public Suffix List (PSL) into a RuleSet.
///
/// These affect I/O and parsing only; they do not change how lookups behave
/// at runtime (see `MatchOpts` for that).
///
/// - `sections`: How to handle PSL section markers (ICANN/PRIVATE) during parsing.
/// - `comments`: Which kinds of comment lines to accept while parsing.
/// - `strict_rules`: If true, reject malformed rules with an error instead of skipping them.
/// - `collect_warnings`: If true, collect non-fatal parser warnings (e.g., duplicated rules).
pub struct LoadOpts {
    /// How to handle PSL section markers (ICANN/PRIVATE) during parsing.
    pub sections: SectionPolicy,
    /// Which kinds of comment lines to accept while parsing.
    pub comments: CommentPolicy,
    /// If true, reject malformed rules with an error instead of skipping them.
    pub strict_rules: bool,
    /// If true, collect non-fatal parser warnings (e.g., duplicated rules).
    pub collect_warnings: bool,
}
impl Default for LoadOpts {
    /// Defaults suitable for most applications:
    /// - `sections`: Auto
    /// - `comments`: Common
    /// - `strict_rules`: false (best-effort parsing)
    /// - `collect_warnings`: false
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
/// - `Auto`: Honor section markers when present; tolerate files without markers.
/// - `Ignore`: Ignore section markers; treat all rules as unclassified.
/// - `Require`: Require well-formed section markers; error if missing or malformed.
pub enum SectionPolicy {
    /// Honor section markers when present; tolerate files without markers.
    Auto,
    /// Ignore section markers; treat all rules as unclassified.
    Ignore,
    /// Require well-formed section markers; error if missing or malformed.
    Require,
}
#[derive(Clone, Copy)]
/// Which comment syntaxes are accepted when parsing a PSL file.
///
/// - `Common`: Accept both the official `// ...` and commonly-seen `# ...` comments.
/// - `OfficialOnly`: Accept only the official PSL `// ...` comments.
pub enum CommentPolicy {
    /// Accept both the official `// ...` and commonly-seen `# ...` or `; ...` comments.
    Common,
    /// Accept only the official PSL `// ...` comments.
    OfficialOnly,
}

#[derive(Clone, Default)]
/// Zero-copy normalization options applied to the input host view.
///
/// Internally, only adjustments that can be expressed as a borrowed slice
/// are applied (e.g., stripping a trailing dot). For lowercasing or IDNA
/// mapping, pre-process the host in an owned buffer and pass that string
/// to the matcher.
/// - `lowercase`: Lowercase ASCII A–Z before matching.
/// - `strip_trailing_dot`: Strip a single trailing dot (root label), if present.
/// - `idna_ascii`: Convert Unicode labels to IDNA ASCII (A-label) form before matching.
pub struct Normalizer {
    /// Lowercase ASCII A–Z before matching.
    pub lowercase: bool,
    /// Strip a single trailing dot (root label), if present.
    pub strip_trailing_dot: bool,
    /// Convert Unicode labels to IDNA ASCII (A-label) form before matching.
    pub idna_ascii: bool,
}

/// Compile-time preset mirroring python-publicsuffix2’s behavior.
pub const PS2_NORMALIZER: Normalizer = Normalizer {
    lowercase: true,
    strip_trailing_dot: true,
    idna_ascii: cfg!(feature = "idna"),
};

/// Explicit “no normalization”.
pub const RAW_NORMALIZER: Normalizer = Normalizer {
    lowercase: false,
    strip_trailing_dot: false,
    idna_ascii: false,
};

impl Normalizer {
    /// A preset that mirrors python-publicsuffix2's behavior.
    pub const fn ps2() -> Self {
        PS2_NORMALIZER
    }
    /// A preset that disables all normalization.
    pub const fn raw() -> Self {
        RAW_NORMALIZER
    }

    /// A preset that only enables lowercasing.
    pub const fn lowercase_only() -> Self {
        Normalizer {
            lowercase: true,
            ..RAW_NORMALIZER
        }
    }
    /// A preset that only enables stripping the trailing dot.
    pub const fn strip_dot_only() -> Self {
        Normalizer {
            strip_trailing_dot: true,
            ..RAW_NORMALIZER
        }
    }
    /// A preset that only enables IDNA ASCII conversion.
    pub const fn idna_only() -> Self {
        Normalizer {
            idna_ascii: true,
            ..RAW_NORMALIZER
        }
    }
}

#[derive(Clone, Copy)]
/// Match-time options for splitting a host into prefix/SLL/SLD/TLD.
///
/// These options do not modify the RuleSet; they control how a specific host
/// string is interpreted during lookups. See `Default` for typical settings.
/// The lifetime `'n` ties the borrowed `Normalizer` to this struct.
///
/// - `wildcard`: Enable PSL wildcard rules (e.g., `*.uk`). When false, only exact-label rules are considered and wildcard matches are ignored.
/// - `strict`: Require a rule-derived suffix. If true and no rule matches (or the ruleset is empty), return `None` instead of falling back to “last label is the TLD”.
/// - `types`: Which PSL sections are eligible for matching (ICANN, Private, or Any).
/// - `normalizer`: Optional borrowed normalizer applied to the input view (zero-copy tweaks like stripping a trailing dot). For lowercasing or IDNA mapping, preprocess in an owned buffer before matching and pass that string here.
pub struct MatchOpts<'n> {
    /// Enable PSL wildcard rules (e.g., `*.uk`).
    pub wildcard: bool,
    /// Require a rule-derived suffix.
    pub strict: bool,
    /// Which PSL sections are eligible for matching (ICANN, Private, or Any).
    pub types: super::rules::TypeFilter,
    /// Optional borrowed normalizer applied to the input view.
    pub normalizer: Option<&'n Normalizer>,
}
impl Default for MatchOpts<'_> {
    /// Default implementation for `MatchOpts`:
    /// - `wildcard` = true (enable wildcard PSL rules)
    /// - `strict` = false (allow non-strict fallback when rules are empty)
    /// - `types` = TypeFilter::Any (accept ICANN and Private sections)
    /// - `normalizer` = ``Some(&PS2_NORMALIZER)`` (use python-publicsuffix2-like normalization)
    fn default() -> Self {
        Self {
            wildcard: true,
            strict: false,
            types: super::rules::TypeFilter::Any,
            normalizer: Some(&PS2_NORMALIZER),
        }
    }
}

impl<'n> MatchOpts<'n> {
    /// Explicit PS2 preset (same as Default).
    pub fn ps2() -> Self {
        Self::default()
    }

    /// Explicitly disable all normalization.
    pub fn raw() -> Self {
        Self {
            normalizer: None,
            ..Self::default()
        }
    }

    /// Use a custom normalizer preset.
    pub fn with_normalizer(n: &'n Normalizer) -> Self {
        Self {
            normalizer: Some(n),
            ..Self::default()
        }
    }
}
