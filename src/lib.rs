mod analyzer;
mod markdown;
mod model;
mod render;
mod rust;
mod util;

pub use analyzer::{analyze_file, analyze_source, detect_language};
pub use model::{Language, Node, RenderOptions, StructureView};
pub use render::render_text;
pub use util::truncate_preview;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_language_from_extension() {
        assert_eq!(
            detect_language(Path::new("README.md"), ""),
            Language::Markdown
        );
        assert_eq!(detect_language(Path::new("src/lib.rs"), ""), Language::Rust);
        assert_eq!(detect_language(Path::new("LICENSE"), ""), Language::Unknown);
    }

    #[test]
    fn creates_file_node_for_non_empty_source() {
        let view = analyze_source("example.txt", Language::Unknown, "\nhello\nworld\n", 20);

        assert_eq!(view.nodes.len(), 1);
        assert_eq!(view.nodes[0].kind, "file");
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 3);
        assert_eq!(view.nodes[0].preview.as_deref(), Some("hello"));
    }

    #[test]
    fn truncates_preview_on_character_boundary() {
        assert_eq!(truncate_preview("abcdef", 4), "abc…");
        assert_eq!(truncate_preview("ab", 4), "ab");
    }

    #[test]
    fn extracts_markdown_sections() {
        let source = "---\ntitle: Test\n---\n# Intro\n\n- one\n- two\n\n## Details\n\n```rust\nfn main() {}\n```\n# Next\n";
        let view = analyze_source("README.md", Language::Markdown, source, 40);

        assert_eq!(view.nodes[0].kind, "frontmatter");
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 3);

        let intro = &view.nodes[1];
        assert_eq!(intro.kind, "heading");
        assert_eq!(intro.name.as_deref(), Some("Intro"));
        assert_eq!(intro.start_line, 4);
        assert_eq!(intro.end_line, 13);
        assert_eq!(intro.children[0].kind, "list");
        assert_eq!(intro.children[1].name.as_deref(), Some("Details"));
        assert_eq!(intro.children[1].children[0].kind, "code_block");
        assert_eq!(view.nodes[2].name.as_deref(), Some("Next"));
    }

    #[test]
    fn renders_text_as_tree_outline() {
        let source = "# Intro\n\n## Details\n";
        let view = analyze_source("README.md", Language::Markdown, source, 40);
        let output = render_text(
            &view,
            &RenderOptions {
                max_depth: None,
                max_nodes: 20,
            },
        );

        assert_eq!(
            output,
            "README.md (markdown)\n└─ heading Intro L1-3\n   └─ heading Details L3-3\n"
        );
    }

    #[test]
    fn extracts_rust_items() {
        let source = "pub mod api {\n    pub struct Client;\n\n    impl Client {\n        pub fn new() -> Self {\n            Self\n        }\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn creates_client() {}\n}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        let api = &view.nodes[0];
        assert_eq!(api.kind, "module");
        assert_eq!(api.name.as_deref(), Some("api"));
        assert_eq!(api.start_line, 1);
        assert_eq!(api.end_line, 9);
        assert_eq!(api.children[0].kind, "struct");
        assert_eq!(api.children[1].kind, "impl");
        assert_eq!(api.children[1].children[0].kind, "function");
        assert_eq!(api.children[1].children[0].name.as_deref(), Some("new"));
        assert_eq!(api.children[1].children[0].end_line, 7);

        let tests = &view.nodes[1];
        assert_eq!(tests.name.as_deref(), Some("tests"));
        assert_eq!(tests.children[0].kind, "test");
        assert_eq!(tests.children[0].name.as_deref(), Some("creates_client"));
    }

    #[test]
    fn tracks_multiline_rust_signatures() {
        let source = "pub fn build(\n    input: String,\n) -> String {\n    input\n}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        assert_eq!(view.nodes[0].kind, "function");
        assert_eq!(view.nodes[0].name.as_deref(), Some("build"));
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 5);
    }

    #[test]
    fn braces_in_char_literals_do_not_hide_closing_blocks() {
        let source = "fn contains_quote() {\n    let quote = '\"';\n}\nfn next() {}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        assert_eq!(view.nodes[0].name.as_deref(), Some("contains_quote"));
        assert_eq!(view.nodes[0].end_line, 3);
        assert_eq!(view.nodes[1].name.as_deref(), Some("next"));
    }

    #[test]
    fn extracts_rust_ast_fixture_shapes() {
        let source = "macro_rules! make_client { () => {}; }\n\n#[derive(Debug)]\npub struct Client {\n    value: String,\n}\n\npub enum Mode { Fast, Slow }\n\npub trait Service {\n    fn call(&self);\n}\n\nimpl Client {\n    pub async fn build(\n        value: String,\n    ) -> Self {\n        make_client!();\n        Self { value }\n    }\n}\n\nmod tests {\n    #[tokio::test(flavor = \"current_thread\")]\n    async fn builds_client() {}\n}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        assert_eq!(view.nodes[0].kind, "struct");
        assert_eq!(view.nodes[0].name.as_deref(), Some("Client"));
        assert_eq!(view.nodes[0].start_line, 4);
        assert_eq!(view.nodes[0].end_line, 6);
        assert_eq!(view.nodes[1].kind, "enum");
        assert_eq!(view.nodes[1].name.as_deref(), Some("Mode"));
        assert_eq!(view.nodes[2].kind, "trait");
        assert_eq!(view.nodes[2].name.as_deref(), Some("Service"));

        let impl_node = &view.nodes[3];
        assert_eq!(impl_node.kind, "impl");
        assert_eq!(impl_node.name.as_deref(), Some("impl Client"));
        assert_eq!(impl_node.start_line, 14);
        assert_eq!(impl_node.end_line, 21);
        assert_eq!(impl_node.children[0].kind, "function");
        assert_eq!(impl_node.children[0].name.as_deref(), Some("build"));
        assert_eq!(impl_node.children[0].start_line, 15);
        assert_eq!(impl_node.children[0].end_line, 20);

        let tests = &view.nodes[4];
        assert_eq!(tests.kind, "module");
        assert_eq!(tests.name.as_deref(), Some("tests"));
        assert_eq!(tests.children[0].kind, "test");
        assert_eq!(tests.children[0].name.as_deref(), Some("builds_client"));
    }
}
