use crate::{
    model::Node,
    traverse::{analyze_tree_sitter, ast_preview, NodeClassifier},
    util::truncate_preview,
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_rust(source: &str, preview_len: usize) -> Vec<Node> {
    analyze_tree_sitter(
        tree_sitter_rust::LANGUAGE.into(),
        source,
        preview_len,
        RustClassifier::default(),
    )
}

#[derive(Default)]
struct RustClassifier {
    /// Set when the preceding sibling was a `#[test]` attribute.
    /// Consumed by the next `classify` call.
    pending_test_attribute: bool,
}

impl NodeClassifier for RustClassifier {
    fn classify(&mut self, ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
        let has_test_attribute = self.pending_test_attribute;
        self.pending_test_attribute = false;
        rust_node_from_ast(ast_node, source, preview_len, has_test_attribute)
    }

    fn observe_sibling(&mut self, ast_node: AstNode, source: &str) -> bool {
        if ast_node.kind() == "attribute_item" {
            // Attribute nodes are not structural items; skip them but
            // record test-attribute status for the next sibling.
            self.pending_test_attribute = rust_is_test_attribute(ast_node, source);
            true
        } else {
            false
        }
    }
}

fn rust_node_from_ast(
    ast_node: AstNode,
    source: &str,
    preview_len: usize,
    has_test_attribute: bool,
) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "mod_item" => ("module", rust_ast_name(ast_node, source)?),
        "struct_item" => ("struct", rust_ast_name(ast_node, source)?),
        "enum_item" => ("enum", rust_ast_name(ast_node, source)?),
        "trait_item" => ("trait", rust_ast_name(ast_node, source)?),
        "impl_item" => ("impl", rust_impl_ast_name(ast_node, source)),
        "function_item" => {
            let kind = if has_test_attribute {
                "test"
            } else {
                "function"
            };
            (kind, rust_ast_name(ast_node, source)?)
        }
        _ => return None,
    };

    Some(Node {
        kind: kind.to_string(),
        level: None,
        name: Some(name),
        start_line: ast_node.start_position().row + 1,
        end_line: ast_node.end_position().row + 1,
        preview: ast_preview(ast_node, source, preview_len),
        children: Vec::new(),
    })
}

fn rust_ast_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn rust_impl_ast_name(ast_node: AstNode, source: &str) -> String {
    let text = ast_node.utf8_text(source.as_bytes()).unwrap_or("impl");
    let signature = text
        .split('{')
        .next()
        .unwrap_or(text)
        .trim()
        .trim_end_matches(';')
        .trim();
    truncate_preview(signature, 80)
}

fn rust_is_test_attribute(ast_node: AstNode, source: &str) -> bool {
    ast_node
        .utf8_text(source.as_bytes())
        .is_ok_and(|text| text.starts_with("#[test") || text.starts_with("#[tokio::test"))
}
