use publicsuffix2::options::{CommentPolicy, LoadOpts, MatchOpts, Normalizer, SectionPolicy};

#[test]
fn loadopts_default_values() {
    let opts = LoadOpts::default();
    assert!(matches!(opts.sections, SectionPolicy::Auto));
    assert!(matches!(opts.comments, CommentPolicy::Common));
    assert!(!opts.strict_rules);
    assert!(!opts.collect_warnings);
}

#[test]
fn loadopts_is_copy_and_clone() {
    let a = LoadOpts::default();
    let b = a; // Copy
    let _ = a; // still usable if Copy
               // basic field checks on the copied value
    assert!(matches!(b.sections, SectionPolicy::Auto));
    assert!(matches!(b.comments, CommentPolicy::Common));

    let c = b; // another implicit copy
    let _ = c;
}

#[test]
fn loadopts_update_from_default() {
    let opts = LoadOpts {
        strict_rules: true,
        collect_warnings: true,
        ..LoadOpts::default()
    };
    assert!(matches!(opts.sections, SectionPolicy::Auto));
    assert!(matches!(opts.comments, CommentPolicy::Common));
    assert!(opts.strict_rules);
    assert!(opts.collect_warnings);
}

#[test]
fn section_and_comment_policy_are_copy() {
    let s1 = SectionPolicy::Ignore;
    let s2 = s1; // Copy
    let _ = s1; // still usable
    assert!(matches!(s2, SectionPolicy::Ignore));

    let c1 = CommentPolicy::OfficialOnly;
    let c2 = c1; // Copy
    let _ = c1; // still usable
    assert!(matches!(c2, CommentPolicy::OfficialOnly));
}

#[test]
fn matchopts_default_values() {
    let m = MatchOpts::default();
    assert!(m.wildcard);
    assert!(!m.strict);
    assert!(m.normalizer.is_some());
    let n = m.normalizer.unwrap();
    assert!(n.lowercase);
    assert!(n.strip_trailing_dot);
    assert_eq!(n.idna_ascii, cfg!(feature = "idna"));
}

#[test]
fn matchopts_is_copy_and_holds_normalizer_ref() {
    let norm = Normalizer {
        lowercase: true,
        strip_trailing_dot: true,
        idna_ascii: true,
    };
    let m1 = MatchOpts {
        normalizer: Some(&norm),
        ..MatchOpts::default()
    };
    assert!(m1.normalizer.is_some());
    let m2 = m1; // Copy
    let _ = m1; // still usable if Copy

    // The reference should be identical after copy
    let p1 = m2.normalizer.unwrap();
    assert!(p1.lowercase && p1.strip_trailing_dot && p1.idna_ascii);
    // pointer identity
    assert!(core::ptr::eq(m2.normalizer.unwrap(), p1));
}

#[test]
fn normalizer_default_is_all_false() {
    let n = Normalizer::default();
    assert!(!n.lowercase);
    assert!(!n.strip_trailing_dot);
    assert!(!n.idna_ascii);
}

#[test]
fn normalizer_update_from_default() {
    let n = Normalizer {
        lowercase: true,
        ..Normalizer::default()
    };
    assert!(n.lowercase);
    assert!(!n.strip_trailing_dot);
    assert!(!n.idna_ascii);
}

#[test]
fn normalizer_presets() {
    // ps2()
    let n_ps2 = Normalizer::ps2();
    assert!(n_ps2.lowercase);
    assert!(n_ps2.strip_trailing_dot);
    assert_eq!(n_ps2.idna_ascii, cfg!(feature = "idna"));

    // raw()
    let n_raw = Normalizer::raw();
    assert!(!n_raw.lowercase);
    assert!(!n_raw.strip_trailing_dot);
    assert!(!n_raw.idna_ascii);

    // lowercase_only()
    let n_lower = Normalizer::lowercase_only();
    assert!(n_lower.lowercase);
    assert!(!n_lower.strip_trailing_dot);
    assert!(!n_lower.idna_ascii);

    // strip_dot_only()
    let n_dot = Normalizer::strip_dot_only();
    assert!(!n_dot.lowercase);
    assert!(n_dot.strip_trailing_dot);
    assert!(!n_dot.idna_ascii);

    // idna_only()
    let n_idna = Normalizer::idna_only();
    assert!(!n_idna.lowercase);
    assert!(!n_idna.strip_trailing_dot);
    assert!(n_idna.idna_ascii);
}

#[test]
fn matchopts_presets() {
    // ps2()
    let m_ps2 = MatchOpts::ps2();
    assert!(m_ps2.wildcard);
    assert!(!m_ps2.strict);
    assert!(m_ps2.normalizer.is_some());

    // raw()
    let m_raw = MatchOpts::raw();
    assert!(m_raw.wildcard);
    assert!(!m_raw.strict);
    assert!(m_raw.normalizer.is_none());

    // with_normalizer()
    let norm = Normalizer::raw();
    let m_with = MatchOpts::with_normalizer(&norm);
    assert!(m_with.normalizer.is_some());
    assert!(core::ptr::eq(m_with.normalizer.unwrap(), &norm));
}
