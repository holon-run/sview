use crate::model::Node;
use tree_sitter::{Node as AstNode, Parser};

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

/// Parse source code with the given tree-sitter grammar, returning `None`
/// on parse failure.  This centralizes parser setup so that each language
/// analyzer only needs to supply its grammar.
pub(crate) fn parse(grammar: tree_sitter::Language, source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .expect("tree-sitter grammar is valid");
    parser.parse(source, None)
}

/// Walk `ast_node` and its named descendants, collecting [`FlatItem`]s into
/// `items`.
///
/// `classify` is called for every named node.  When it returns a `Node`,
/// that node is appended to `items` and its index becomes the parent for
/// any nodes produced from its descendants; otherwise traversal continues
/// without introducing a new parent.
pub(crate) fn collect_items<F>(
    ast_node: AstNode,
    parent: Option<usize>,
    items: &mut Vec<FlatItem>,
    classify: &F,
) where
    F: Fn(AstNode) -> Option<Node>,
{
    let current_parent = match classify(ast_node) {
        Some(node) => {
            let index = items.len();
            items.push(FlatItem { parent, node });
            Some(index)
        }
        None => parent,
    };

    let mut cursor = ast_node.walk();
    for child in ast_node.named_children(&mut cursor) {
        collect_items(child, current_parent, items, classify);
    }
}

/// Extract a single-line preview from `source` for the given AST node.
///
/// The first non-empty line at the node's start position is trimmed and
/// truncated to `preview_len` characters.  Returns `None` when the line
/// is empty.
pub(crate) fn ast_preview(ast_node: AstNode, source: &str, preview_len: usize) -> Option<String> {
    source
        .lines()
        .nth(ast_node.start_position().row)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| crate::util::truncate_preview(line, preview_len))
}
