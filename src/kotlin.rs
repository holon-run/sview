use crate::{
    model::Node,
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_kotlin(source: &str, preview_len: usize) -> Vec<Node> {
    let Some(tree) = parse(tree_sitter_kotlin_sg::LANGUAGE.into(), source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        kotlin_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn kotlin_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "package_header" => ("package", first_identifier(ast_node, source)?),
        "import_header" => ("import", kotlin_import_name(ast_node, source)?),
        "class_declaration" => (
            kotlin_class_kind(ast_node, source),
            first_type_identifier(ast_node, source)?,
        ),
        "object_declaration" => ("object", first_type_identifier(ast_node, source)?),
        "function_declaration" => ("function", first_simple_identifier(ast_node, source)?),
        "property_declaration" => ("property", kotlin_property_name(ast_node, source)?),
        "primary_constructor" | "secondary_constructor" => {
            ("constructor", enclosing_class_name(ast_node, source)?)
        }
        _ => return None,
    };
    let (start_line, end_line) = line_range(ast_node);

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

fn line_range(ast_node: AstNode) -> (usize, usize) {
    let start = ast_node.start_position();
    let end = ast_node.end_position();
    let end_row = if end.column == 0 && end.row > start.row {
        end.row
    } else {
        end.row + 1
    };
    (start.row + 1, end_row)
}

fn kotlin_class_kind(ast_node: AstNode, source: &str) -> &'static str {
    let text = ast_node.utf8_text(source.as_bytes()).unwrap_or_default();
    if text.contains("annotation class") {
        "annotation"
    } else if text.contains("interface ") {
        "interface"
    } else if text.contains("enum class") {
        "enum"
    } else {
        "class"
    }
}

fn kotlin_import_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut name = first_identifier(ast_node, source)?;
    let mut cursor = ast_node.walk();
    if ast_node
        .named_children(&mut cursor)
        .any(|child| child.kind() == "wildcard_import")
    {
        name.push_str(".*");
    }
    Some(name)
}

fn kotlin_property_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "variable_declaration")
        .and_then(|node| first_simple_identifier(node, source));
    name
}

fn enclosing_class_name(ast_node: AstNode, source: &str) -> Option<String> {
    let mut parent = ast_node.parent();
    while let Some(node) = parent {
        if matches!(node.kind(), "class_declaration" | "object_declaration") {
            return first_type_identifier(node, source);
        }
        parent = node.parent();
    }
    None
}

fn first_type_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    first_descendant(ast_node, &["type_identifier"], source)
}

fn first_simple_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    first_descendant(ast_node, &["simple_identifier"], source)
}

fn first_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    first_descendant(ast_node, &["identifier"], source)
}

fn first_descendant(ast_node: AstNode, kinds: &[&str], source: &str) -> Option<String> {
    if kinds.contains(&ast_node.kind()) {
        return ast_node
            .utf8_text(source.as_bytes())
            .ok()
            .map(ToString::to_string);
    }

    let mut cursor = ast_node.walk();
    let name = ast_node
        .named_children(&mut cursor)
        .find_map(|child| first_descendant(child, kinds, source));
    name
}
