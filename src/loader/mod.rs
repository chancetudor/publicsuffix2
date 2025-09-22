use crate::rules::{Leaf, RuleSet, Type};
use crate::{
    errors::{Error, Result, RuleSyntax},
    options::{CommentPolicy, LoadOpts, SectionPolicy},
};
#[cfg(feature = "idna")]
use idna;

// Loads a `RuleSet` from a string slice containing the Public Suffix List.
///
/// This function parses the text line by line, handling comments, section markers,
/// and individual rules. It supports various loading options specified via the
/// `LoadOpts` struct.
///
/// # Errors
///
/// This function will return an error if:
/// - The input text is not valid UTF-8.
/// - The list is empty or contains no valid rules.
/// - `LoadOpts::strict_rules` is enabled and an invalid rule is found.
/// - `LoadOpts::sections` is set to `Require` and section markers are missing.
pub fn load(text: &str, opts: LoadOpts) -> Result<RuleSet> {
    if !text.is_char_boundary(text.len()) {
        return Err(Error::NotUtf8);
    }

    let mut rules = RuleSet::default();
    let mut cur_type: Option<Type> = None;
    let mut saw_marker = false;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || is_comment(line, opts.comments) {
            handle_markers(line, &mut cur_type, &mut saw_marker);
            continue;
        }

        let tok = line.split_whitespace().next().unwrap();
        let (neg, raw_rule) = tok
            .strip_prefix('!')
            .map(|r| (true, r))
            .unwrap_or((false, tok));
        let rule = raw_rule.trim_matches('.');
        if rule.is_empty() {
            if opts.strict_rules {
                return Err(Error::InvalidRule {
                    rule: raw_rule.into(),
                    reason: RuleSyntax::Empty,
                });
            } else {
                continue;
            }
        }

        let typ = match opts.sections {
            SectionPolicy::Auto => {
                if saw_marker {
                    cur_type
                } else {
                    None
                }
            }
            SectionPolicy::Ignore => None,
            SectionPolicy::Require => cur_type,
        };
        if matches!(opts.sections, SectionPolicy::Require) && typ.is_none() {
            continue;
        }

        insert(&mut rules, rule, cur_type, neg);
        // If IDNA is enabled and rule contains non-ASCII, also add an ASCII (A-label) duplicate.
        #[cfg(feature = "idna")]
        if rule.bytes().any(|b| b >= 0x80) {
            if let Ok(ascii) = idna::Config::default().to_ascii(rule) {
                if ascii.as_str() != rule {
                    insert(&mut rules, &ascii, typ, neg);
                }
            }
        }
    }

    if matches!(opts.sections, SectionPolicy::Require) && !saw_marker {
        return Err(Error::MissingSections);
    }
    if rules.root.kids.is_empty() {
        return Err(Error::EmptyList);
    }
    Ok(rules)
}

fn is_comment(s: &str, policy: CommentPolicy) -> bool {
    match policy {
        CommentPolicy::Common => s.starts_with("//") || s.starts_with('#') || s.starts_with(';'),
        CommentPolicy::OfficialOnly => s.starts_with("//"),
    }
}

fn handle_markers(line: &str, cur: &mut Option<Type>, saw: &mut bool) {
    if !line.starts_with("//") {
        return;
    }
    if line.contains("BEGIN ICANN DOMAINS") {
        *cur = Some(Type::Icann);
        *saw = true;
    }
    if line.contains("END ICANN DOMAINS") {
        *cur = None;
    }
    if line.contains("BEGIN PRIVATE DOMAINS") {
        *cur = Some(Type::Private);
        *saw = true;
    }
    if line.contains("END PRIVATE DOMAINS") {
        *cur = None;
    }
}

fn insert(rules: &mut RuleSet, rule: &str, typ: Option<Type>, neg: bool) {
    let mut cur = &mut rules.root;
    for lbl in rule.rsplit('.') {
        cur = cur.kids.entry(lbl.to_string()).or_default();
    }
    cur.leaf = if neg { Leaf::Negative } else { Leaf::Positive };
    cur.typ = typ;
}
