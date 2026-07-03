use sview::{analyze_source, Language, Node};

#[test]
fn c_fixture_covers_includes_types_fields_and_functions() {
    let view = analyze_source(
        "tests/fixtures/c_sample.c",
        Language::C,
        include_str!("fixtures/c_sample.c"),
        80,
    );

    assert_node(&view.nodes[0], "include", "<stdio.h>", 1, 1);
    assert_node(&view.nodes[1], "struct", "Client", 3, 6);
    assert_node(&view.nodes[1].children[0], "field", "id", 4, 4);
    assert_node(&view.nodes[1].children[1], "field", "name", 5, 5);
    assert_node(&view.nodes[2], "enum", "Mode", 8, 11);
    assert_node(&view.nodes[3], "function", "client_create", 13, 15);
}

#[test]
fn cpp_fixture_covers_namespaces_classes_methods_and_functions() {
    let view = analyze_source(
        "tests/fixtures/cpp_sample.cpp",
        Language::Cpp,
        include_str!("fixtures/cpp_sample.cpp"),
        80,
    );

    assert_node(&view.nodes[0], "include", "<string>", 1, 1);
    assert_node(&view.nodes[1], "namespace", "demo", 3, 18);
    assert_node(&view.nodes[1].children[0], "class", "Client", 4, 11);
    assert_node(
        &view.nodes[1].children[0].children[0],
        "method",
        "Client",
        6,
        6,
    );
    assert_node(
        &view.nodes[1].children[0].children[1],
        "method",
        "fetch",
        7,
        7,
    );
    assert_node(
        &view.nodes[1].children[0].children[2],
        "field",
        "name_",
        10,
        10,
    );
    assert_node(
        &view.nodes[1].children[1],
        "function",
        "Client::fetch",
        13,
        13,
    );
    assert_node(&view.nodes[1].children[2], "enum", "Mode", 15, 15);
    assert_node(&view.nodes[1].children[3], "function", "add", 17, 17);
}

#[test]
fn swift_fixture_covers_apple_source_shapes() {
    let view = analyze_source(
        "tests/fixtures/swift_sample.swift",
        Language::Swift,
        include_str!("fixtures/swift_sample.swift"),
        80,
    );

    assert_node(&view.nodes[0], "import", "Foundation", 1, 1);
    assert_node(&view.nodes[1], "struct", "Client", 3, 13);
    assert_node(&view.nodes[1].children[0], "property", "title", 4, 4);
    assert_node(&view.nodes[1].children[1], "initializer", "init", 6, 8);
    assert_node(&view.nodes[1].children[2], "function", "fetch", 10, 12);
    assert_node(&view.nodes[2], "protocol", "Screen", 15, 18);
    assert_node(&view.nodes[2].children[0], "property", "name", 16, 16);
    assert_node(&view.nodes[2].children[1], "function", "render", 17, 17);
    assert_node(&view.nodes[3], "extension", "Client", 20, 22);
    assert_node(&view.nodes[3].children[0], "function", "reset", 21, 21);
    assert_node(&view.nodes[4], "enum", "Mode", 24, 27);
}

#[test]
fn objc_fixture_covers_interfaces_implementations_protocols_and_methods() {
    let view = analyze_source(
        "tests/fixtures/objc_sample.m",
        Language::ObjectiveC,
        include_str!("fixtures/objc_sample.m"),
        80,
    );

    assert_node(&view.nodes[0], "import", "<Foundation/Foundation.h>", 1, 1);
    assert_node(&view.nodes[1], "interface", "Client", 3, 7);
    assert_node(&view.nodes[1].children[0], "property", "title", 4, 4);
    assert_node(&view.nodes[1].children[1], "method", "initWithTitle:", 5, 5);
    assert_node(&view.nodes[1].children[2], "method", "kind", 6, 6);
    assert_node(&view.nodes[2], "implementation", "Client", 9, 17);
    assert_node(
        &view.nodes[2].children[0],
        "method",
        "initWithTitle:",
        10,
        12,
    );
    assert_node(&view.nodes[2].children[1], "method", "kind", 14, 16);
    assert_node(&view.nodes[3], "protocol", "Screen", 19, 21);
    assert_node(&view.nodes[3].children[0], "method", "render", 20, 20);
}

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
fn java_fixture_covers_android_class_shapes() {
    let view = analyze_source(
        "tests/fixtures/java_sample.java",
        Language::Java,
        include_str!("fixtures/java_sample.java"),
        80,
    );

    assert_node(&view.nodes[0], "package", "com.example.app", 1, 1);
    assert_node(&view.nodes[1], "import", "android.app.Activity", 3, 3);
    assert_node(&view.nodes[2], "import", "android.os.Bundle", 4, 4);
    assert_node(&view.nodes[3], "class", "MainActivity", 6, 26);
    assert_node(&view.nodes[3].children[0], "field", "title", 7, 7);
    assert_node(
        &view.nodes[3].children[1],
        "constructor",
        "MainActivity",
        9,
        11,
    );
    assert_node(&view.nodes[3].children[2], "method", "onCreate", 13, 16);
    assert_node(&view.nodes[3].children[3], "interface", "Screen", 18, 20);
    assert_node(
        &view.nodes[3].children[3].children[0],
        "method",
        "render",
        19,
        19,
    );
    assert_node(&view.nodes[3].children[4], "enum", "Mode", 22, 25);
}

#[test]
fn kotlin_fixture_covers_android_source_shapes() {
    let view = analyze_source(
        "tests/fixtures/kotlin_sample.kt",
        Language::Kotlin,
        include_str!("fixtures/kotlin_sample.kt"),
        80,
    );

    assert_node(&view.nodes[0], "package", "com.example.app", 1, 1);
    assert_node(&view.nodes[1], "import", "android.app.Activity", 3, 3);
    assert_node(&view.nodes[2], "import", "kotlinx.coroutines.*", 4, 4);
    assert_node(&view.nodes[3], "annotation", "Screen", 6, 6);
    assert_node(&view.nodes[4], "interface", "Presenter", 8, 10);
    assert_node(&view.nodes[4].children[0], "function", "start", 9, 9);
    assert_node(&view.nodes[5], "class", "UiState", 12, 12);
    assert_node(&view.nodes[6], "object", "AppConfig", 14, 16);
    assert_node(&view.nodes[6].children[0], "property", "name", 15, 15);
    assert_node(&view.nodes[7], "class", "MainActivity", 18, 28);
    assert_node(
        &view.nodes[7].children[0],
        "constructor",
        "MainActivity",
        18,
        20,
    );
    assert_node(&view.nodes[7].children[1], "property", "title", 21, 21);
    assert_node(
        &view.nodes[7].children[2],
        "constructor",
        "MainActivity",
        23,
        23,
    );
    assert_node(&view.nodes[7].children[3], "function", "onCreate", 25, 27);
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
