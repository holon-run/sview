use crate::{
    model::{Language, Node},
    traverse::{analyze_tree_sitter, ast_preview, NodeClassifier},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_javascript(
    language: Language,
    source: &str,
    preview_len: usize,
) -> Vec<Node> {
    let grammar = match language {
        Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        _ => unreachable!("analyze_javascript only accepts JS/TS languages"),
    };
    analyze_tree_sitter(grammar, source, preview_len, JsClassifier)
}

struct JsClassifier;

impl NodeClassifier for JsClassifier {
    fn classify(&mut self, ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
        javascript_node_from_ast(ast_node, source, preview_len)
    }
}

fn javascript_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let kind = match ast_node.kind() {
        "function_declaration" | "generator_function_declaration" => "function",
        "class_declaration" => "class",
        "method_definition" => "method",
        "interface_declaration" => "interface",
        "type_alias_declaration" => "type",
        "enum_declaration" => "enum",
        "variable_declarator" if variable_declarator_is_function(ast_node) => "function",
        _ => return None,
    };
    let name = javascript_ast_name(ast_node, source)?;

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

fn variable_declarator_is_function(ast_node: AstNode) -> bool {
    ast_node
        .child_by_field_name("value")
        .is_some_and(|value| matches!(value.kind(), "arrow_function" | "function_expression"))
}

fn javascript_ast_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .or_else(|| ast_node.child_by_field_name("property"))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}
