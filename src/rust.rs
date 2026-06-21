use crate::{model::Node, util::truncate_preview};
use tree_sitter::{Node as AstNode, Parser};

pub(crate) fn analyze_rust(source: &str, preview_len: usize) -> Vec<Node> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("tree-sitter Rust grammar is valid");
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let mut items = Vec::<RustItem>::new();
    collect_rust_items(
        tree.root_node(),
        source,
        preview_len,
        None,
        false,
        &mut items,
    );
    build_rust_tree(&items, None)
}

#[derive(Debug, Clone)]
struct RustItem {
    parent: Option<usize>,
    node: Node,
}

fn build_rust_tree(items: &[RustItem], parent: Option<usize>) -> Vec<Node> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.parent == parent)
        .map(|(index, item)| {
            let mut node = item.node.clone();
            node.children = build_rust_tree(items, Some(index));
            node
        })
        .collect()
}

fn collect_rust_items(
    ast_node: AstNode,
    source: &str,
    preview_len: usize,
    parent: Option<usize>,
    has_test_attribute: bool,
    items: &mut Vec<RustItem>,
) {
    let current_parent =
        if let Some(node) = rust_node_from_ast(ast_node, source, preview_len, has_test_attribute) {
            let index = items.len();
            items.push(RustItem { parent, node });
            Some(index)
        } else {
            parent
        };

    let mut cursor = ast_node.walk();
    let mut pending_test_attribute = false;
    for child in ast_node.named_children(&mut cursor) {
        if child.kind() == "attribute_item" {
            pending_test_attribute = rust_is_test_attribute(child, source);
            continue;
        }

        collect_rust_items(
            child,
            source,
            preview_len,
            current_parent,
            pending_test_attribute,
            items,
        );
        pending_test_attribute = false;
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
        preview: rust_ast_preview(ast_node, source, preview_len),
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

fn rust_ast_preview(ast_node: AstNode, source: &str, preview_len: usize) -> Option<String> {
    source
        .lines()
        .nth(ast_node.start_position().row)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| truncate_preview(line, preview_len))
}
