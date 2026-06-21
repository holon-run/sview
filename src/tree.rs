use crate::model::Node;

/// A flattened tree entry produced during AST or text collection.
///
/// `parent` stores the index into the flat `Vec` of the enclosing item
/// (or `None` for a top-level item).  After collection, [`build_tree`]
/// reconstructs the hierarchical `Vec<Node>` from the flat list.
#[derive(Debug, Clone)]
pub(crate) struct FlatItem {
    pub parent: Option<usize>,
    pub node: Node,
}

/// Reconstruct a hierarchical node tree from a flat list of [`FlatItem`]s.
///
/// Items whose `parent` matches the supplied index become children of the
/// corresponding parent.  Called with `None` to obtain the top-level nodes.
pub(crate) fn build_tree(items: &[FlatItem], parent: Option<usize>) -> Vec<Node> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.parent == parent)
        .map(|(index, item)| {
            let mut node = item.node.clone();
            node.children = build_tree(items, Some(index));
            node
        })
        .collect()
}
