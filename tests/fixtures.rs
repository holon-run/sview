use sview::{analyze_source, Language, Node};

#[test]
fn rust_fixture_covers_ast_shapes() {
    let view = analyze_source(
        "tests/fixtures/rust_sample.rs",
        Language::Rust,
        include_str!("fixtures/rust_sample.rs"),
        80,
    );

    assert_node(&view.nodes[0], "struct", "Client", 6, 8);
    assert_node(&view.nodes[1], "enum", "Mode", 10, 13);
    assert_node(&view.nodes[2], "trait", "Service", 15, 17);
    assert_node(&view.nodes[3], "impl", "impl Client", 19, 24);
    assert_node(&view.nodes[3].children[0], "function", "build", 20, 23);
    assert_node(&view.nodes[4], "module", "tests", 26, 29);
    assert_node(&view.nodes[4].children[0], "test", "builds_client", 28, 28);
}

#[test]
fn javascript_fixture_covers_functions_classes_and_methods() {
    let view = analyze_source(
        "tests/fixtures/javascript_sample.js",
        Language::JavaScript,
        include_str!("fixtures/javascript_sample.js"),
        80,
    );

    assert_node(&view.nodes[0], "function", "loadUser", 1, 3);
    assert_node(&view.nodes[1], "class", "Client", 5, 9);
    assert_node(&view.nodes[1].children[0], "method", "fetch", 6, 8);
    assert_node(&view.nodes[2], "function", "helper", 11, 11);
}

#[test]
fn typescript_fixture_covers_types_and_classes() {
    let view = analyze_source(
        "tests/fixtures/typescript_sample.ts",
        Language::TypeScript,
        include_str!("fixtures/typescript_sample.ts"),
        80,
    );

    assert_node(&view.nodes[0], "interface", "User", 1, 3);
    assert_node(&view.nodes[1], "type", "UserId", 5, 5);
    assert_node(&view.nodes[2], "enum", "Mode", 7, 10);
    assert_node(&view.nodes[3], "class", "Service", 12, 16);
    assert_node(&view.nodes[3].children[0], "method", "load", 13, 15);
}

#[test]
fn tsx_fixture_covers_function_components() {
    let view = analyze_source(
        "tests/fixtures/tsx_sample.tsx",
        Language::Tsx,
        include_str!("fixtures/tsx_sample.tsx"),
        80,
    );

    assert_node(&view.nodes[0], "function", "App", 1, 3);
    assert_node(&view.nodes[1], "function", "Header", 5, 5);
}

#[test]
fn markdown_fixture_covers_document_shapes() {
    let view = analyze_source(
        "tests/fixtures/markdown_sample.md",
        Language::Markdown,
        include_str!("fixtures/markdown_sample.md"),
        80,
    );

    assert_node(&view.nodes[0], "frontmatter", "frontmatter", 1, 3);
    assert_node(&view.nodes[1], "heading", "Intro", 5, 14);
    assert_node(&view.nodes[1].children[0], "list", "list", 7, 9);
    assert_node(&view.nodes[1].children[1], "heading", "Details", 10, 14);
    assert_node(
        &view.nodes[1].children[1].children[0],
        "code_block",
        "rust",
        12,
        14,
    );
}

fn assert_node(node: &Node, kind: &str, name: &str, start_line: usize, end_line: usize) {
    assert_eq!(node.kind, kind);
    assert_eq!(node.name.as_deref(), Some(name));
    assert_eq!(node.start_line, start_line);
    assert_eq!(node.end_line, end_line);
}
