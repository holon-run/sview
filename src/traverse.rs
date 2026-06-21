use crate::{
    model::Node,
    tree::{build_tree, FlatItem},
    util::truncate_preview,
};
use tree_sitter::{Language as Grammar, Node as AstNode, Parser};

/// Language-specific classification of a tree-sitter AST node.
///
/// Each language implements this trait to decide which AST nodes are
/// structural items worth surfacing, what `kind` label they receive, and
/// how their display name is derived.  The shared traversal machinery in
/// [`collect_items`] / [`analyze_tree_sitter`] handles the recursive walk
/// and flat-list-to-tree reconstruction.
///
/// For languages that need to propagate sibling-level state (e.g. Rust's
/// `#[test]` attribute), the classifier can maintain internal mutable
/// state: [`observe_sibling`](NodeClassifier::observe_sibling) is called
/// for each named child before recursion, and the classifier decides
/// whether to skip the node and/or stash context for the next sibling's
/// [`classify`](NodeClassifier::classify) call.
pub(crate) trait NodeClassifier {
    /// Classify `ast_node` into an output [`Node`].
    ///
    /// Returns `None` when the node is not a structural item.
    fn classify(&mut self, ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node>;

    /// Inspect a named child before the traversal recurses into it.
    ///
    /// Returns `true` to skip the node entirely (do not recurse or
    /// classify).  This is where languages detect attribute/decorator
    /// nodes and stash state that the next [`classify`] call reads.
    ///
    /// [`classify`]: NodeClassifier::classify
    fn observe_sibling(&mut self, _ast_node: AstNode, _source: &str) -> bool {
        false
    }
}

/// Parse `source` with the given tree-sitter `grammar`, traverse the AST
/// using `classifier`, and return the hierarchical node tree.
///
/// This is the shared entry point for all tree-sitter-based analyzers.
/// Language-specific behaviour is supplied via the [`NodeClassifier`]
/// implementation.
pub(crate) fn analyze_tree_sitter(
    grammar: Grammar,
    source: &str,
    preview_len: usize,
    classifier: impl NodeClassifier,
) -> Vec<Node> {
    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .expect("tree-sitter grammar is valid");
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    let mut classifier = classifier;
    collect_items(
        tree.root_node(),
        source,
        preview_len,
        None,
        &mut classifier,
        &mut items,
    );
    build_tree(&items, None)
}

/// Recursively walk `ast_node`'s named children, classifying each node
/// via `classifier` and accumulating [`FlatItem`]s into `items`.
///
/// Nodes that produce an output [`Node`] become parents for their
/// descendants; nodes that do not are transparent (children attach to the
/// current parent).
fn collect_items(
    ast_node: AstNode,
    source: &str,
    preview_len: usize,
    parent: Option<usize>,
    classifier: &mut impl NodeClassifier,
    items: &mut Vec<FlatItem>,
) {
    let current_parent = if let Some(node) = classifier.classify(ast_node, source, preview_len) {
        let index = items.len();
        items.push(FlatItem { parent, node });
        Some(index)
    } else {
        parent
    };

    let mut cursor = ast_node.walk();
    for child in ast_node.named_children(&mut cursor) {
        if classifier.observe_sibling(child, source) {
            continue;
        }
        collect_items(
            child,
            source,
            preview_len,
            current_parent,
            classifier,
            items,
        );
    }
}

/// Extract the first non-empty source line at the node's start position,
/// trimmed and truncated to `preview_len` characters.
pub(crate) fn ast_preview(ast_node: AstNode, source: &str, preview_len: usize) -> Option<String> {
    source
        .lines()
        .nth(ast_node.start_position().row)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| truncate_preview(line, preview_len))
}
