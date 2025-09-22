use hashbrown::HashMap;

/// PSL rule section classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    /// Rules curated by ICANN.
    Icann,
    /// Rules contributed by private orgs and service providers.
    Private,
}

/// Filter applied at match time to restrict which sections are eligible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeFilter {
    /// Allow rules from any section (ICANN and Private).
    Any,
    /// Allow only ICANN rules.
    Icann,
    /// Allow only Private rules.
    Private,
}

/// Marker placed on a trie node indicating how the label path acts as a rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Leaf {
    /// This path is not a rule; traversal may continue to children.
    #[default]
    None,
    /// Positive rule: this label path is a public suffix.
    Positive,
    /// Exception rule (PSL “!”): cancels a broader rule one label deeper.
    ///
    /// Example: with `*.uk` (positive) and `!city.uk` (exception),
    /// the host `foo.city.uk` yields TLD = "uk".
    Negative,
}

/// Node in the reverse-label trie used to match PSL rules.
///
/// Children are keyed by label strings as they appear in the list
/// (including "*" for wildcard entries). The trie is traversed from the
/// rightmost label of an input host toward the left.
#[derive(Default, Clone)]
pub struct Node {
    /// Whether this node represents a rule and of what kind.
    pub leaf: Leaf,
    /// Optional section classification for this node’s rule.
    pub typ: Option<Type>,
    /// Child labels reachable from this node.
    pub kids: HashMap<String, Node>,
}

/// Top-level container for the rule trie.
#[derive(Default, Clone)]
pub struct RuleSet {
    /// Root of the reverse-label trie (has no label itself).
    pub(crate) root: Node,
}
// -------------------------------------
// Unit tests for this private module
// -------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_are_copy_eq_and_debuggable() {
        let t1 = Type::Icann;
        let t2 = t1;
        assert_eq!(t2, Type::Icann);
        assert_ne!(t2, Type::Private);
        assert!(!format!("{:?}", t2).is_empty());

        let f1 = TypeFilter::Any;
        let f2 = f1;
        assert_eq!(f2, TypeFilter::Any);
        assert_ne!(TypeFilter::Icann, TypeFilter::Private);
        assert!(!format!("{:?}", f2).is_empty());

        let l1 = Leaf::Positive;
        let l2 = l1;
        assert_eq!(l2, Leaf::Positive);
        assert_ne!(l2, Leaf::Negative);
        assert!(!format!("{:?}", l2).is_empty());
    }

    #[test]
    fn leaf_default_is_none() {
        assert_eq!(Leaf::default(), Leaf::None);
    }

    #[test]
    fn node_default_state_is_empty() {
        let n = Node::default();
        assert_eq!(n.leaf, Leaf::None);
        assert!(n.typ.is_none());
        assert!(n.kids.is_empty());
    }

    #[test]
    fn node_kids_insert_and_get_mut() {
        let mut n = Node::default();
        n.kids.insert("com".to_string(), Node::default());
        assert!(n.kids.contains_key("com"));

        let child = n.kids.get_mut("com").unwrap();
        assert_eq!(child.leaf, Leaf::None);
        child.leaf = Leaf::Positive;
        child.typ = Some(Type::Icann);

        let child_again = n.kids.get("com").unwrap();
        assert_eq!(child_again.leaf, Leaf::Positive);
        assert_eq!(child_again.typ, Some(Type::Icann));
    }

    #[test]
    fn node_clone_is_deep_for_kids_map() {
        let mut n = Node::default();
        let sub = Node {
            leaf: Leaf::Negative,
            ..Default::default()
        };
        n.kids.insert("net".into(), sub);

        let cloned = n.clone();

        n.kids.get_mut("net").unwrap().leaf = Leaf::Positive;
        n.kids.get_mut("net").unwrap().typ = Some(Type::Private);

        let cloned_child = cloned.kids.get("net").unwrap();
        assert_eq!(cloned_child.leaf, Leaf::Negative);
        assert!(cloned_child.typ.is_none());
    }

    #[test]
    fn node_typ_option_roundtrip_and_clone() {
        let mut n = Node::default();
        assert!(n.typ.is_none());
        n.typ = Some(Type::Private);
        assert_eq!(n.typ, Some(Type::Private));

        let c = n.clone();
        assert_eq!(c.typ, Some(Type::Private));
    }

    #[test]
    fn ruleset_default_root_is_empty_node() {
        let rs = RuleSet::default();
        assert_eq!(rs.root.leaf, Leaf::None);
        assert!(rs.root.typ.is_none());
        assert!(rs.root.kids.is_empty());
    }
}
