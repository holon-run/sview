use crate::{
    model::Node,
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_java(source: &str, preview_len: usize) -> Vec<Node> {
    let Some(tree) = parse(tree_sitter_java::LANGUAGE.into(), source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        java_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn java_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "package_declaration" => ("package", java_package_name(ast_node, source)?),
        "import_declaration" => ("import", java_import_name(ast_node, source)?),
        "class_declaration" => ("class", java_ast_name(ast_node, source)?),
        "interface_declaration" => ("interface", java_ast_name(ast_node, source)?),
        "enum_declaration" => ("enum", java_ast_name(ast_node, source)?),
        "annotation_type_declaration" => ("annotation", java_ast_name(ast_node, source)?),
        "constructor_declaration" => ("constructor", java_ast_name(ast_node, source)?),
        "method_declaration" => ("method", java_ast_name(ast_node, source)?),
        "field_declaration" => ("field", java_field_name(ast_node, source)?),
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

fn java_ast_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn java_field_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "variable_declarator")
        .and_then(|declarator| java_ast_name(declarator, source));
    name
}

fn java_package_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .or_else(|| last_scoped_identifier(ast_node))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn java_import_name(ast_node: AstNode, source: &str) -> Option<String> {
    last_scoped_identifier(ast_node)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn last_scoped_identifier(ast_node: AstNode) -> Option<AstNode> {
    let mut cursor = ast_node.walk();
    ast_node
        .named_children(&mut cursor)
        .filter(|child| matches!(child.kind(), "scoped_identifier" | "identifier"))
        .last()
}
