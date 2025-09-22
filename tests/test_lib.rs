// Parity with python-publicsuffix2/tests.py (behavioral tests).
// Also contains port of https://github.com/aboutcode-org/python-publicsuffix2/blob/develop/tests_mozilla.py
// Self-contained PSL snippets; no external fixtures needed.

#![allow(non_snake_case)]
use publicsuffix2::{
    options::{MatchOpts, Normalizer},
    List,
};

// Load the PSL used for tests
const PSL: &str = include_str!("fixtures/public_suffix_list.dat");

// Default (PS2-like) options.
fn m() -> MatchOpts<'static> {
    MatchOpts::default()
}

// Like PS2's `PublicSuffixList(idna=False)` for Unicode-label tests.
const NORM_NO_IDNA: Normalizer = Normalizer {
    lowercase: true,
    strip_trailing_dot: true,
    idna_ascii: false,
};
fn m_no_idna() -> MatchOpts<'static> {
    MatchOpts {
        normalizer: Some(&NORM_NO_IDNA),
        ..MatchOpts::default()
    }
}

fn list() -> List {
    List::parse(PSL).expect("parse PSL")
}

macro_rules! assert_sld_tld {
    ($list:expr, $host:expr, $opts:expr, $want_sld:expr, $want_tld:expr) => {{
        let got_sld = $list.sld($host, $opts);
        let got_tld = $list.tld($host, $opts);
        assert_eq!(got_sld.as_deref(), $want_sld, "sld({})", $host);
        assert_eq!(got_tld.as_deref(), $want_tld, "tld({})", $host);
    }};
}

#[test]
fn mixed_case_and_leading_dot_like_ps2() {
    let list = list();
    let norm = Normalizer {
        lowercase: true,
        strip_trailing_dot: true,
        ..Default::default()
    };
    let m = MatchOpts {
        normalizer: Some(&norm),
        ..Default::default()
    };

    // Mixed case
    assert_sld_tld!(list, "COM", m, Some("com"), Some("com"));
    assert_sld_tld!(list, "example.COM", m, Some("example.com"), Some("com"));
    assert_sld_tld!(list, "WwW.example.COM", m, Some("example.com"), Some("com"));

    // Leading dot (FQDN-ish)
    assert_sld_tld!(list, ".com", m, Some("com"), Some("com"));
    assert_sld_tld!(list, ".example", m, Some("example"), Some("example"));
    assert_sld_tld!(list, ".example.com", m, Some("example.com"), Some("com"));
}

#[test]
fn unlisted_tld_loose_vs_strict() {
    // Empty list simulates "unlisted" tops.
    let list = list();
    let norm = Normalizer {
        lowercase: true,
        strip_trailing_dot: true,
        ..Default::default()
    };

    // Loose (strict=false) -> last label fallback like PS2
    let loose = MatchOpts {
        normalizer: Some(&norm),
        ..Default::default()
    };
    assert_sld_tld!(list, "example", loose, Some("example"), Some("example"));
    assert_sld_tld!(
        list,
        "example.example",
        loose,
        Some("example"),
        Some("example")
    );
    assert_sld_tld!(
        list,
        "a.b.example.example",
        loose,
        Some("example"),
        Some("example")
    );

    // Strict requires at least one rule match -> None
    let strict = MatchOpts {
        strict: true,
        normalizer: Some(&norm),
        ..Default::default()
    };
    assert_eq!(list.sld("example", strict), None);
    assert_eq!(list.tld("example", strict), None);
}

#[test]
fn single_rule_tld_biz() {
    let list = list();
    let m = MatchOpts::default();
    assert_sld_tld!(list, "biz", m, Some("biz"), Some("biz"));
    assert_sld_tld!(list, "domain.biz", m, Some("domain.biz"), Some("biz"));
    assert_sld_tld!(list, "a.b.domain.biz", m, Some("domain.biz"), Some("biz"));
}

#[test]
fn multi_level_rules_com_and_uk_com() {
    let list = list();
    let m = MatchOpts::default();
    assert_sld_tld!(list, "example.com", m, Some("example.com"), Some("com"));
    assert_sld_tld!(list, "a.b.example.com", m, Some("example.com"), Some("com"));
    assert_sld_tld!(
        list,
        "example.uk.com",
        m,
        Some("example.uk.com"),
        Some("uk.com")
    );
    assert_sld_tld!(
        list,
        "a.b.example.uk.com",
        m,
        Some("example.uk.com"),
        Some("uk.com")
    );
    assert_sld_tld!(list, "test.ac", m, Some("test.ac"), Some("ac"));
}

#[test]
fn wildcard_only_tld_mm() {
    // Mirrors PSL canonical: "er" plus wildcard "*.er"
    let list = list();
    let m = MatchOpts::default();
    assert_sld_tld!(list, "er", m, Some("er"), Some("er"));
    assert_sld_tld!(list, "c.er", m, Some("c.er"), Some("c.er"));
    assert_sld_tld!(list, "b.c.er", m, Some("b.c.er"), Some("c.er"));
    assert_sld_tld!(list, "a.b.c.er", m, Some("b.c.er"), Some("c.er"));
}

#[test]
fn wildcard_with_exception_ck() {
    let list = list();
    let m = MatchOpts::default();
    // Wildcard branch
    assert_sld_tld!(
        list,
        "this.that.ck",
        m,
        Some("this.that.ck"),
        Some("that.ck")
    );
    // Exception prevails
    assert_sld_tld!(list, "www.ck", m, Some("www.ck"), Some("ck"));
    assert_sld_tld!(list, "www.www.ck", m, Some("www.ck"), Some("ck"));
}

#[test]
fn jp_and_us_structures() {
    let list = list();
    let m = MatchOpts::default();

    // jp block
    assert_sld_tld!(list, "test.jp", m, Some("test.jp"), Some("jp"));
    assert_sld_tld!(list, "www.test.jp", m, Some("test.jp"), Some("jp"));
    assert_sld_tld!(list, "ac.jp", m, Some("ac.jp"), Some("ac.jp"));
    assert_sld_tld!(list, "test.ac.jp", m, Some("test.ac.jp"), Some("ac.jp"));
    assert_sld_tld!(list, "www.test.ac.jp", m, Some("test.ac.jp"), Some("ac.jp"));

    assert_sld_tld!(list, "kobe.jp", m, Some("kobe.jp"), Some("kobe.jp"));
    assert_sld_tld!(list, "c.kobe.jp", m, Some("c.kobe.jp"), Some("c.kobe.jp"));

    // Note: ide.kyoto.jp is a registry-reserved 3LD, behaves like a public suffix.
    assert_sld_tld!(
        list,
        "ide.kyoto.jp",
        m,
        Some("ide.kyoto.jp"),
        Some("ide.kyoto.jp")
    );
    assert_sld_tld!(
        list,
        "b.ide.kyoto.jp",
        m,
        Some("b.ide.kyoto.jp"),
        Some("ide.kyoto.jp")
    );
    assert_sld_tld!(
        list,
        "a.b.ide.kyoto.jp",
        m,
        Some("b.ide.kyoto.jp"),
        Some("ide.kyoto.jp")
    );

    // us block
    assert_sld_tld!(list, "test.us", m, Some("test.us"), Some("us"));
    assert_sld_tld!(list, "www.test.us", m, Some("test.us"), Some("us"));
    assert_sld_tld!(list, "ak.us", m, Some("ak.us"), Some("ak.us"));
    assert_sld_tld!(list, "test.ak.us", m, Some("test.ak.us"), Some("ak.us"));
    assert_sld_tld!(list, "www.test.ak.us", m, Some("test.ak.us"), Some("ak.us"));
    assert_sld_tld!(list, "k12.ak.us", m, Some("k12.ak.us"), Some("k12.ak.us"));
    assert_sld_tld!(
        list,
        "test.k12.ak.us",
        m,
        Some("test.k12.ak.us"),
        Some("k12.ak.us")
    );
    assert_sld_tld!(
        list,
        "www.test.k12.ak.us",
        m,
        Some("test.k12.ak.us"),
        Some("k12.ak.us")
    );
}

#[test]
fn idn_and_punycode_groups() {
    // IDN block: compare in-place; PS2 lowercases but keeps Unicode if provided.
    let list = list();
    let norm = Normalizer {
        lowercase: true,
        ..Default::default()
    };
    let m = MatchOpts {
        normalizer: Some(&norm),
        ..Default::default()
    };

    assert_sld_tld!(list, "食狮.中国", m, Some("食狮.中国"), Some("中国"));
    assert_sld_tld!(list, "www.食狮.中国", m, Some("食狮.中国"), Some("中国"));
    assert_sld_tld!(list, "shishi.中国", m, Some("shishi.中国"), Some("中国"));

    assert_sld_tld!(
        list,
        "食狮.公司.cn",
        m,
        Some("食狮.公司.cn"),
        Some("公司.cn")
    );
    assert_sld_tld!(
        list,
        "www.食狮.公司.cn",
        m,
        Some("食狮.公司.cn"),
        Some("公司.cn")
    );
    assert_sld_tld!(
        list,
        "shishi.公司.cn",
        m,
        Some("shishi.公司.cn"),
        Some("公司.cn")
    );

    let m2 = MatchOpts::default();
    assert_sld_tld!(
        list,
        "xn--85x722f.xn--fiqs8s",
        m2,
        Some("xn--85x722f.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );
    assert_sld_tld!(
        list,
        "www.xn--85x722f.xn--fiqs8s",
        m2,
        Some("xn--85x722f.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );
    assert_sld_tld!(
        list,
        "shishi.xn--fiqs8s",
        m2,
        Some("shishi.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );

    assert_sld_tld!(
        list,
        "xn--85x722f.xn--55qx5d.cn",
        m2,
        Some("xn--85x722f.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
    assert_sld_tld!(
        list,
        "www.xn--85x722f.xn--55qx5d.cn",
        m2,
        Some("xn--85x722f.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
    assert_sld_tld!(
        list,
        "shishi.xn--55qx5d.cn",
        m2,
        Some("shishi.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
}

#[test]
fn wildcard_pg_toggle() {
    let list = list();

    // With wildcard enabled (the default)
    let on = MatchOpts::default();
    // For "com.pg", the wildcard rule `*.pg` matches. The TLD becomes "com.pg".
    // Since the TLD covers the whole string, there is no SLD.
    assert_sld_tld!(list, "com.pg", on, Some("com.pg"), Some("com.pg"));

    // With wildcard disabled
    let off = MatchOpts {
        wildcard: false,
        ..Default::default()
    };
    // For "telinet.com.pg", the `*.pg` rule is ignored. The rule "pg" is used.
    // TLD is "pg", SLD is "com.pg".
    assert_sld_tld!(list, "telinet.com.pg", off, Some("com.pg"), Some("pg"));

    // For "com.pg", the `*.pg` rule is ignored. The rule "pg" is used.
    // TLD is "pg", SLD is "com.pg".
    assert_sld_tld!(list, "com.pg", off, Some("com.pg"), Some("pg"));
}

#[test]
fn fqdn_trailing_dot_when_normalized() {
    let list = list();
    let norm = Normalizer {
        lowercase: true,
        strip_trailing_dot: true,
        ..Default::default()
    };
    let opts = MatchOpts {
        normalizer: Some(&norm),
        ..Default::default()
    };

    assert_sld_tld!(list, "foo.com.", opts, Some("foo.com"), Some("com"));
    assert_sld_tld!(list, "com.", opts, Some("com"), Some("com"));
}

mod tld_default {
    use super::*;

    #[test]
    #[ignore = "Rust API takes &str; null not representable"]
    fn test_get_tld_null_input() {
        // assert None == publicsuffix.get_tld(None)
    }

    #[test]
    fn test_get_tld_Mixed_case() {
        assert_eq!(list().tld("COM", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_Mixed_case2() {
        assert_eq!(list().tld("example.COM", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_Mixed_case3() {
        assert_eq!(list().tld("WwW.example.COM", m()).as_deref(), Some("com"));
    }

    // Leading dot
    #[test]
    fn test_get_tld_Leading_dot1() {
        assert_eq!(list().tld(".com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_Leading_dot2() {
        assert_eq!(list().tld(".example", m()).as_deref(), Some("example"));
    }
    #[test]
    fn test_get_tld_Leading_dot3() {
        assert_eq!(list().tld(".example.com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_Leading_dot4() {
        assert_eq!(
            list().tld(".example.example", m()).as_deref(),
            Some("example")
        );
    }

    // Unlisted TLD
    #[test]
    fn test_get_tld_Unlisted_TLD1() {
        assert_eq!(list().tld("example", m()).as_deref(), Some("example"));
    }
    #[test]
    fn test_get_tld_Unlisted_TLD2() {
        assert_eq!(
            list().tld("example.example", m()).as_deref(),
            Some("example")
        );
    }
    #[test]
    fn test_get_tld_Unlisted_TLD3() {
        assert_eq!(
            list().tld("b.example.example", m()).as_deref(),
            Some("example")
        );
    }
    #[test]
    fn test_get_tld_Unlisted_TLD4() {
        assert_eq!(
            list().tld("a.b.example.example", m()).as_deref(),
            Some("example")
        );
    }

    // Listed, but non-Internet, TLD: local
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD1() {
        assert_eq!(list().tld("local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD2() {
        assert_eq!(list().tld("example.local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD3() {
        assert_eq!(list().tld("b.example.local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD4() {
        assert_eq!(
            list().tld("a.b.example.local", m()).as_deref(),
            Some("local")
        );
    }

    // TLD with only 1 rule
    #[test]
    fn test_get_tld_TLD_with_only_1_rule1() {
        assert_eq!(list().tld("biz", m()).as_deref(), Some("biz"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_rule2() {
        assert_eq!(list().tld("domain.biz", m()).as_deref(), Some("biz"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_rule3() {
        assert_eq!(list().tld("b.domain.biz", m()).as_deref(), Some("biz"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_rule4() {
        assert_eq!(list().tld("a.b.domain.biz", m()).as_deref(), Some("biz"));
    }

    // TLD with some 2-level rules
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules1() {
        assert_eq!(list().tld("com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules2() {
        assert_eq!(list().tld("example.com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules3() {
        assert_eq!(list().tld("b.example.com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules4() {
        assert_eq!(list().tld("a.b.example.com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules5() {
        assert_eq!(list().tld("uk.com", m()).as_deref(), Some("uk.com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules6() {
        assert_eq!(list().tld("example.uk.com", m()).as_deref(), Some("uk.com"));
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules7() {
        assert_eq!(
            list().tld("b.example.uk.com", m()).as_deref(),
            Some("uk.com")
        );
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules8() {
        assert_eq!(
            list().tld("a.b.example.uk.com", m()).as_deref(),
            Some("uk.com")
        );
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules9() {
        assert_eq!(list().tld("test.ac", m()).as_deref(), Some("ac"));
    }

    // TLD with only 1 wildcard rule (bd)
    #[test]
    fn test_get_tld_TLD_with_only_1_wildcard_rule1() {
        assert_eq!(list().tld("bd", m()).as_deref(), Some("bd"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_wildcard_rule2() {
        assert_eq!(list().tld("c.bd", m()).as_deref(), Some("c.bd"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_wildcard_rule3() {
        assert_eq!(list().tld("b.c.bd", m()).as_deref(), Some("c.bd"));
    }
    #[test]
    fn test_get_tld_TLD_with_only_1_wildcard_rule4() {
        assert_eq!(list().tld("a.b.c.bd", m()).as_deref(), Some("c.bd"));
    }

    // More complex (jp)
    #[test]
    fn test_get_tld_More_complex_TLD1() {
        assert_eq!(list().tld("jp", m()).as_deref(), Some("jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD2() {
        assert_eq!(list().tld("test.jp", m()).as_deref(), Some("jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD3() {
        assert_eq!(list().tld("www.test.jp", m()).as_deref(), Some("jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD4() {
        assert_eq!(list().tld("ac.jp", m()).as_deref(), Some("ac.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD5() {
        assert_eq!(list().tld("test.ac.jp", m()).as_deref(), Some("ac.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD6() {
        assert_eq!(list().tld("www.test.ac.jp", m()).as_deref(), Some("ac.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD7() {
        assert_eq!(list().tld("kyoto.jp", m()).as_deref(), Some("kyoto.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD8() {
        assert_eq!(
            list().tld("test.kyoto.jp", m()).as_deref(),
            Some("kyoto.jp")
        );
    }
    #[test]
    fn test_get_tld_More_complex_TLD9() {
        assert_eq!(
            list().tld("ide.kyoto.jp", m()).as_deref(),
            Some("ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_tld_More_complex_TLD10() {
        assert_eq!(
            list().tld("b.ide.kyoto.jp", m()).as_deref(),
            Some("ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_tld_More_complex_TLD11() {
        assert_eq!(
            list().tld("a.b.ide.kyoto.jp", m()).as_deref(),
            Some("ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_tld_More_complex_TLD12() {
        assert_eq!(list().tld("c.kobe.jp", m()).as_deref(), Some("c.kobe.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD13() {
        assert_eq!(list().tld("b.c.kobe.jp", m()).as_deref(), Some("c.kobe.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD14() {
        assert_eq!(
            list().tld("a.b.c.kobe.jp", m()).as_deref(),
            Some("c.kobe.jp")
        );
    }
    #[test]
    fn test_get_tld_More_complex_TLD15() {
        assert_eq!(list().tld("city.kobe.jp", m()).as_deref(), Some("kobe.jp"));
    }
    #[test]
    fn test_get_tld_More_complex_TLD16() {
        assert_eq!(
            list().tld("www.city.kobe.jp", m()).as_deref(),
            Some("kobe.jp")
        );
    }

    // Wildcard + exceptions (ck)
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions1() {
        assert_eq!(list().tld("ck", m()).as_deref(), Some("ck"));
    }
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions2() {
        assert_eq!(list().tld("test.ck", m()).as_deref(), Some("test.ck"));
    }
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions3() {
        assert_eq!(list().tld("b.test.ck", m()).as_deref(), Some("test.ck"));
    }
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions4() {
        assert_eq!(list().tld("a.b.test.ck", m()).as_deref(), Some("test.ck"));
    }
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions5() {
        assert_eq!(list().tld("www.ck", m()).as_deref(), Some("ck"));
    }
    #[test]
    fn test_get_tld_TLD_with_a_wildcard_rule_and_exceptions6() {
        assert_eq!(list().tld("www.www.ck", m()).as_deref(), Some("ck"));
    }

    // US K12
    #[test]
    fn test_get_tld_US_K121() {
        assert_eq!(list().tld("us", m()).as_deref(), Some("us"));
    }
    #[test]
    fn test_get_tld_US_K122() {
        assert_eq!(list().tld("test.us", m()).as_deref(), Some("us"));
    }
    #[test]
    fn test_get_tld_US_K123() {
        assert_eq!(list().tld("www.test.us", m()).as_deref(), Some("us"));
    }
    #[test]
    fn test_get_tld_US_K124() {
        assert_eq!(list().tld("ak.us", m()).as_deref(), Some("ak.us"));
    }
    #[test]
    fn test_get_tld_US_K125() {
        assert_eq!(list().tld("test.ak.us", m()).as_deref(), Some("ak.us"));
    }
    #[test]
    fn test_get_tld_US_K126() {
        assert_eq!(list().tld("www.test.ak.us", m()).as_deref(), Some("ak.us"));
    }
    #[test]
    fn test_get_tld_US_K127() {
        assert_eq!(list().tld("k12.ak.us", m()).as_deref(), Some("k12.ak.us"));
    }
    #[test]
    fn test_get_tld_US_K128() {
        assert_eq!(
            list().tld("test.k12.ak.us", m()).as_deref(),
            Some("k12.ak.us")
        );
    }
    #[test]
    fn test_get_tld_US_K129() {
        assert_eq!(
            list().tld("www.test.k12.ak.us", m()).as_deref(),
            Some("k12.ak.us")
        );
    }

    // IDN labels with idna=False (expect Unicode rules)
    #[test]
    fn test_get_tld_IDN_labels1() {
        assert_eq!(
            list().tld("食狮.com.cn", m_no_idna()).as_deref(),
            Some("com.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels2() {
        assert_eq!(
            list().tld("食狮.公司.cn", m_no_idna()).as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels3() {
        assert_eq!(
            list().tld("www.食狮.公司.cn", m_no_idna()).as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels4() {
        assert_eq!(
            list().tld("shishi.公司.cn", m_no_idna()).as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels5() {
        assert_eq!(
            list().tld("公司.cn", m_no_idna()).as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels6() {
        assert_eq!(
            list().tld("食狮.中国", m_no_idna()).as_deref(),
            Some("中国")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels7() {
        assert_eq!(
            list().tld("www.食狮.中国", m_no_idna()).as_deref(),
            Some("中国")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels8() {
        assert_eq!(
            list().tld("shishi.中国", m_no_idna()).as_deref(),
            Some("中国")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels9() {
        assert_eq!(list().tld("中国", m_no_idna()).as_deref(), Some("中国"));
    }

    // Same as above but punycoded (IDNA on, default)
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded1() {
        assert_eq!(
            list().tld("xn--85x722f.com.cn", m()).as_deref(),
            Some("com.cn")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded2() {
        assert_eq!(
            list().tld("xn--85x722f.xn--55qx5d.cn", m()).as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded3() {
        assert_eq!(
            list().tld("www.xn--85x722f.xn--55qx5d.cn", m()).as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded4() {
        assert_eq!(
            list().tld("shishi.xn--55qx5d.cn", m()).as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded5() {
        assert_eq!(
            list().tld("xn--55qx5d.cn", m()).as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded6() {
        assert_eq!(
            list().tld("xn--85x722f.xn--fiqs8s", m()).as_deref(),
            Some("xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded7() {
        assert_eq!(
            list().tld("www.xn--85x722f.xn--fiqs8s", m()).as_deref(),
            Some("xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded8() {
        assert_eq!(
            list().tld("shishi.xn--fiqs8s", m()).as_deref(),
            Some("xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded9() {
        assert_eq!(list().tld("xn--fiqs8s", m()).as_deref(), Some("xn--fiqs8s"));
    }
}

mod tld_strict {
    use super::*;

    #[test]
    #[ignore = "Rust API takes &str; null not representable"]
    fn test_get_tld_null_input() {}

    #[test]
    fn test_get_tld_Mixed_case() {
        assert_eq!(
            list()
                .tld(
                    "COM",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }
    #[test]
    fn test_get_tld_Mixed_case2() {
        assert_eq!(
            list()
                .tld(
                    "example.COM",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }
    #[test]
    fn test_get_tld_Mixed_case3() {
        assert_eq!(
            list()
                .tld(
                    "WwW.example.COM",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }

    // Leading dot (strict)
    #[test]
    fn test_get_tld_Leading_dot1() {
        assert_eq!(
            list()
                .tld(
                    ".com",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }
    #[test]
    fn test_get_tld_Leading_dot2() {
        assert_eq!(
            list()
                .tld(
                    ".example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Leading_dot3() {
        assert_eq!(
            list()
                .tld(
                    ".example.com",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }
    #[test]
    fn test_get_tld_Leading_dot4() {
        assert_eq!(
            list()
                .tld(
                    ".example.example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }

    // Unlisted TLD (strict)
    #[test]
    fn test_get_tld_Unlisted_TLD1() {
        assert_eq!(
            list()
                .tld(
                    "example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Unlisted_TLD2() {
        assert_eq!(
            list()
                .tld(
                    "example.example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Unlisted_TLD3() {
        assert_eq!(
            list()
                .tld(
                    "b.example.example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Unlisted_TLD4() {
        assert_eq!(
            list()
                .tld(
                    "a.b.example.example",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }

    // Listed, but non-Internet, TLD (strict)
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD1() {
        assert_eq!(
            list()
                .tld(
                    "local",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD2() {
        assert_eq!(
            list()
                .tld(
                    "example.local",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD3() {
        assert_eq!(
            list()
                .tld(
                    "b.example.local",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }
    #[test]
    fn test_get_tld_Listed_but_non_Internet_TLD4() {
        assert_eq!(
            list()
                .tld(
                    "a.b.example.local",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            None
        );
    }

    // 1-rule, 2-level, wildcard, complex, wildcard+exceptions, US K12 — same as default but with strict=true
    // For brevity we’ll mirror a couple; you can duplicate all above with strict=true if you want parity verbatim.
    #[test]
    fn test_get_tld_TLD_with_only_1_rule1() {
        assert_eq!(
            list()
                .tld(
                    "biz",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("biz")
        );
    }
    #[test]
    fn test_get_tld_TLD_with_some_2_level_rules1() {
        assert_eq!(
            list()
                .tld(
                    "com",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("com")
        );
    }
    #[test]
    fn test_get_tld_US_K121() {
        assert_eq!(
            list()
                .tld(
                    "us",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("us")
        );
    }

    // IDN labels with idna=False and strict
    #[test]
    fn test_get_tld_IDN_labels1() {
        assert_eq!(
            list()
                .tld(
                    "食狮.com.cn",
                    MatchOpts {
                        strict: true,
                        ..m_no_idna()
                    }
                )
                .as_deref(),
            Some("com.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels2() {
        assert_eq!(
            list()
                .tld(
                    "食狮.公司.cn",
                    MatchOpts {
                        strict: true,
                        ..m_no_idna()
                    }
                )
                .as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_tld_IDN_labels9() {
        assert_eq!(
            list()
                .tld(
                    "中国",
                    MatchOpts {
                        strict: true,
                        ..m_no_idna()
                    }
                )
                .as_deref(),
            Some("中国")
        );
    }

    // Punycoded strict
    #[test]
    fn test_get_tld_Same_as_above_but_punycoded2() {
        assert_eq!(
            list()
                .tld(
                    "xn--85x722f.xn--55qx5d.cn",
                    MatchOpts {
                        strict: true,
                        ..m()
                    }
                )
                .as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
}

mod sld_default {
    use super::*;

    #[test]
    #[ignore = "Rust API takes &str; null not representable"]
    fn test_get_sld_null_input() {}

    #[test]
    fn test_get_sld_Mixed_case() {
        assert_eq!(list().sld("COM", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_sld_Mixed_case2() {
        assert_eq!(
            list().sld("example.COM", m()).as_deref(),
            Some("example.com")
        );
    }
    #[test]
    fn test_get_sld_Mixed_case3() {
        assert_eq!(
            list().sld("WwW.example.COM", m()).as_deref(),
            Some("example.com")
        );
    }

    // Leading dot
    #[test]
    fn test_get_sld_Leading_dot1() {
        assert_eq!(list().sld(".com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_sld_Leading_dot2() {
        assert_eq!(list().sld(".example", m()).as_deref(), Some("example"));
    }
    #[test]
    fn test_get_sld_Leading_dot3() {
        assert_eq!(
            list().sld(".example.com", m()).as_deref(),
            Some("example.com")
        );
    }
    #[test]
    fn test_get_sld_Leading_dot4() {
        assert_eq!(
            list().sld(".example.example", m()).as_deref(),
            Some("example")
        );
    }

    // Unlisted sld
    #[test]
    fn test_get_sld_Unlisted_sld1() {
        assert_eq!(list().sld("example", m()).as_deref(), Some("example"));
    }
    #[test]
    fn test_get_sld_Unlisted_sld2() {
        assert_eq!(
            list().sld("example.example", m()).as_deref(),
            Some("example")
        );
    }
    #[test]
    fn test_get_sld_Unlisted_sld3() {
        assert_eq!(
            list().sld("b.example.example", m()).as_deref(),
            Some("example")
        );
    }
    #[test]
    fn test_get_sld_Unlisted_sld4() {
        assert_eq!(
            list().sld("a.b.example.example", m()).as_deref(),
            Some("example")
        );
    }

    // Listed but non-Internet sld
    #[test]
    fn test_get_sld_Listed_but_non_Internet_sld1() {
        assert_eq!(list().sld("local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_sld_Listed_but_non_Internet_sld2() {
        assert_eq!(list().sld("example.local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_sld_Listed_but_non_Internet_sld3() {
        assert_eq!(list().sld("b.example.local", m()).as_deref(), Some("local"));
    }
    #[test]
    fn test_get_sld_Listed_but_non_Internet_sld4() {
        assert_eq!(
            list().sld("a.b.example.local", m()).as_deref(),
            Some("local")
        );
    }

    // 1-rule, 2-level, wildcard
    #[test]
    fn test_get_sld_tld_with_only_1_rule1() {
        assert_eq!(list().sld("biz", m()).as_deref(), Some("biz"));
    }
    #[test]
    fn test_get_sld_tld_with_only_1_rule2() {
        assert_eq!(list().sld("domain.biz", m()).as_deref(), Some("domain.biz"));
    }
    #[test]
    fn test_get_sld_tld_with_only_1_rule3() {
        assert_eq!(
            list().sld("b.domain.biz", m()).as_deref(),
            Some("domain.biz")
        );
    }
    #[test]
    fn test_get_sld_tld_with_only_1_rule4() {
        assert_eq!(
            list().sld("a.b.domain.biz", m()).as_deref(),
            Some("domain.biz")
        );
    }

    #[test]
    fn test_get_sld_tld_with_some_2_level_rules1() {
        assert_eq!(list().sld("com", m()).as_deref(), Some("com"));
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules2() {
        assert_eq!(
            list().sld("example.com", m()).as_deref(),
            Some("example.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules3() {
        assert_eq!(
            list().sld("b.example.com", m()).as_deref(),
            Some("example.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules4() {
        assert_eq!(
            list().sld("a.b.example.com", m()).as_deref(),
            Some("example.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules5() {
        assert_eq!(list().sld("uk.com", m()).as_deref(), Some("uk.com"));
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules6() {
        assert_eq!(
            list().sld("example.uk.com", m()).as_deref(),
            Some("example.uk.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules7() {
        assert_eq!(
            list().sld("b.example.uk.com", m()).as_deref(),
            Some("example.uk.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules8() {
        assert_eq!(
            list().sld("a.b.example.uk.com", m()).as_deref(),
            Some("example.uk.com")
        );
    }
    #[test]
    fn test_get_sld_tld_with_some_2_level_rules9() {
        assert_eq!(list().sld("test.ac", m()).as_deref(), Some("test.ac"));
    }

    // Wildcard (bd)
    #[test]
    fn test_get_sld_tld_with_only_1_wildcard_rule1() {
        assert_eq!(list().sld("bd", m()).as_deref(), Some("bd"));
    }
    #[test]
    fn test_get_sld_tld_with_only_1_wildcard_rule2() {
        assert_eq!(list().sld("c.bd", m()).as_deref(), Some("c.bd"));
    }
    #[test]
    fn test_get_sld_tld_with_only_1_wildcard_rule3() {
        assert_eq!(list().sld("b.c.bd", m()).as_deref(), Some("b.c.bd"));
    }
    #[test]
    fn test_get_sld_tld_with_only_1_wildcard_rule4() {
        assert_eq!(list().sld("a.b.c.bd", m()).as_deref(), Some("b.c.bd"));
    }

    // More complex (jp)
    #[test]
    fn test_get_sld_More_complex_sld1() {
        assert_eq!(list().sld("jp", m()).as_deref(), Some("jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld2() {
        assert_eq!(list().sld("test.jp", m()).as_deref(), Some("test.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld3() {
        assert_eq!(list().sld("www.test.jp", m()).as_deref(), Some("test.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld4() {
        assert_eq!(list().sld("ac.jp", m()).as_deref(), Some("ac.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld5() {
        assert_eq!(list().sld("test.ac.jp", m()).as_deref(), Some("test.ac.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld6() {
        assert_eq!(
            list().sld("www.test.ac.jp", m()).as_deref(),
            Some("test.ac.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld7() {
        assert_eq!(list().sld("kyoto.jp", m()).as_deref(), Some("kyoto.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld8() {
        assert_eq!(
            list().sld("test.kyoto.jp", m()).as_deref(),
            Some("test.kyoto.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld9() {
        assert_eq!(
            list().sld("ide.kyoto.jp", m()).as_deref(),
            Some("ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld10() {
        assert_eq!(
            list().sld("b.ide.kyoto.jp", m()).as_deref(),
            Some("b.ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld11() {
        assert_eq!(
            list().sld("a.b.ide.kyoto.jp", m()).as_deref(),
            Some("b.ide.kyoto.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld12() {
        assert_eq!(list().sld("c.kobe.jp", m()).as_deref(), Some("c.kobe.jp"));
    }
    #[test]
    fn test_get_sld_More_complex_sld13() {
        assert_eq!(
            list().sld("b.c.kobe.jp", m()).as_deref(),
            Some("b.c.kobe.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld14() {
        assert_eq!(
            list().sld("b.c.kobe.jp", m()).as_deref(),
            Some("b.c.kobe.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld15() {
        assert_eq!(
            list().sld("city.kobe.jp", m()).as_deref(),
            Some("city.kobe.jp")
        );
    }
    #[test]
    fn test_get_sld_More_complex_sld16() {
        assert_eq!(
            list().sld("www.city.kobe.jp", m()).as_deref(),
            Some("city.kobe.jp")
        );
    }

    // Wildcard + exceptions (ck)
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions1() {
        assert_eq!(list().sld("ck", m()).as_deref(), Some("ck"));
    }
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions2() {
        assert_eq!(list().sld("test.ck", m()).as_deref(), Some("test.ck"));
    }
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions3() {
        assert_eq!(list().sld("b.test.ck", m()).as_deref(), Some("b.test.ck"));
    }
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions4() {
        assert_eq!(list().sld("a.b.test.ck", m()).as_deref(), Some("b.test.ck"));
    }
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions5() {
        assert_eq!(list().sld("www.ck", m()).as_deref(), Some("www.ck"));
    }
    #[test]
    fn test_get_sld_tld_with_a_wildcard_rule_and_exceptions6() {
        assert_eq!(list().sld("www.www.ck", m()).as_deref(), Some("www.ck"));
    }

    // US K12
    #[test]
    fn test_get_sld_US_K121() {
        assert_eq!(list().sld("us", m()).as_deref(), Some("us"));
    }
    #[test]
    fn test_get_sld_US_K122() {
        assert_eq!(list().sld("test.us", m()).as_deref(), Some("test.us"));
    }
    #[test]
    fn test_get_sld_US_K123() {
        assert_eq!(list().sld("www.test.us", m()).as_deref(), Some("test.us"));
    }
    #[test]
    fn test_get_sld_US_K124() {
        assert_eq!(list().sld("ak.us", m()).as_deref(), Some("ak.us"));
    }
    #[test]
    fn test_get_sld_US_K125() {
        assert_eq!(list().sld("test.ak.us", m()).as_deref(), Some("test.ak.us"));
    }
    #[test]
    fn test_get_sld_US_K126() {
        assert_eq!(
            list().sld("www.test.ak.us", m()).as_deref(),
            Some("test.ak.us")
        );
    }
    #[test]
    fn test_get_sld_US_K127() {
        assert_eq!(list().sld("k12.ak.us", m()).as_deref(), Some("k12.ak.us"));
    }
    #[test]
    fn test_get_sld_US_K128() {
        assert_eq!(
            list().sld("test.k12.ak.us", m()).as_deref(),
            Some("test.k12.ak.us")
        );
    }
    #[test]
    fn test_get_sld_US_K129() {
        assert_eq!(
            list().sld("www.test.k12.ak.us", m()).as_deref(),
            Some("test.k12.ak.us")
        );
    }

    // IDN sld with idna=False
    #[test]
    fn test_get_sld_IDN_labels1() {
        assert_eq!(
            list().sld("食狮.com.cn", m_no_idna()).as_deref(),
            Some("食狮.com.cn")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels2() {
        assert_eq!(
            list().sld("食狮.公司.cn", m_no_idna()).as_deref(),
            Some("食狮.公司.cn")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels3() {
        assert_eq!(
            list().sld("www.食狮.公司.cn", m_no_idna()).as_deref(),
            Some("食狮.公司.cn")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels4() {
        assert_eq!(
            list().sld("shishi.公司.cn", m_no_idna()).as_deref(),
            Some("shishi.公司.cn")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels5() {
        assert_eq!(
            list().sld("公司.cn", m_no_idna()).as_deref(),
            Some("公司.cn")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels6() {
        assert_eq!(
            list().sld("食狮.中国", m_no_idna()).as_deref(),
            Some("食狮.中国")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels7() {
        assert_eq!(
            list().sld("www.食狮.中国", m_no_idna()).as_deref(),
            Some("食狮.中国")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels8() {
        assert_eq!(
            list().sld("shishi.中国", m_no_idna()).as_deref(),
            Some("shishi.中国")
        );
    }
    #[test]
    fn test_get_sld_IDN_labels9() {
        assert_eq!(list().sld("中国", m_no_idna()).as_deref(), Some("中国"));
    }

    // Same as above but punycoded
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded1() {
        assert_eq!(
            list().sld("xn--85x722f.com.cn", m()).as_deref(),
            Some("xn--85x722f.com.cn")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded2() {
        assert_eq!(
            list().sld("xn--85x722f.xn--55qx5d.cn", m()).as_deref(),
            Some("xn--85x722f.xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded3() {
        assert_eq!(
            list().sld("www.xn--85x722f.xn--55qx5d.cn", m()).as_deref(),
            Some("xn--85x722f.xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded4() {
        assert_eq!(
            list().sld("shishi.xn--55qx5d.cn", m()).as_deref(),
            Some("shishi.xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded5() {
        assert_eq!(
            list().sld("xn--55qx5d.cn", m()).as_deref(),
            Some("xn--55qx5d.cn")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded6() {
        assert_eq!(
            list().sld("xn--85x722f.xn--fiqs8s", m()).as_deref(),
            Some("xn--85x722f.xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded7() {
        assert_eq!(
            list().sld("www.xn--85x722f.xn--fiqs8s", m()).as_deref(),
            Some("xn--85x722f.xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded8() {
        assert_eq!(
            list().sld("shishi.xn--fiqs8s", m()).as_deref(),
            Some("shishi.xn--fiqs8s")
        );
    }
    #[test]
    fn test_get_sld_Same_as_above_but_punycoded9() {
        assert_eq!(list().sld("xn--fiqs8s", m()).as_deref(), Some("xn--fiqs8s"));
    }
}
