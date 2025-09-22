// Parity with python-publicsuffix2/tests.py (behavioral tests).
// Self-contained PSL snippets; no external fixtures needed.

use publicsuffix2::{
    List,
    options::{MatchOpts, Normalizer},
};

// Load the PSL used for tests
const PSL: &str = include_str!("fixtures/public_suffix_list.dat");

fn create_list(psl: &str) -> List {
    List::parse(psl).expect("parse")
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
    let list = create_list(PSL);
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
    let list = create_list(PSL);
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
    let list = create_list(PSL);
    let m = MatchOpts::default();
    assert_sld_tld!(list, "biz", m, Some("biz"), Some("biz"));
    assert_sld_tld!(list, "domain.biz", m, Some("domain.biz"), Some("biz"));
    assert_sld_tld!(list, "a.b.domain.biz", m, Some("domain.biz"), Some("biz"));
}

#[test]
fn multi_level_rules_com_and_uk_com() {
    let list = create_list(PSL);
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
    let list = create_list(PSL);
    let m = MatchOpts::default();
    assert_sld_tld!(list, "er", m, Some("er"), Some("er"));
    assert_sld_tld!(list, "c.er", m, Some("c.er"), Some("c.er"));
    assert_sld_tld!(list, "b.c.er", m, Some("b.c.er"), Some("c.er"));
    assert_sld_tld!(list, "a.b.c.er", m, Some("b.c.er"), Some("c.er"));
}

#[test]
fn wildcard_with_exception_ck() {
    let list = create_list(PSL);
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
    let list = create_list(PSL);
    let m = MatchOpts::default();

    // jp block
    assert_sld_tld!(list, "test.jp", m, Some("test.jp"), Some("jp"));
    assert_sld_tld!(list, "www.test.jp", m, Some("test.jp"), Some("jp"));
    assert_sld_tld!(list, "ac.jp", m, Some("ac.jp"), Some("ac.jp"));
    assert_sld_tld!(list, "test.ac.jp", m, Some("test.ac.jp"), Some("ac.jp"));
    assert_sld_tld!(list, "www.test.ac.jp", m, Some("test.ac.jp"), Some("ac.jp"));

    assert_sld_tld!(list, "kobe.jp", m, Some("kobe.jp"), Some("kobe.jp"));
    assert_sld_tld!(
        list,
        "c.kobe.jp",
        m,
        Some("c.kobe.jp"),
        Some("c.kobe.jp")
    );

    // Note: ide.kyoto.jp is a registry-reserved 3LD, behaves like a public suffix.
    assert_sld_tld!(list, "ide.kyoto.jp", m, Some("ide.kyoto.jp"), Some("ide.kyoto.jp"));
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
    let list = create_list(PSL);
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

    // Punycode variants (treat as raw ASCII unless you wire IDNA in Normalizer)
    let list2 = create_list(PSL); // 中国 / 公司.cn
    let m2 = MatchOpts::default();
    assert_sld_tld!(
        list2,
        "xn--85x722f.xn--fiqs8s",
        m2,
        Some("xn--85x722f.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );
    assert_sld_tld!(
        list2,
        "www.xn--85x722f.xn--fiqs8s",
        m2,
        Some("xn--85x722f.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );
    assert_sld_tld!(
        list2,
        "shishi.xn--fiqs8s",
        m2,
        Some("shishi.xn--fiqs8s"),
        Some("xn--fiqs8s")
    );

    assert_sld_tld!(
        list2,
        "xn--85x722f.xn--55qx5d.cn",
        m2,
        Some("xn--85x722f.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
    assert_sld_tld!(
        list2,
        "www.xn--85x722f.xn--55qx5d.cn",
        m2,
        Some("xn--85x722f.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
    assert_sld_tld!(
        list2,
        "shishi.xn--55qx5d.cn",
        m2,
        Some("shishi.xn--55qx5d.cn"),
        Some("xn--55qx5d.cn")
    );
}

#[test]
fn wildcard_pg_toggle() {
    let list = create_list(PSL);

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
    let list = create_list(PSL);
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
