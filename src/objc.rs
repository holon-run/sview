use crate::{
    model::Node,
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_objc(source: &str, preview_len: usize) -> Vec<Node> {
    let Some(tree) = parse(tree_sitter_objc::LANGUAGE.into(), source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        objc_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn objc_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "preproc_include" => ("import", objc_include_name(ast_node, source)?),
        "class_interface" => ("interface", objc_first_identifier(ast_node, source)?),
        "class_implementation" => ("implementation", objc_first_identifier(ast_node, source)?),
        "category_interface" => ("category", objc_category_name(ast_node, source)?),
        "category_implementation" => ("category", objc_category_name(ast_node, source)?),
        "protocol_declaration" => ("protocol", objc_first_identifier(ast_node, source)?),
        "property_declaration" => ("property", objc_property_name(ast_node, source)?),
        "method_declaration" | "method_definition" => {
            ("method", objc_method_name(ast_node, source)?)
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

fn objc_include_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("path")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn objc_first_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "identifier")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string);
    name
}

fn objc_category_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let identifiers = ast_node
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "identifier")
        .filter_map(|node| node.utf8_text(source.as_bytes()).ok())
        .take(2)
        .collect::<Vec<_>>();
    match identifiers.as_slice() {
        [class_name, category_name] => Some(format!("{class_name}({category_name})")),
        [class_name] => Some((*class_name).to_string()),
        _ => None,
    }
}

fn objc_property_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "struct_declaration")
        .and_then(|node| objc_declarator_identifier(node, source));
    name
}

fn objc_method_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let parts = ast_node
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "identifier")
        .filter_map(|node| node.utf8_text(source.as_bytes()).ok())
        .collect::<Vec<_>>();
    let first = parts.first()?;
    if ast_node
        .named_children(&mut ast_node.walk())
        .any(|child| child.kind() == "method_parameter")
    {
        Some(format!("{first}:"))
    } else {
        Some((*first).to_string())
    }
}

fn objc_declarator_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    if ast_node.kind() == "identifier" {
        return ast_node
            .utf8_text(source.as_bytes())
            .ok()
            .map(ToString::to_string);
    }

    if let Some(declarator) = ast_node.child_by_field_name("declarator") {
        return objc_declarator_identifier(declarator, source);
    }

    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find_map(|child| objc_declarator_identifier(child, source));
    name
}
