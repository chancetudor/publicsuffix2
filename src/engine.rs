use crate::options::MatchOpts;
use crate::rules::{Leaf, Node, RuleSet, TypeFilter};

#[derive(Debug, PartialEq, Eq)]
pub struct Parts<'a> {
    pub prefix: Option<&'a str>, // everything left of sld
    pub sll: Option<&'a str>,    // "second-level label" (just the label)
    pub sld: Option<&'a str>,    // registrable domain (eTLD+1)
    pub tld: &'a str,            // public suffix (PSL match)
}

impl RuleSet {
    /// Core: PS2-style split into prefix/sll/sld/tld.
    pub fn split<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Parts<'a>> {
        let s = normalize_view(host, opts)?;
        let (_, tld) = self.match_tld(s, opts)?;

        // sld_end = len(host) - len(tld) - 1 (position of dot left of tld)
        let sld_end = s.len().saturating_sub(tld.len()).saturating_sub(1);
        if sld_end == 0 {
            // no label left of tld → no sll and no sld
            return Some(Parts {
                prefix: None,
                sll: None,
                sld: None,
                tld,
            });
        }

        let idx = s[..sld_end].rfind('.');
        let prefix = idx.filter(|&i| i > 0).map(|i| &s[..i]);
        let sll = Some(&s[idx.map(|i| i + 1).unwrap_or(0)..sld_end]).filter(|v| !v.is_empty());
        let sld = Some(&s[idx.map(|i| i + 1).unwrap_or(0)..]); // from sll start through tld

        Some(Parts {
            prefix,
            sll,
            sld,
            tld,
        })
    }

    /// Core: PS2-style registrable domain (eTLD+1).
    pub fn sld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<&'a str> {
        self.split(host, opts).and_then(|p| p.sld)
    }

    /// Core: PS2-style public suffix.
    pub fn tld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<&'a str> {
        self.match_tld(host, opts).map(|(_, t)| t)
    }

    fn match_tld<'a>(&self, s: &'a str, opts: MatchOpts<'_>) -> Option<(usize, &'a str)> {
        if s.is_empty() {
            return None;
        }
        if self.root.kids.is_empty() {
            if opts.strict {
                return None;
            }
            let last = s.rfind('.').map(|i| &s[i + 1..]).unwrap_or(s);
            if last.is_empty() {
                return None;
            }
            let start = s.len() - last.len();
            return Some((start.saturating_sub(1), last));
        }

        let mut lbl_end: isize = 0;
        let mut lbl_start: isize = s.len() as isize;
        let mut tld_start: isize = -1;
        let mut match_found = false;
        let mut parent: Option<&Node> = Some(&self.root);

        while lbl_end != -1 && lbl_start != -1 && parent.is_some() {
            lbl_end = lbl_start;
            lbl_start = rfind_dot(s, lbl_start);

            let lbl = &s[(lbl_start + 1) as usize..lbl_end as usize];
            let node = parent.unwrap();

            let mut next = node.kids.get(lbl);
            if next.is_none() && opts.wildcard {
                next = node.kids.get("*");
            }

            match next {
                Some(n) => match n.leaf {
                    Leaf::Positive => {
                        if accept_type(n, opts.types) {
                            tld_start = lbl_start; // exact match extends suffix
                        }
                        parent = Some(n);
                        match_found = true;
                    }
                    Leaf::Negative => {
                        tld_start = lbl_end; // exception: revert one label
                        match_found = true;
                        break; // <-- critical: stop here
                    }
                    Leaf::None => {
                        parent = Some(n);
                        match_found = true;
                    }
                },
                None => {
                    parent = None;
                    if match_found {
                        tld_start = lbl_end; // revert one label after last match
                    }
                }
            }
        }

        if !match_found && opts.strict {
            return None;
        }
        let start = (tld_start + 1) as usize;
        let res = &s[start..];
        if res.is_empty() {
            None
        } else {
            Some((tld_start as usize, res))
        }
    }
}

fn rfind_dot(s: &str, end: isize) -> isize {
    match s[..end as usize].rfind('.') {
        Some(i) => i as isize,
        None => -1,
    }
}

fn accept_type(n: &Node, filt: TypeFilter) -> bool {
    match (filt, n.typ) {
        (TypeFilter::Any, _) => true,
        (TypeFilter::Icann, Some(crate::rules::Type::Icann)) => true,
        (TypeFilter::Private, Some(crate::rules::Type::Private)) => true,
        _ => false,
    }
}

fn normalize_view<'a>(s: &'a str, opts: MatchOpts<'_>) -> Option<&'a str> {
    if let Some(n) = opts.normalizer {
        if n.strip_trailing_dot && s.ends_with('.') {
            return Some(&s[..s.len() - 1]);
        }
        // lowercase/IDNA remain caller-owned if you want zero-copy
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{MatchOpts, Normalizer};
    use crate::rules::{Leaf, Node, RuleSet};

    fn rs_empty() -> RuleSet {
        RuleSet::default()
    }

    fn rs_com_only() -> RuleSet {
        let mut rs = RuleSet::default();
        let mut com = Node::default();
        com.leaf = Leaf::Positive;
        rs.root.kids.insert("com".into(), com);
        rs
    }

    fn rs_uk_wildcard_and_exception() -> RuleSet {
        let mut rs = RuleSet::default();

        // com => positive rule
        let mut com = Node::default();
        com.leaf = Leaf::Positive;
        rs.root.kids.insert("com".into(), com);

        // uk => wildcard positive (*.uk) and exception (!city.uk)
        let mut uk = Node::default();

        let mut star = Node::default();
        star.leaf = Leaf::Positive;
        uk.kids.insert("*".into(), star);

        let mut city = Node::default();
        city.leaf = Leaf::Negative;
        uk.kids.insert("city".into(), city);

        rs.root.kids.insert("uk".into(), uk);

        rs
    }

    #[test]
    fn split_basic_with_no_rules() {
        let rs = rs_empty();
        let m = MatchOpts::default();

        let p = rs.split("www.example.com", m).expect("parts");
        assert_eq!(p.prefix, Some("www"));
        assert_eq!(p.sll, Some("example"));
        assert_eq!(p.sld, Some("example.com"));
        assert_eq!(p.tld, "com");

        let p2 = rs.split("example.com", m).expect("parts");
        assert_eq!(p2.prefix, None);
        assert_eq!(p2.sll, Some("example"));
        assert_eq!(p2.sld, Some("example.com"));
        assert_eq!(p2.tld, "com");
    }

    #[test]
    fn leading_dot_is_tolerated_without_normalizer() {
        let rs = rs_empty();
        let m = MatchOpts::default();

        let p = rs.split(".example.com", m).expect("parts");
        assert_eq!(p.prefix, None);
        assert_eq!(p.sll, Some("example"));
        assert_eq!(p.sld, Some("example.com"));
        assert_eq!(p.tld, "com");
    }

    #[test]
    fn trailing_dot_requires_normalizer() {
        let rs = rs_empty();

        // No normalizer => blocked
        assert!(rs.split("example.com.", MatchOpts::default()).is_none());
        assert!(rs.tld("example.com.", MatchOpts::default()).is_none());
        assert!(rs.sld("example.com.", MatchOpts::default()).is_none());

        // With normalizer => works and preserves case
        let norm = Normalizer {
            strip_trailing_dot: true,
            ..Normalizer::default()
        };
        let m = MatchOpts {
            normalizer: Some(&norm),
            ..MatchOpts::default()
        };
        let p = rs.split("WWW.Example.COM.", m).expect("parts");
        assert_eq!(p.prefix, Some("WWW"));
        assert_eq!(p.sll, Some("Example"));
        assert_eq!(p.sld, Some("Example.COM"));
        assert_eq!(p.tld, "COM");
    }

    #[test]
    fn strict_mode_blocks_empty_rules() {
        let rs = rs_empty();
        let m = MatchOpts {
            strict: true,
            ..MatchOpts::default()
        };
        assert!(rs.tld("example.com", m).is_none());
        assert!(rs.sld("example.com", m).is_none());
        assert!(rs.split("example.com", m).is_none());
    }

    #[test]
    fn wildcard_enabled_vs_disabled_under_uk() {
        let rs = rs_uk_wildcard_and_exception();

        // Wildcard enabled (default): *.uk matches "bar.uk"
        // TLD = "bar.uk"; SLD (registrable) = "foo.bar.uk"
        // SLL = label immediately left of TLD = "foo"
        // No prefix remains.
        let p_wild = rs.split("foo.bar.uk", MatchOpts::default()).expect("parts");
        assert_eq!(p_wild.tld, "bar.uk");
        assert_eq!(p_wild.sld, Some("foo.bar.uk"));
        assert_eq!(p_wild.sll, Some("foo"));
        assert_eq!(p_wild.prefix, None);

        // Wildcard disabled: no match on "bar", revert one label → TLD = "uk"
        // Registrable = "bar.uk"; SLL = "bar"; Prefix = "foo"
        let m_nowild = MatchOpts { wildcard: false, ..MatchOpts::default() };
        let p_nowild = rs.split("foo.bar.uk", m_nowild).expect("parts");
        assert_eq!(p_nowild.tld, "uk");
        assert_eq!(p_nowild.sld, Some("bar.uk"));
        assert_eq!(p_nowild.sll, Some("bar"));
        assert_eq!(p_nowild.prefix, Some("foo"));
    }


    #[test]
    fn exception_city_under_uk() {
        let rs = rs_uk_wildcard_and_exception();
        let m = MatchOpts::default();

        // Exception (!city.uk) => tld is "uk", sld is "city.uk"
        let p = rs.split("foo.city.uk", m).expect("parts");
        assert_eq!(p.prefix, Some("foo"));
        assert_eq!(p.sll, Some("city"));
        assert_eq!(p.sld, Some("city.uk"));
        assert_eq!(p.tld, "uk");
    }

    #[test]
    fn single_label_with_rule_and_without() {
        // With com rule present
        let rs = rs_com_only();
        let m = MatchOpts::default();

        let p_com = rs.split("com", m).expect("parts");
        assert_eq!(p_com.prefix, None);
        assert_eq!(p_com.sll, None);
        assert_eq!(p_com.sld, None);
        assert_eq!(p_com.tld, "com");

        // With no rules
        let rs2 = rs_empty();
        let p_local = rs2.split("localhost", m).expect("parts");
        assert_eq!(p_local.prefix, None);
        assert_eq!(p_local.sll, None);
        assert_eq!(p_local.sld, None);
        assert_eq!(p_local.tld, "localhost");
    }

    #[test]
    fn multilabel_under_com() {
        let rs = rs_com_only();
        let m = MatchOpts::default();

        let p = rs.split("x.y.z.com", m).expect("parts");
        assert_eq!(p.prefix, Some("x.y"));
        assert_eq!(p.sll, Some("z"));
        assert_eq!(p.sld, Some("z.com"));
        assert_eq!(p.tld, "com");
    }

    #[test]
    fn no_match_with_some_rules_falls_back_to_entire_host_tld() {
        let rs = rs_com_only();
        let m = MatchOpts::default();

        // Only "com" is known; for "example.org" there is no match in a non-empty tree.
        // Current behavior: entire host becomes the tld, making registrable = host.
        let p = rs.split("example.org", m).expect("parts");
        assert_eq!(p.prefix, None);
        assert_eq!(p.sll, None);
        assert_eq!(p.sld, None);
        assert_eq!(p.tld, "example.org");
    }

    #[test]
    fn rfind_dot_various_positions() {
        // "a.b.c"
        let s = "a.b.c";
        assert_eq!(rfind_dot(s, s.len() as isize), 3); // before "c"
        assert_eq!(rfind_dot(s, 3), 1); // before "b"
        assert_eq!(rfind_dot(s, 2), 1);
        assert_eq!(rfind_dot(s, 1), -1);
        assert_eq!(rfind_dot(s, 0), -1);

        // no dots
        let s2 = "abc";
        assert_eq!(rfind_dot(s2, s2.len() as isize), -1);
    }
}
