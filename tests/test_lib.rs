use publicsuffix2::{Error, List, MatchOpts};

fn sample_list() -> List {
    let psl = concat!(
        "bar.uk\n",
        "*.uk\n",
        "!city.uk\n",
        "com\n",
        "blogspot.com\n",
    );
    List::parse(psl).expect("failed to parse sample PSL")
}

#[test]
fn tld_and_sld_basic_bar_uk() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "foo.bar.uk";
    assert_eq!(list.tld(host, opts), Some("bar.uk".into()));
    assert_eq!(list.sld(host, opts), Some("foo.bar.uk".into()));
}

#[test]
fn tld_and_sld_wildcard_anything_uk() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "foo.qux.uk";
    assert_eq!(list.tld(host, opts), Some("qux.uk".into()));
    assert_eq!(list.sld(host, opts), Some("foo.qux.uk".into()));
}

#[test]
fn tld_and_sld_exception_city_uk() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "foo.city.uk";
    assert_eq!(list.tld(host, opts), Some("uk".into()));
    assert_eq!(list.sld(host, opts), Some("city.uk".into()));
}

#[test]
fn sld_none_when_no_label_left_of_tld_due_to_wildcard() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "foo.uk";
    assert_eq!(list.tld(host, opts), Some("foo.uk".into()));
    assert_eq!(list.sld(host, opts), None);
}

#[test]
fn tld_and_sld_com() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "a.b.example.com";
    assert_eq!(list.tld(host, opts), Some("com".into()));
    assert_eq!(list.sld(host, opts), Some("example.com".into()));
}

#[test]
fn tld_and_sld_private_blogspot() {
    let list = sample_list();
    let opts = MatchOpts::default();

    let host = "foo.blogspot.com";
    assert_eq!(list.tld(host, opts), Some("blogspot.com".into()));
    assert_eq!(list.sld(host, opts), Some("foo.blogspot.com".into()));
}

#[test]
fn split_examples_match_docs() {
    let list = sample_list();
    let opts = MatchOpts::default();

    // "foo.bar.uk" → TLD="bar.uk", SLD="foo.bar.uk", SLL="foo", Prefix=None
    let p = list.split("foo.bar.uk", opts).expect("parts");
    assert_eq!(p.tld, "bar.uk");
    assert_eq!(p.sld, Some("foo.bar.uk".into()));
    assert_eq!(p.sll, Some("foo".into()));
    assert_eq!(p.prefix, None);

    // "foo.city.uk" → TLD="uk", SLD="city.uk", SLL="city", Prefix=Some("foo")
    let p = list.split("foo.city.uk", opts).expect("parts");
    assert_eq!(p.tld, "uk");
    assert_eq!(p.sld, Some("city.uk".into()));
    assert_eq!(p.sll, Some("city".into()));
    assert_eq!(p.prefix, Some("foo".into()));

    // wildcard case: "foo.qux.uk" → TLD="qux.uk", SLD="foo.qux.uk", SLL="foo", Prefix=None
    let p = list.split("foo.qux.uk", opts).expect("parts");
    assert_eq!(p.tld, "qux.uk");
    assert_eq!(p.sld, Some("foo.qux.uk".into()));
    assert_eq!(p.sll, Some("foo".into()));
    assert_eq!(p.prefix, None);
}

#[test]
fn empty_psl_fallback_behavior() {
    // Expect an error for empty PSL.
    let err = List::parse("").err().expect("empty PSL should error");
    assert!(matches!(err, Error::EmptyList));
}

#[test]
fn empty_or_invalid_inputs() {
    let list = sample_list();
    let opts = MatchOpts::default();

    assert_eq!(list.tld("", opts), None);
    assert_eq!(list.sld("", opts), None);
    assert!(list.split("", opts).is_none());
}

#[test]
fn cloning_list_is_usable() {
    let list = sample_list();
    let list2 = list.clone();
    let opts = MatchOpts::default();

    assert_eq!(list.tld("foo.bar.uk", opts), Some("bar.uk".into()));
    assert_eq!(list2.tld("foo.bar.uk", opts), Some("bar.uk".into()));
}
