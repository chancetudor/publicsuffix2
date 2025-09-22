use publicsuffix2::errors::{Error, Result as PslResult, RuleSyntax, Warning};

#[test]
fn display_matches_debug_for_simple_errors() {
    let variants = [
        Error::EmptyList,
        Error::MissingSections,
        Error::NotUtf8,
        Error::LabelTooLong {
            label: "too-long".into(),
        },
        Error::RuleDepthExceeded { depth: 42 },
        Error::InvalidRule {
            rule: "com..".into(),
            reason: RuleSyntax::HasEmptyLabel,
        },
    ];

    for e in variants {
        assert_eq!(format!("{}", e), format!("{:?}", e));
    }
}

#[cfg(feature = "std")]
#[test]
fn error_implements_std_error_when_std_feature_enabled() {
    fn assert_is_std_error<E: std::error::Error + 'static>(_e: &E) {}
    let e = Error::EmptyList;
    assert_is_std_error(&e);
}

#[cfg(feature = "std")]
#[test]
fn io_variant_display_has_stable_prefix() {
    let io_err = std::io::Error::other("oops");
    let e = Error::Io(io_err);
    let s = format!("{}", e);
    assert!(s.starts_with("Io("), "unexpected Display: {s}");
}

#[test]
fn rule_syntax_is_copy_and_debug() {
    let a = RuleSyntax::ContainsIllegalChar;
    let b = a; // Copy
    assert_eq!(format!("{:?}", a), format!("{:?}", b));
}

#[test]
fn warnings_are_cloneable_and_debuggable() {
    let ws = [
        Warning::DuplicateRule { rule: "foo".into() },
        Warning::ShadowedRule { rule: "bar".into() },
        Warning::UnknownMarker {
            line: "?? marker".into(),
        },
        Warning::TrailingDotRule {
            rule: "example.com.".into(),
        },
    ];
    for w in ws {
        let w2 = w.clone();
        assert_eq!(format!("{:?}", w), format!("{:?}", w2));
    }
}

#[test]
fn result_alias_compiles_and_is_ok() {
    fn use_result(r: PslResult<()>) -> PslResult<()> {
        r
    }
    let r: PslResult<()> = Ok(());
    let out = use_result(r);
    assert!(out.is_ok());
}
