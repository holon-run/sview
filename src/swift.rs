use crate::{
    model::Node,
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_swift(source: &str, preview_len: usize) -> Vec<Node> {
    let Some(tree) = parse(tree_sitter_swift::LANGUAGE.into(), source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        swift_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn swift_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "import_declaration" => ("import", swift_import_name(ast_node, source)?),
        "class_declaration" => (
            swift_declaration_kind(ast_node, source),
            swift_ast_name(ast_node, source)?,
        ),
        "protocol_declaration" => ("protocol", swift_ast_name(ast_node, source)?),
        "function_declaration" | "protocol_function_declaration" => {
            ("function", swift_ast_name(ast_node, source)?)
        }
        "init_declaration" => ("initializer", "init".to_string()),
        "property_declaration" | "protocol_property_declaration" => {
            ("property", swift_property_name(ast_node, source)?)
        }
        _ => return None,
    };

    let (start_line, end_line) = line_range(ast_node, source);

    Some(Node {
        kind: kind.to_string(),
        level: None,
        name: Some(name),
        start_line,
        end_line,
        preview: ast_preview(ast_node, source, preview_len),
        children: Vec::new(),
    })
}

fn line_range(ast_node: AstNode, source: &str) -> (usize, usize) {
    let start = ast_node.start_position();
    let end = ast_node.end_position();
    let mut end_line = if end.column == 0 && end.row > start.row {
        end.row
    } else {
        end.row + 1
    };
    let start_line = start.row + 1;
    let lines = source.lines().collect::<Vec<_>>();
    while end_line > start_line
        && lines
            .get(end_line.saturating_sub(1))
            .is_some_and(|line| line.trim().is_empty())
    {
        end_line -= 1;
    }
    (start_line, end_line)
}

fn swift_declaration_kind(ast_node: AstNode, source: &str) -> &'static str {
    let preview = ast_preview(ast_node, source, 32).unwrap_or_default();
    match preview.split_whitespace().next() {
        Some("struct") => "struct",
        Some("enum") => "enum",
        Some("extension") => "extension",
        _ => "class",
    }
}

fn swift_ast_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn swift_import_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find(|child| matches!(child.kind(), "identifier" | "simple_identifier"))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string);
    name
}

fn swift_property_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .and_then(|node| swift_bound_identifier(node, source))
        .or_else(|| swift_bound_identifier(ast_node, source))
}

fn swift_bound_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    if ast_node.kind() == "simple_identifier" {
        return ast_node
            .utf8_text(source.as_bytes())
            .ok()
            .map(ToString::to_string);
    }

    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find_map(|child| swift_bound_identifier(child, source));
    name
}
