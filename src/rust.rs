use crate::{
    model::Node,
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
    util::truncate_preview,
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_rust(source: &str, preview_len: usize) -> Vec<Node> {
    let Some(tree) = parse(tree_sitter_rust::LANGUAGE.into(), source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        rust_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn rust_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let has_test_attribute = ast_node.prev_named_sibling().is_some_and(|sibling| {
        sibling.kind() == "attribute_item" && rust_is_test_attribute(sibling, source)
    });

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
