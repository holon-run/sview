use crate::{
    analyzer::{build_tree, AnalyzerItem},
    model::{Language, Node},
    util::truncate_preview,
};
use tree_sitter::{Node as AstNode, Parser};

pub(crate) fn analyze_javascript(
    language: Language,
    source: &str,
    preview_len: usize,
) -> Vec<Node> {
    let mut parser = Parser::new();
    let grammar = match language {
        Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        _ => unreachable!("analyze_javascript only accepts JS/TS languages"),
    };
    parser
        .set_language(&grammar)
        .expect("tree-sitter JavaScript/TypeScript grammar is valid");
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let mut items = Vec::<AnalyzerItem>::new();
    collect_javascript_items(tree.root_node(), source, preview_len, None, &mut items);
    build_tree(&items, None)
}

fn collect_javascript_items(
    ast_node: AstNode,
    source: &str,
    preview_len: usize,
    parent: Option<usize>,
    items: &mut Vec<AnalyzerItem>,
) {
    let current_parent = if let Some(node) = javascript_node_from_ast(ast_node, source, preview_len)
    {
        let index = items.len();
        items.push(AnalyzerItem { parent, node });
        Some(index)
    } else {
        parent
    };

    let mut cursor = ast_node.walk();
    for child in ast_node.named_children(&mut cursor) {
        collect_javascript_items(child, source, preview_len, current_parent, items);
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
        preview: javascript_ast_preview(ast_node, source, preview_len),
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

fn javascript_ast_preview(ast_node: AstNode, source: &str, preview_len: usize) -> Option<String> {
    source
        .lines()
        .nth(ast_node.start_position().row)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| truncate_preview(line, preview_len))
}
