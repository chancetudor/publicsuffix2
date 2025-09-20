#[derive(Clone, Copy)]
pub struct LoadOpts {
    pub sections: SectionPolicy, // Auto | Ignore | Require
    pub comments: CommentPolicy, // Common | OfficialOnly
    pub strict_rules: bool,      // fail on malformed rules
    pub collect_warnings: bool,  // optional non-fatal notes
}
impl Default for LoadOpts {
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
pub enum SectionPolicy {
    Auto,
    Ignore,
    Require,
}
#[derive(Clone, Copy)]
pub enum CommentPolicy {
    Common,
    OfficialOnly,
}

#[derive(Clone, Copy)]
pub struct MatchOpts<'n> {
    pub wildcard: bool,
    pub strict: bool,
    pub types: super::rules::TypeFilter,
    pub normalizer: Option<&'n Normalizer>,
}
impl Default for MatchOpts<'_> {
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
