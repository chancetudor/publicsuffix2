use crate::options::MatchOpts;
use crate::rules::{Leaf, Node, RuleSet, TypeFilter};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
/// Represents the constituent parts of a domain name, separated according to the Public Suffix List rules.
pub struct Parts<'a> {
    /// The part of the host that is not part of the registrable domain, if any.
    /// For `www.example.com`, this would be `www`.
    pub prefix: Option<Cow<'a, str>>, // everything left of sld
    /// The second-level label: the label immediately to the left of the public suffix.
    /// For `www.example.com`, this would be `example`.
    pub sll: Option<Cow<'a, str>>, // second-level label
    /// The registrable domain, also known as eTLD+1 (effective Top-Level Domain plus one label).
    /// For `www.example.com`, this would be `example.com`.
    pub sld: Option<Cow<'a, str>>, // registrable (eTLD+1)
    /// The public suffix (eTLD).
    /// For `www.example.com`, this would be `com`. For `www.example.co.uk`, this would be `co.uk`.
    pub tld: Cow<'a, str>, // public suffix
}

impl<'a> Parts<'a> {
    /// Converts a `Parts<'a>` into a `Parts<'static>` by cloning the internal data.
    pub fn into_owned(self) -> Parts<'static> {
        Parts {
            prefix: self.prefix.map(|v| Cow::Owned(v.into_owned())),
            sll: self.sll.map(|v| Cow::Owned(v.into_owned())),
            sld: self.sld.map(|v| Cow::Owned(v.into_owned())),
            tld: Cow::Owned(self.tld.into_owned()),
        }
    }
}

impl RuleSet {
    /// Splits a domain name into its constituent parts: prefix, second-level label,
    /// registrable domain, and public suffix.
    ///
    /// This is the most comprehensive parsing function, returning all parts of a domain.
    /// Behavior is controlled by `MatchOpts` (wildcards, strict mode, type filter,
    /// normalization).
    pub fn split<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Parts<'a>> {
        let s = normalize_view(host, opts);

        match s {
            Cow::Borrowed(b) => {
                let (_, tld) = self.match_tld(b, opts)?;
                let sld_end = b.len().saturating_sub(tld.len()).saturating_sub(1);

                // If public suffix covers the whole host, registrable domain equals the host.
                if tld.len() == b.len() {
                    return Some(Parts {
                        prefix: None,
                        sll: None,
                        sld: Some(Cow::Borrowed(b)),
                        tld: Cow::Borrowed(tld),
                    });
                }

                // Unlisted-TLD fallback: when suffix is a single label *not* in the rules,
                // collapse SLD to the TLD (e.g., "example.example" → "example", "example.local" → "local").
                if !tld.contains('.') && !self.root.kids.contains_key(tld) {
                    return Some(Parts {
                        prefix: None,
                        sll: None,
                        sld: Some(Cow::Borrowed(tld)),
                        tld: Cow::Borrowed(tld),
                    });
                }

                debug_assert_eq!(b.as_bytes()[sld_end], b'.');

                let idx = b[..sld_end].rfind('.');
                let mut start = idx.map(|i| i + 1).unwrap_or(0);
                if start == 0 && b.as_bytes().get(0) == Some(&b'.') {
                    start = 1;
                }

                let prefix = idx.filter(|&i| i > 0).map(|i| Cow::Borrowed(&b[..i]));
                let sll_slice = &b[start..sld_end];
                let sll = if !sll_slice.is_empty() {
                    Some(Cow::Borrowed(sll_slice))
                } else {
                    None
                };
                let sld = Some(Cow::Borrowed(&b[start..]));

                Some(Parts {
                    prefix,
                    sll,
                    sld,
                    tld: Cow::Borrowed(tld),
                })
            }

            Cow::Owned(o) => {
                let (_, tld) = self.match_tld(&o, opts)?;
                let sld_end = o.len().saturating_sub(tld.len()).saturating_sub(1);

                // If public suffix covers the whole host, registrable domain equals the host.
                if tld.len() == o.len() {
                    return Some(Parts {
                        prefix: None,
                        sll: None,
                        sld: Some(Cow::<str>::Owned(o.clone())),
                        tld: Cow::<str>::Owned(tld.to_string()),
                    });
                }
                if !tld.contains('.') && !self.root.kids.contains_key(tld) {
                    return Some(Parts {
                        prefix: None,
                        sll: None,
                        sld: Some(Cow::Owned(tld.to_string())),
                        tld: Cow::Owned(tld.to_string()),
                    });
                }

                debug_assert_eq!(o.as_bytes()[sld_end], b'.');

                let idx = o[..sld_end].rfind('.');
                let mut start = idx.map(|i| i + 1).unwrap_or(0);
                if start == 0 && o.as_bytes().get(0) == Some(&b'.') {
                    start = 1;
                }

                let prefix = idx
                    .filter(|&i| i > 0)
                    .map(|i| Cow::<str>::Owned(o[..i].to_string()));
                let sll = {
                    let lbl = &o[start..sld_end];
                    if !lbl.is_empty() {
                        Some(Cow::<str>::Owned(lbl.to_string()))
                    } else {
                        None
                    }
                };
                let sld = Some(Cow::<str>::Owned(o[start..].to_string()));

                Some(Parts {
                    prefix,
                    sll,
                    sld,
                    tld: Cow::<str>::Owned(tld.to_string()),
                })
            }
        }
    }

    /// Extracts the registrable domain (eTLD+1) from a host name.
    ///
    /// The registrable domain is the public suffix plus one preceding label.
    /// For example, for `www.example.com`, the registrable domain is `example.com`.
    ///
    /// This is a convenience method that calls `split` and returns only the `sld` part.
    pub fn sld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Cow<'a, str>> {
        self.split(host, opts).and_then(|p| p.sld)
    }

    /// Extracts the public suffix (eTLD) from a host name.
    ///
    /// The public suffix is the part of the domain name that is present in the Public Suffix List.
    /// For example, for `www.example.co.uk`, the public suffix is `co.uk`.
    ///
    /// This is an optimized method that directly finds the public suffix without calculating
    /// the other parts of the domain. If you need other parts, use `split`.
    pub fn tld<'a>(&self, host: &'a str, opts: MatchOpts<'_>) -> Option<Cow<'a, str>> {
        let s = normalize_view(host, opts); // Cow<'a, str>

        match s {
            Cow::Borrowed(b) => {
                let (_, tld) = self.match_tld(b, opts)?; // tld: &str inside `host`
                Some(Cow::Borrowed(tld))
            }
            Cow::Owned(o) => {
                let (_, tld) = self.match_tld(&o, opts)?; // tld: &str inside local `o`
                Some(Cow::Owned(tld.to_string())) // copy so it outlives this fn
            }
        }
    }

    fn match_tld<'s>(&self, s: &'s str, opts: MatchOpts<'_>) -> Option<(usize, &'s str)> {
        // invalid: empty label, leading dot, trailing dot (when not stripped), or ".."
        if s.is_empty() || s.ends_with('.') || s.contains("..") {
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

        let mut longest_match: Option<(isize, &Node)> = None;
        let mut parent: Option<&Node> = Some(&self.root);

        let mut lbl_end = s.len() as isize;
        let mut lbl_start = s.len() as isize;

        while lbl_end != -1 && parent.is_some() {
            lbl_start = rfind_dot(s, lbl_start);
            let lbl = &s[(lbl_start + 1) as usize..lbl_end as usize];
            let node = parent.unwrap();

            let mut next = node.kids.get(lbl);
            if next.is_none() && opts.wildcard {
                next = node.kids.get("*");
            }

            match next {
                Some(n) => {
                    if accept_type(n, opts.types) {
                        longest_match = Some((lbl_start, n));
                    }
                    parent = Some(n);
                }
                None => {
                    parent = None;
                }
            }
            lbl_end = lbl_start;
        }

        match longest_match {
            Some((tld_start, node)) => {
                // An exception rule means the public suffix is one level up from the exception.
                // e.g., for !city.uk on foo.city.uk, the match is on 'city', but the TLD is 'uk'.
                if node.leaf == Leaf::Negative {
                    let dot = s[(tld_start + 1) as usize..]
                        .find('.')
                        .map(|i| i as isize + tld_start + 1)
                        .unwrap_or(-1);
                    let start = (dot + 1) as usize;
                    return Some((dot as usize, &s[start..]));
                }

                let start = (tld_start + 1) as usize;
                Some((tld_start as usize, &s[start..]))
            }
            None => {
                if opts.strict {
                    return None;
                }
                // Non-strict fallback for unlisted TLDs: last label is the public suffix.
                let dot = s.rfind('.').map(|i| i as isize).unwrap_or(-1);
                let start = (dot + 1) as usize;
                Some((dot as usize, &s[start..]))
            }
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

fn normalize_view<'a>(s: &'a str, opts: MatchOpts<'_>) -> Cow<'a, str> {
    let Some(n) = opts.normalizer else {
        return Cow::Borrowed(s); // no normalization
    };

    // Drop a single leading dot, then handle trailing dot.
    let base = if s.starts_with('.') { &s[1..] } else { s };
    let mut out: Cow<'a, str> = if n.strip_trailing_dot && base.ends_with('.') {
        Cow::Owned(base[..base.len() - 1].to_string())
    } else {
        Cow::Borrowed(base)
    };

    // Lowercase (allocate only if needed).
    if n.lowercase {
        if out.chars().any(|c| c.is_ascii_uppercase()) {
            out = Cow::Owned(out.to_lowercase());
        }
    }

    // IDNA -> ASCII (feature-gated; allocate only if non-ASCII)
    #[cfg(feature = "idna")]
    if n.idna_ascii && !out.is_ascii() {
        if let Ok(ascii) = idna::domain_to_ascii(&out) {
            out = Cow::Owned(ascii);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::MatchOpts;
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
        assert_eq!(p.prefix, None);
        assert_eq!(p.sll, None);
        assert_eq!(p.sld, Some("com".into()));
        assert_eq!(p.tld, "com");
    }

    #[test]
    fn leading_dot_is_valid() {
        let rs = rs_empty();
        let m = MatchOpts::default();

        let p = rs.split(".com", m).expect("parts");
        assert_eq!(p.prefix, None);
        assert_eq!(p.sll, None);
        assert_eq!(p.sld, Some("com".into()));
        assert_eq!(p.tld, "com");
    }

    #[test]
    fn trailing_dot_requires_normalizer() {
        let rs = rs_empty();

        // Raw / no normalization => blocked due to trailing root label.
        let raw = MatchOpts {
            normalizer: None,
            ..MatchOpts::default()
        };
        assert!(rs.split("example.com.", raw).is_none());
        assert!(rs.tld("example.com.", raw).is_none());
        assert!(rs.sld("example.com.", raw).is_none());
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
        assert_eq!(p_wild.sld, Some("foo.bar.uk".into()));
        assert_eq!(p_wild.sll, Some("foo".into()));
        assert_eq!(p_wild.prefix, None);

        // Wildcard disabled: no match on "bar", revert one label → TLD = "uk"
        // Registrable = "bar.uk"; SLL = "bar"; Prefix = "foo"
        let m_nowild = MatchOpts {
            wildcard: false,
            ..MatchOpts::default()
        };
        let p_nowild = rs.split("foo.bar.uk", m_nowild).expect("parts");
        assert_eq!(p_nowild.tld, "uk");
        assert_eq!(p_nowild.sld, Some("bar.uk".into()));
        assert_eq!(p_nowild.sll, Some("bar".into()));
        assert_eq!(p_nowild.prefix, Some("foo".into()));
    }

    #[test]
    fn exception_city_under_uk() {
        let rs = rs_uk_wildcard_and_exception();
        let m = MatchOpts::default();

        // Exception (!city.uk) => tld is "uk", sld is "city.uk"
        let p = rs.split("foo.city.uk", m).expect("parts");
        assert_eq!(p.prefix, Some("foo".into()));
        assert_eq!(p.sll, Some("city".into()));
        assert_eq!(p.sld, Some("city.uk".into()));
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
        assert_eq!(p_com.sld, Some("com".into()));
        assert_eq!(p_com.tld, "com");

        // With no rules
        let rs2 = rs_empty();
        let p_local = rs2.split("localhost", m).expect("parts");
        assert_eq!(p_local.prefix, None);
        assert_eq!(p_local.sll, None);
        assert_eq!(p_local.sld, Some("localhost".into()));
        assert_eq!(p_local.tld, "localhost");
    }

    #[test]
    fn multilabel_under_com() {
        let rs = rs_com_only();
        let m = MatchOpts::default();

        let p = rs.split("x.y.z.com", m).expect("parts");
        assert_eq!(p.prefix, Some("x.y".into()));
        assert_eq!(p.sll, Some("z".into()));
        assert_eq!(p.sld, Some("z.com".into()));
        assert_eq!(p.tld, "com");
    }

    #[test]
    fn no_match_with_some_rules_falls_back_to_last_label_tld() {
        let rs = rs_com_only();
        let m = MatchOpts::default();

        let p = rs.split("example.org", m).expect("parts");
        assert_eq!(p.prefix.as_deref(), None);
        assert_eq!(p.sll.as_deref(), None);
        assert_eq!(p.sld.as_deref(), Some("org"));
        assert_eq!(p.tld.as_ref(), "org");

        // And in strict mode, no match at all:
        let strict = MatchOpts { strict: true, ..m };
        assert!(rs.split("example.org", strict).is_none());
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
