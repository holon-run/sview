use crate::{
    model::{Language, Node},
    tree::{ast_preview, build_tree, collect_items, parse, FlatItem},
};
use tree_sitter::Node as AstNode;

pub(crate) fn analyze_cpp_like(language: Language, source: &str, preview_len: usize) -> Vec<Node> {
    let grammar = match language {
        Language::C => tree_sitter_c::LANGUAGE.into(),
        Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        _ => unreachable!("analyze_cpp_like only accepts C/C++ languages"),
    };
    let Some(tree) = parse(grammar, source) else {
        return Vec::new();
    };

    let mut items = Vec::<FlatItem>::new();
    collect_items(tree.root_node(), None, &mut items, &|ast_node| {
        cpp_node_from_ast(ast_node, source, preview_len)
    });
    build_tree(&items, None)
}

fn cpp_node_from_ast(ast_node: AstNode, source: &str, preview_len: usize) -> Option<Node> {
    let (kind, name) = match ast_node.kind() {
        "preproc_include" => ("include", include_name(ast_node, source)?),
        "namespace_definition" => ("namespace", ast_name(ast_node, source)?),
        "class_specifier" => ("class", ast_name(ast_node, source)?),
        "struct_specifier" => ("struct", ast_name(ast_node, source)?),
        "union_specifier" => ("union", ast_name(ast_node, source)?),
        "enum_specifier" => ("enum", ast_name(ast_node, source)?),
        "function_definition" => (function_kind(ast_node), declarator_name(ast_node, source)?),
        "declaration" if declaration_is_function(ast_node) => {
            (function_kind(ast_node), declarator_name(ast_node, source)?)
        }
        "field_declaration" if declaration_is_function(ast_node) => {
            ("method", declarator_name(ast_node, source)?)
        }
        "field_declaration" => ("field", declarator_name(ast_node, source)?),
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

fn ast_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("name")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn include_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("path")
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(ToString::to_string)
}

fn function_kind(ast_node: AstNode) -> &'static str {
    if is_inside_type(ast_node) {
        "method"
    } else {
        "function"
    }
}

fn declaration_is_function(ast_node: AstNode) -> bool {
    ast_node
        .child_by_field_name("declarator")
        .is_some_and(contains_function_declarator)
}

fn declarator_name(ast_node: AstNode, source: &str) -> Option<String> {
    ast_node
        .child_by_field_name("declarator")
        .or_else(|| first_declarator(ast_node))
        .and_then(|node| declarator_identifier(node, source))
}

fn first_declarator(ast_node: AstNode) -> Option<AstNode> {
    let mut cursor = ast_node.walk();
    let declarator = ast_node
        .named_children(&mut cursor)
        .find(|child| child.kind().contains("declarator"));
    declarator
}

fn contains_function_declarator(ast_node: AstNode) -> bool {
    if ast_node.kind() == "function_declarator" {
        return true;
    }

    let mut cursor = ast_node.walk();
    let contains = ast_node
        .named_children(&mut cursor)
        .any(contains_function_declarator);
    contains
}

fn declarator_identifier(ast_node: AstNode, source: &str) -> Option<String> {
    if matches!(
        ast_node.kind(),
        "identifier" | "field_identifier" | "type_identifier" | "qualified_identifier"
    ) {
        return ast_node
            .utf8_text(source.as_bytes())
            .ok()
            .map(ToString::to_string);
    }

    if let Some(declarator) = ast_node.child_by_field_name("declarator") {
        return declarator_identifier(declarator, source);
    }

    let mut cursor = ast_node.walk();
    let identifier = ast_node
        .named_children(&mut cursor)
        .find_map(|child| declarator_identifier(child, source));
    identifier
}

fn is_inside_type(ast_node: AstNode) -> bool {
    let mut parent = ast_node.parent();
    while let Some(node) = parent {
        if matches!(
            node.kind(),
            "class_specifier" | "struct_specifier" | "union_specifier"
        ) {
            return true;
        }
        parent = node.parent();
    }
    false
}
